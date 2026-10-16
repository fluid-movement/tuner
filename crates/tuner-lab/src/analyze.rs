//! Per-take analysis: single-string experience on the plucks, and strum
//! estimates checked against the plucks as ground truth.

use crate::audio::{self, Audio};
use crate::segment::{Event, Kind, segment};
use serde::Serialize;
use std::path::Path;
use tuner_core::MonoTuner;
use tuner_core::poly::{PolyConfig, estimate_resolved};
use tuner_core::tuning::{Instrument, cents_between};

/// Hop between tracker frames, as in the app.
const HOP: usize = 1024;
/// Window after a pluck's onset whose tracked readings define its truth.
const TRUTH_WINDOW: (f32, f32) = (0.2, 1.5);
/// The experience metrics look at the first two seconds of each pluck.
const EXPERIENCE_SECONDS: f32 = 2.0;
/// A shown reading counts as the plucked note within this many cents.
const SAME_NOTE_CENTS: f32 = 50.0;
/// In tune within this many cents of the target.
pub const IN_TUNE_CENTS: f32 = 5.0;
/// Truths this close to the in-tune boundary are too ambiguous to judge.
pub const BOUNDARY_CENTS: f32 = 1.5;

#[derive(Debug, Clone, Serialize)]
pub struct PluckResult {
    pub onset: f32,
    pub string: &'static str,
    pub string_index: usize,
    /// Median tracked frequency 0.2–1.5 s after the onset (the truth).
    pub frequency: f32,
    /// Offset of that frequency from the string's target.
    pub cents: f32,
    /// Time from the onset to the first reading of this note.
    pub first_reading_ms: Option<f32>,
    /// Share of the first 2 s (or the segment) showing this note.
    pub shown_share: f32,
    /// Max − min of the shown value 0.2–1.5 s after the onset, in cents.
    pub spread_cents: f32,
    /// Median shown value 1.0–1.5 s minus 0.2–0.5 s, in cents.
    pub drift_cents: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    InTune,
    Flat,
    Sharp,
}

pub fn verdict(cents: f32) -> Verdict {
    if cents.abs() <= IN_TUNE_CENTS {
        Verdict::InTune
    } else if cents < 0.0 {
        Verdict::Flat
    } else {
        Verdict::Sharp
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct StrumString {
    pub string: &'static str,
    /// Truth from this take's pluck of the string, cents from the target.
    pub truth_cents: Option<f32>,
    /// Strum estimate, cents from the target.
    pub estimate_cents: Option<f32>,
    /// Estimate − truth, in cents.
    pub error_cents: Option<f32>,
    /// Whether the in-tune/flat/sharp verdicts agree; `None` when there is
    /// no truth or estimate, or the truth is too close to the boundary.
    pub agree: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StrumResult {
    pub onset: f32,
    /// Seconds of audio analysed (0.68 s at 48 kHz normally).
    pub window_seconds: f32,
    pub strings: Vec<StrumString>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TakeReport {
    pub file: String,
    pub instrument: &'static str,
    pub seconds: f32,
    pub sample_rate: f32,
    pub events: Vec<Event>,
    pub plucks: Vec<PluckResult>,
    pub strums: Vec<StrumResult>,
    pub notes: Vec<String>,
}

/// The instrument named at the start of a file name (`guitar…`, `gtr…`,
/// `ukulele…`, `uke…`).
pub fn instrument_from_name(name: &str) -> Option<Instrument> {
    let n = name.to_ascii_lowercase();
    if n.starts_with("guitar") || n.starts_with("gtr") {
        Some(Instrument::Guitar)
    } else if n.starts_with("ukulele") || n.starts_with("uke") {
        Some(Instrument::Ukulele)
    } else {
        None
    }
}

pub fn instrument_name(instrument: Instrument) -> &'static str {
    match instrument {
        Instrument::Guitar => "guitar",
        Instrument::Ukulele => "ukulele",
    }
}

pub fn analyze_file(path: &Path) -> Result<TakeReport, String> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default();
    let instrument = instrument_from_name(name)
        .ok_or_else(|| format!("{name}: file name must start with guitar/gtr or ukulele/uke"))?;
    let audio = audio::load(path)?;
    Ok(analyze(name, &audio, instrument))
}

pub fn analyze(name: &str, audio: &Audio, instrument: Instrument) -> TakeReport {
    let events = segment(audio, instrument);
    let mut notes = Vec::new();
    let plucks: Vec<PluckResult> = events
        .iter()
        .filter(|e| e.kind == Kind::Pluck)
        .filter_map(|e| {
            let r = analyze_pluck(audio, instrument, e);
            if r.is_none() {
                notes.push(format!("pluck at {:.2}s: no stable reading", e.onset));
            }
            r
        })
        .collect();

    // One truth per string: the median over its plucks.
    let truths: Vec<Option<f32>> = (0..instrument.strings().len())
        .map(|i| {
            median(
                plucks
                    .iter()
                    .filter(|p| p.string_index == i)
                    .map(|p| p.frequency)
                    .collect(),
            )
        })
        .collect();
    for (i, t) in instrument.strings().iter().enumerate() {
        if truths[i].is_none() {
            notes.push(format!(
                "no pluck of {}: its strum results have no truth",
                t.name
            ));
        }
    }

    let strums = events
        .iter()
        .filter(|e| e.kind == Kind::Strum)
        .filter_map(|e| {
            let r = analyze_strum(audio, instrument, e, &truths);
            if r.is_none() {
                notes.push(format!("strum at {:.2}s: too short to analyse", e.onset));
            }
            r
        })
        .collect();

    TakeReport {
        file: name.to_string(),
        instrument: instrument_name(instrument),
        seconds: audio.seconds(),
        sample_rate: audio.sample_rate,
        events,
        plucks,
        strums,
        notes,
    }
}

fn analyze_pluck(audio: &Audio, instrument: Instrument, event: &Event) -> Option<PluckResult> {
    let rate = audio.sample_rate;
    let mut tuner = MonoTuner::new(rate, HOP);
    let n = tuner.frame_size();
    // Start a little early so the tracker learns the background.
    let start = ((event.onset - 0.5).max(0.0) * rate) as usize;
    let end = ((event.end * rate) as usize).min(audio.samples.len());
    // (seconds after the onset, shown frequency, tracking?)
    let mut frames: Vec<(f32, Option<f32>, bool)> = Vec::new();
    let mut i = start;
    while i + n <= end {
        let out = tuner.process(&audio.samples[i..i + n]);
        let t = (i + n) as f32 / rate - event.onset;
        frames.push((
            t,
            out.frequency,
            out.state == tuner_core::TrackState::Tracking,
        ));
        i += HOP;
    }

    let in_window = |(a, b): (f32, f32)| {
        frames
            .iter()
            .filter(move |(t, f, tracking)| *t >= a && *t <= b && *tracking && f.is_some())
            .map(|(_, f, _)| f.unwrap())
    };
    let truth_readings: Vec<f32> = in_window(TRUTH_WINDOW).collect();
    if truth_readings.len() < 5 {
        return None;
    }
    let frequency = median(truth_readings.clone())?;
    let (string_index, target, cents) = instrument.nearest_string(frequency);

    let same_note =
        |f: Option<f32>| f.is_some_and(|f| cents_between(f, frequency).abs() <= SAME_NOTE_CENTS);
    let first_reading_ms = frames
        .iter()
        .find(|(t, f, _)| *t >= 0.0 && same_note(*f))
        .map(|(t, ..)| t * 1e3);
    let horizon = EXPERIENCE_SECONDS.min(event.end - event.onset);
    let window: Vec<_> = frames
        .iter()
        .filter(|(t, ..)| *t >= 0.0 && *t <= horizon)
        .collect();
    let shown = window.iter().filter(|(_, f, _)| same_note(*f)).count();
    let shown_share = if window.is_empty() {
        0.0
    } else {
        shown as f32 / window.len() as f32
    };

    let shown_cents: Vec<f32> = frames
        .iter()
        .filter(|(t, f, _)| *t >= TRUTH_WINDOW.0 && *t <= TRUTH_WINDOW.1 && same_note(*f))
        .map(|(_, f, _)| cents_between(f.unwrap(), frequency))
        .collect();
    let spread_cents = shown_cents.iter().copied().fold(f32::MIN, f32::max)
        - shown_cents.iter().copied().fold(f32::MAX, f32::min);
    let median_cents = |a: f32, b: f32| {
        median(
            frames
                .iter()
                .filter(|(t, f, _)| *t >= a && *t <= b && same_note(*f))
                .map(|(_, f, _)| cents_between(f.unwrap(), frequency))
                .collect(),
        )
    };
    let drift_cents = match (median_cents(0.2, 0.5), median_cents(1.0, 1.5)) {
        (Some(early), Some(late)) => Some(late - early),
        _ => None,
    };

    Some(PluckResult {
        onset: event.onset,
        string: target.name,
        string_index,
        frequency,
        cents,
        first_reading_ms,
        shown_share,
        spread_cents: spread_cents.max(0.0),
        drift_cents,
    })
}

fn analyze_strum(
    audio: &Audio,
    instrument: Instrument,
    event: &Event,
    truths: &[Option<f32>],
) -> Option<StrumResult> {
    let rate = audio.sample_rate;
    // At least 80 ms after the last string of the strum (strum research).
    let start = ((event.onset + 0.12) * rate) as usize;
    let available = ((event.end * rate) as usize)
        .min(audio.samples.len())
        .saturating_sub(start);
    let len = [32_768usize, 16_384]
        .into_iter()
        .find(|&n| n <= available)?;
    let config = PolyConfig::new(rate, instrument);
    let estimates = estimate_resolved(&audio.samples[start..start + len], instrument, &config);

    let strings = instrument
        .strings()
        .iter()
        .zip(estimates)
        .zip(truths)
        .map(|((target, est), truth)| {
            let truth_cents = truth.map(|f| cents_between(f, target.frequency));
            let estimate_cents = est.map(|e| e.cents);
            let error_cents = match (est, truth) {
                (Some(e), Some(t)) => Some(cents_between(e.frequency, *t)),
                _ => None,
            };
            let agree = match (truth_cents, estimate_cents) {
                (Some(t), Some(e)) if (t.abs() - IN_TUNE_CENTS).abs() > BOUNDARY_CENTS => {
                    Some(verdict(t) == verdict(e))
                }
                _ => None,
            };
            StrumString {
                string: target.name,
                truth_cents,
                estimate_cents,
                error_cents,
                agree,
            }
        })
        .collect();

    Some(StrumResult {
        onset: event.onset,
        window_seconds: len as f32 / rate,
        strings,
    })
}

pub fn median(mut v: Vec<f32>) -> Option<f32> {
    if v.is_empty() {
        return None;
    }
    v.sort_by(f32::total_cmp);
    let m = v.len() / 2;
    Some(if v.len() % 2 == 1 {
        v[m]
    } else {
        0.5 * (v[m - 1] + v[m])
    })
}
