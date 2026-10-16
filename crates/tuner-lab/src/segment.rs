//! Splitting a take into events (single plucks and strums).

use crate::audio::Audio;
use serde::Serialize;
use tuner_core::poly::{PolyConfig, clean_partials, estimate_comb};
use tuner_core::synth::highpass;
use tuner_core::tuning::Instrument;
use tuner_core::{PitchConfig, PitchDetector};

/// Envelope resolution in seconds.
const HOP_SECONDS: f32 = 0.01;
/// A rise of at least this many dB within `RISE_WINDOW` hops is an onset...
const RISE_DB: f32 = 8.0;
const RISE_WINDOW: usize = 10;
/// ...if the envelope is also this far above the take's background...
const ABOVE_BACKGROUND_DB: f32 = 15.0;
/// ...and this far above its level 100–300 ms earlier, so a decaying chord
/// whose beating dips and recovers is not mistaken for a new strum.
const ABOVE_EARLIER_DB: f32 = 3.0;
const EARLIER_HOPS: (usize, usize) = (10, 30);
/// Onsets this close together belong to one event (the strings of a strum).
const MERGE_SECONDS: f32 = 0.08;
/// Minimum time between events.
const MIN_GAP_SECONDS: f32 = 0.25;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Pluck,
    Strum,
}

#[derive(Debug, Clone, Serialize)]
pub struct Event {
    /// Onset time in seconds.
    pub onset: f32,
    /// End of the segment (next event or end of the take), in seconds.
    pub end: f32,
    pub kind: Kind,
    /// Strings with clean partials that clearly sound (see [`classify`]).
    pub sounding_strings: usize,
    /// Median pitch clarity shortly after the onset.
    pub clarity: f32,
}

/// Envelope in dB of the 60–1500 Hz band, one value per `HOP_SECONDS`.
fn envelope(audio: &Audio) -> Vec<f32> {
    let mut band = audio.samples.clone();
    highpass(&mut band, 60.0, audio.sample_rate);
    // Two one-pole low-passes at ~1.5 kHz.
    let a = (-std::f32::consts::TAU * 1500.0 / audio.sample_rate).exp();
    for _ in 0..2 {
        let mut y = 0.0;
        for s in &mut band {
            y = (1.0 - a) * *s + a * y;
            *s = y;
        }
    }
    let hop = (HOP_SECONDS * audio.sample_rate) as usize;
    band.chunks(hop)
        .map(|c| {
            let rms = (c.iter().map(|v| v * v).sum::<f32>() / c.len() as f32).sqrt();
            20.0 * rms.max(1e-7).log10()
        })
        .collect()
}

/// Onset times (seconds) of events in the take.
pub fn onsets(audio: &Audio) -> Vec<f32> {
    let env = envelope(audio);
    if env.is_empty() {
        return vec![];
    }
    let mut sorted = env.clone();
    sorted.sort_by(f32::total_cmp);
    let background = sorted[sorted.len() / 10];

    let mut raw = Vec::new();
    for i in RISE_WINDOW..env.len() {
        let recent_min = env[i - RISE_WINDOW..i]
            .iter()
            .copied()
            .fold(f32::MAX, f32::min);
        let earlier_max = env[i.saturating_sub(EARLIER_HOPS.1)..i.saturating_sub(EARLIER_HOPS.0)]
            .iter()
            .copied()
            .fold(f32::MIN, f32::max);
        if env[i] - recent_min >= RISE_DB
            && env[i] >= background + ABOVE_BACKGROUND_DB
            && env[i] >= earlier_max + ABOVE_EARLIER_DB
        {
            raw.push(i as f32 * HOP_SECONDS);
        }
    }
    // Merge runs of onsets (a strum's strings, and consecutive hops of one rise).
    let mut events: Vec<f32> = Vec::new();
    let mut last_raw = f32::MIN;
    for t in raw {
        let continues = t - last_raw <= MERGE_SECONDS;
        last_raw = t;
        if continues {
            continue;
        }
        if events.last().is_none_or(|&e| t - e >= MIN_GAP_SECONDS) {
            events.push(t);
        }
    }
    events
}

/// Segments a take and classifies each event.
///
/// A single pluck makes at most one of the strings with uncolliding
/// ("clean") partials sound; even B3 and E4, whose partials all lie on E2's
/// series, make none. A strum makes most of them sound. Pitch clarity (a
/// pluck is clearly periodic) breaks ties.
pub fn segment(audio: &Audio, instrument: Instrument) -> Vec<Event> {
    let onsets = onsets(audio);
    let duration = audio.seconds();
    onsets
        .iter()
        .enumerate()
        .map(|(i, &onset)| {
            let end = onsets.get(i + 1).copied().unwrap_or(duration);
            let (sounding_strings, clarity) = classify(audio, instrument, onset, end);
            let kind = match sounding_strings {
                0 | 1 => Kind::Pluck,
                2 if clarity >= 0.85 => Kind::Pluck,
                _ => Kind::Strum,
            };
            Event {
                onset,
                end,
                kind,
                sounding_strings,
                clarity,
            }
        })
        .collect()
}

/// Level (relative to the strongest partial) above which a string counts as
/// sounding.
const SOUNDING_DB: f32 = -20.0;

fn classify(audio: &Audio, instrument: Instrument, onset: f32, end: f32) -> (usize, f32) {
    let rate = audio.sample_rate;
    let start = ((onset + 0.12) * rate) as usize;
    let available = ((end * rate) as usize)
        .min(audio.samples.len())
        .saturating_sub(start);
    let len = [32_768usize, 16_384, 8192]
        .into_iter()
        .find(|&n| n <= available)
        .unwrap_or(0);

    let config = PolyConfig::new(rate, instrument);
    let clean = clean_partials(instrument.strings(), &config);
    let sounding = if len > 0 {
        estimate_comb(&audio.samples[start..start + len], instrument, &config)
            .iter()
            .zip(&clean)
            .filter(|(e, c)| c.iter().any(|&x| x) && e.is_some_and(|e| e.level_db >= SOUNDING_DB))
            .count()
    } else {
        0
    };

    let mut detector = PitchDetector::new(PitchConfig::permissive(rate));
    let frame = detector.config().frame_size;
    let mut clarities: Vec<f32> = (0..6)
        .filter_map(|k| {
            let s = ((onset + 0.15 + 0.05 * k as f32) * rate) as usize;
            let frame_samples = audio.samples.get(s..s + frame)?;
            detector.detect(frame_samples).map(|e| e.clarity)
        })
        .collect();
    clarities.sort_by(f32::total_cmp);
    let clarity = clarities.get(clarities.len() / 2).copied().unwrap_or(0.0);
    (sounding, clarity)
}
