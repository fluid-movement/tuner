//! Streaming strum analysis: feed audio as it arrives; each detected strum
//! produces reports as more audio after it is heard.
//!
//! Onsets come from the 60–1500 Hz envelope in 10 ms hops: a rise of at least
//! 8 dB within 100 ms, at least 15 dB above the background and 3 dB above the
//! level 100–300 ms earlier (so a decaying chord's beating doesn't count);
//! onsets within 80 ms merge (a strum's strings). Analysis windows start
//! 120 ms after the onset (≥ 80 ms after the last string).
//!
//! Reports are staged so the first one comes quickly: a short window first,
//! then longer ones (see `stages`). Short windows estimate pitch well but
//! can't yet tell a muted string from a quiet one, so until the last stage a
//! string that doesn't clearly sound is reported as pending (no reading, no
//! verdict). In [`Analysis::Tuning`] the readings then keep updating, on
//! the latest window of the last stage's length, while the strum rings, so
//! turning a peg shows. [`Analysis::Chord`] reports the chord instead
//! ([`chord::detect_chord`](crate::chord::detect_chord)).
//!
//! Not for the audio thread: an analysis takes several milliseconds.

use crate::chord::{ChordReading, detect_chord};
use crate::fft::Fft;
use crate::poly::{PRESENT_PROMINENCE_DB, PolyConfig, estimate_strum};
use crate::tuning::Instrument;
use num_complex::Complex32;

const HOP_SECONDS: f32 = 0.01;
const RISE_DB: f32 = 8.0;
const RISE_HOPS: usize = 10;
const ABOVE_BACKGROUND_DB: f32 = 15.0;
/// No onset below this in-band level (dBFS): a strum near a phone or laptop
/// microphone is around -40 to -15; faint room sounds are below.
const MIN_ONSET_DB: f32 = -55.0;
const ABOVE_EARLIER_DB: f32 = 3.0;
const EARLIER_HOPS: (usize, usize) = (10, 30);
const MERGE_SECONDS: f32 = 0.08;
const DELAY_SECONDS: f32 = 0.12;
/// Time constant with which the background rises (it falls immediately).
const BACKGROUND_RISE_SECONDS: f32 = 2.0;
/// Spectral flux: each hop, the spectrum of the last `FLUX_FRAME` samples
/// is compared with the one `FLUX_LAG` hops earlier; the flux is the mean
/// rise (dB, clamped to a range below the loudest bin) over 80 Hz–4 kHz.
/// Re-strumming over ringing strings barely raises the level but brings
/// fresh partials, which the envelope rule misses.
const FLUX_FRAME: usize = 2048;
const FLUX_LAG: usize = 3;
const FLUX_RANGE_DB: f32 = 50.0;
/// A flux peak is an onset if it reaches this...
const FLUX_MIN_DB: f32 = 4.0;
/// ...and this much above the median flux of the last `FLUX_MEDIAN_HOPS`.
const FLUX_MARGIN_DB: f32 = 1.5;
const FLUX_MEDIAN_HOPS: usize = 50;
/// Sound must be this far above the quiet floor for a flux onset...
const FLUX_ABOVE_QUIET_DB: f32 = 10.0;
/// ...a background that falls at once but rises this slowly, so continuous
/// playing doesn't lift it (the onset background follows the music within
/// seconds).
const QUIET_RISE_SECONDS: f32 = 30.0;
/// Live updates (tuning) come this often after the last stage...
const LIVE_HOP_SECONDS: f32 = 0.1;
/// ...while the in-band level stays this far above the background before
/// the strum...
const LIVE_MARGIN_DB: f32 = 10.0;
/// ...for at most this long after the onset.
const LIVE_MAX_SECONDS: f32 = 6.0;

/// What a strum is analysed for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Analysis {
    /// Per-string tuning offsets (standard tuning).
    Tuning = 0,
    /// The chord played: fret per string and name.
    Chord = 1,
}

/// One string's result for a strum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrumString {
    /// Estimated frequency in Hz, if any.
    pub frequency: Option<f32>,
    /// Offset from the string's target in cents.
    pub cents: Option<f32>,
    /// Whether the string was played: `Some(false)` means not heard; `None`
    /// means it can't be judged (guitar B3/E4) or, with no reading, not yet.
    pub present: Option<bool>,
}

const PENDING: StrumString = StrumString {
    frequency: None,
    cents: None,
    present: None,
};

#[derive(Debug, Clone, PartialEq)]
pub struct StrumReport {
    /// Onset time in seconds since the listener started.
    pub onset_seconds: f64,
    /// Whether this is the last stage or a live update after it (string
    /// presence is final); earlier stages may still change.
    pub settled: bool,
    /// In string order of the instrument ([`Analysis::Tuning`]; empty for
    /// [`Analysis::Chord`]).
    pub strings: Vec<StrumString>,
    /// The chord ([`Analysis::Chord`]; `None` if none was recognised).
    pub chord: Option<ChordReading>,
}

/// A strum being reported on.
#[derive(Debug, Clone)]
struct Current {
    onset: u64,
    /// Index of the next stage to analyse.
    stage: usize,
    /// Absolute sample index of the next live update.
    next_live: u64,
    /// Last report's strings (tuning), merged into by live updates.
    strings: Vec<StrumString>,
    /// Background level at the onset (the running background rises towards
    /// a ringing strum).
    background_db: f32,
    /// Chord analysis: the first stage's notes (bit per MIDI note), if it
    /// found a chord; the settled chord must agree with it.
    first_notes: Option<u128>,
}

#[derive(Debug, Clone)]
pub struct StrumListener {
    instrument: Instrument,
    analysis: Analysis,
    poly: PolyConfig,
    sample_rate: f32,
    delay: usize,
    /// Audio history (ring buffer) and the absolute index of the next sample.
    history: Vec<f32>,
    written: u64,
    // Envelope band filter state: 2nd-order high-pass at 60 Hz, then two
    // one-pole low-passes at 1.5 kHz.
    hp: Biquad,
    lp: [f32; 2],
    lp_coeff: f32,
    hop: usize,
    hop_power: f64,
    hop_count: usize,
    /// Recent envelope values in dB, newest last.
    envelope: Vec<f32>,
    background: Option<f32>,
    /// Slow noise floor for flux onsets (see `QUIET_RISE_SECONDS`).
    quiet: Option<f32>,
    /// Absolute sample index of the last raw onset, for merging.
    last_raw: Option<u64>,
    flux: Flux,
    /// Settled report of a strum cut short by the next one, returned by the
    /// next `push`.
    interrupted: Option<StrumReport>,
    current: Option<Current>,
}

/// Window lengths (samples at ≥ 40 kHz) of the staged reports. Ukulele
/// strings are estimated as well from 85 ms as from 0.7 s; guitar needs
/// about 0.34 s (`examples/poly_eval.rs` with `POLY_FRAME`), and string
/// presence needs 0.34 s on ukulele and 0.68 s on guitar
/// (`examples/presence_eval.rs`). Chords need ~0.17 s to separate close
/// partials (`examples/chord_eval.rs`).
fn stages(instrument: Instrument, analysis: Analysis, sample_rate: f32) -> Vec<usize> {
    let windows: &[usize] = match (analysis, instrument) {
        (Analysis::Chord, _) => &[8192, 16_384],
        (Analysis::Tuning, Instrument::Ukulele) => &[4096, 8192, 16_384],
        (Analysis::Tuning, Instrument::Guitar) => &[8192, 16_384, 32_768],
    };
    // Roughly the same durations at low sample rates.
    let scale = if sample_rate >= 40_000.0 { 1 } else { 2 };
    windows.iter().map(|w| w / scale).collect()
}

impl StrumListener {
    pub fn new(sample_rate: f32, instrument: Instrument) -> Self {
        let delay = (DELAY_SECONDS * sample_rate) as usize;
        // Room for the longest window of any analysis.
        let longest = stages(Instrument::Guitar, Analysis::Tuning, sample_rate)
            .into_iter()
            .max()
            .unwrap_or(0);
        Self {
            instrument,
            analysis: Analysis::Tuning,
            poly: PolyConfig::new(sample_rate, instrument),
            sample_rate,
            delay,
            history: vec![0.0; (delay + longest).next_power_of_two() * 2],
            written: 0,
            hp: Biquad::highpass(60.0, sample_rate),
            lp: [0.0; 2],
            lp_coeff: (-std::f32::consts::TAU * 1500.0 / sample_rate).exp(),
            hop: ((HOP_SECONDS * sample_rate) as usize).max(1),
            hop_power: 0.0,
            hop_count: 0,
            envelope: Vec::with_capacity(EARLIER_HOPS.1 + 1),
            background: None,
            quiet: None,
            last_raw: None,
            flux: Flux::new(sample_rate),
            interrupted: None,
            current: None,
        }
    }

    pub fn instrument(&self) -> Instrument {
        self.instrument
    }

    pub fn analysis(&self) -> Analysis {
        self.analysis
    }

    /// Switches instrument; a strum being reported on is dropped.
    pub fn set_instrument(&mut self, instrument: Instrument) {
        self.instrument = instrument;
        self.poly = PolyConfig::new(self.sample_rate, instrument);
        self.current = None;
    }

    /// Switches what strums are analysed for; a strum being reported on is
    /// dropped.
    pub fn set_analysis(&mut self, analysis: Analysis) {
        self.analysis = analysis;
        self.current = None;
    }

    fn stages(&self) -> Vec<usize> {
        stages(self.instrument, self.analysis, self.sample_rate)
    }

    /// Seconds of audio after an onset before its first report is ready.
    pub fn latency_seconds(&self) -> f32 {
        (self.delay + self.stages()[0]) as f32 / self.sample_rate
    }

    /// Seconds of audio after an onset before its report is settled.
    pub fn settled_seconds(&self) -> f32 {
        (self.delay + *self.stages().last().expect("stages")) as f32 / self.sample_rate
    }

    /// Adds audio. Returns a report when a stage's analysis window has just
    /// completed, or a live update is due (at most one per call; call with
    /// small chunks).
    pub fn push(&mut self, samples: &[f32]) -> Option<StrumReport> {
        let mut report = None;
        let cap = self.history.len();
        for &x in samples {
            self.history[(self.written % cap as u64) as usize] = x;
            self.written += 1;

            let mut y = self.hp.process(x);
            for state in &mut self.lp {
                *state = (1.0 - self.lp_coeff) * y + self.lp_coeff * *state;
                y = *state;
            }
            self.hop_power += f64::from(y) * f64::from(y);
            self.hop_count += 1;
            if self.hop_count == self.hop {
                let db = 10.0 * (self.hop_power / self.hop as f64).max(1e-14).log10() as f32;
                self.hop_power = 0.0;
                self.hop_count = 0;
                self.on_envelope(db);
            }
            if report.is_none() {
                report = self.interrupted.take().or_else(|| self.due());
            }
        }
        report
    }

    /// Runs the analysis that is due at the current sample, if any.
    fn due(&mut self) -> Option<StrumReport> {
        let stages = self.stages();
        let written = self.written;
        let current = self.current.as_mut()?;
        let start = current.onset + self.delay as u64;
        if let Some(&window) = stages.get(current.stage) {
            if written < start + window as u64 {
                return None;
            }
            current.stage += 1;
            let settled = current.stage == stages.len();
            current.next_live = written + (LIVE_HOP_SECONDS * self.sample_rate) as u64;
            return Some(self.analyse(start, window, settled, false));
        }
        // Live updates of a settled tuning report while the strum rings.
        let sounding = self
            .envelope
            .last()
            .is_some_and(|&db| db >= current.background_db + LIVE_MARGIN_DB);
        let expired = written > current.onset + (LIVE_MAX_SECONDS * self.sample_rate) as u64;
        if self.analysis != Analysis::Tuning || !sounding || expired {
            self.current = None;
            return None;
        }
        if written < current.next_live {
            return None;
        }
        current.next_live = written + (LIVE_HOP_SECONDS * self.sample_rate) as u64;
        let window = *stages.last().expect("stages") as u64;
        Some(self.analyse(written - window, window as usize, true, true))
    }

    fn analyse(&mut self, start: u64, window: usize, settled: bool, live: bool) -> StrumReport {
        let cap = self.history.len() as u64;
        let frame: Vec<f32> = (start..start + window as u64)
            .map(|i| self.history[(i % cap) as usize])
            .collect();
        let current = self.current.as_mut().expect("a strum is being reported on");
        let onset_seconds = current.onset as f64 / f64::from(self.sample_rate);
        if self.analysis == Analysis::Chord {
            let chord = detect_chord(
                &frame,
                self.sample_rate,
                self.instrument,
                self.poly.inharmonicity,
            );
            let notes = chord.as_ref().map(|c| {
                c.midi
                    .iter()
                    .flatten()
                    .fold(0u128, |m, &n| m | 1 << n.clamp(0, 127))
            });
            let chord = if current.stage <= 1 && !settled {
                // First stage: remember what it heard.
                current.first_notes = notes;
                chord
            } else if current.stage > 1 || settled && current.first_notes.is_some() {
                // A strum keeps its notes as it rings; voices and whistles
                // glide. The settled chord must share at least half its
                // notes with the first stage's (which must have found one).
                let agree = match (&current.first_notes, &notes) {
                    (Some(first), Some(now)) => 2 * (now & first).count_ones() >= now.count_ones(),
                    _ => false,
                };
                chord.filter(|_| agree)
            } else {
                chord
            };
            return StrumReport {
                onset_seconds,
                settled,
                strings: vec![],
                chord,
            };
        }
        let estimates = estimate_strum(&frame, self.instrument, &self.poly);
        let fresh: Vec<StrumString> = estimates
            .iter()
            .map(|e| match e {
                Some(e) => StrumString {
                    frequency: Some(e.frequency),
                    cents: Some(e.cents),
                    present: e.present,
                },
                None => StrumString {
                    present: Some(false),
                    ..PENDING
                },
            })
            .collect();
        let strings: Vec<StrumString> = if live {
            // Presence was settled; keep a string's last reading once it
            // fades (strings decay at different rates).
            current
                .strings
                .iter()
                .zip(&fresh)
                .map(|(old, new)| match old.present {
                    Some(false) => *old,
                    _ if new.present == Some(false) || new.cents.is_none() => *old,
                    _ => StrumString {
                        present: old.present,
                        ..*new
                    },
                })
                .collect()
        } else if settled {
            fresh
        } else {
            // Too early to call a quiet string muted: no verdict yet. A
            // string that clearly sounds is reported (guitar B3/E4 can't be
            // judged, so they always are).
            estimates
                .iter()
                .zip(fresh)
                .map(|(e, s)| match e {
                    Some(e) if e.prominence_db.is_none_or(|p| p >= PRESENT_PROMINENCE_DB) => s,
                    _ => PENDING,
                })
                .collect()
        };
        current.strings = strings.clone();
        StrumReport {
            onset_seconds,
            settled,
            strings,
            chord: None,
        }
    }

    /// If the strum being reported on hasn't settled, analyses it now on the
    /// audio between its window start and `until` (the next onset), if that
    /// is at least the first stage's window, and keeps the settled report.
    fn settle_interrupted(&mut self, until: u64) {
        let stages = self.stages();
        let Some(current) = &self.current else { return };
        if current.stage >= stages.len() {
            return;
        }
        let start = current.onset + self.delay as u64;
        let available = until.saturating_sub(start) as usize;
        let longest = *stages.last().expect("stages");
        // A little short of the first stage still beats nothing: strumming
        // every 0.3 s leaves ~0.17 s (chord_eval: 91% at 7/8 of 8192).
        if available < stages[0] * 7 / 8 {
            return;
        }
        let window = available.min(longest);
        self.interrupted = Some(self.analyse(start, window, true, false));
    }

    /// Updates the spectral flux and says whether this hop ends a flux peak
    /// that marks an onset (a hop late: the peak is known once it falls).
    fn flux_onset(&mut self, db: f32, quiet: f32) -> bool {
        let cap = self.history.len() as u64;
        if self.written < FLUX_FRAME as u64 {
            return false;
        }
        let start = self.written - FLUX_FRAME as u64;
        let frame = (start..self.written).map(|i| self.history[(i % cap) as usize]);
        let value = self.flux.push(frame);
        let f = &self.flux;
        let n = f.values.len();
        if n < 3 {
            return false;
        }
        // The previous hop was a local maximum above the threshold.
        let peak = f.values[n - 2];
        let mut recent: Vec<f32> = f.values[..n - 2].to_vec();
        recent.sort_by(f32::total_cmp);
        let median = recent.get(recent.len() / 2).copied().unwrap_or(0.0);
        peak >= FLUX_MIN_DB
            && peak >= median + FLUX_MARGIN_DB
            && peak >= f.values[n - 3]
            && peak > value
            && db >= quiet + FLUX_ABOVE_QUIET_DB
    }

    fn on_envelope(&mut self, db: f32) {
        let background = self.background.get_or_insert(db);
        if db < *background {
            *background = db;
        } else {
            let a = 1.0 - (-HOP_SECONDS / BACKGROUND_RISE_SECONDS).exp();
            *background += a * (db - *background);
        }
        let background = *background;

        let env = &self.envelope;
        let n = env.len();
        let onset = n >= EARLIER_HOPS.1 && {
            let recent_min = env[n - RISE_HOPS..]
                .iter()
                .copied()
                .fold(f32::MAX, f32::min);
            let earlier_max = env[n - EARLIER_HOPS.1..n - EARLIER_HOPS.0]
                .iter()
                .copied()
                .fold(f32::MIN, f32::max);
            db - recent_min >= RISE_DB
                && db >= background + ABOVE_BACKGROUND_DB
                && db >= earlier_max + ABOVE_EARLIER_DB
        };
        let quiet = self.quiet.get_or_insert(db);
        if db < *quiet {
            *quiet = db;
        } else {
            *quiet += (1.0 - (-HOP_SECONDS / QUIET_RISE_SECONDS).exp()) * (db - *quiet);
        }
        let quiet = *quiet;
        let flux_onset = self.flux_onset(db, quiet);
        if (onset || flux_onset) && db >= MIN_ONSET_DB {
            // The rise ended within this hop; place the onset at its start.
            // A flux peak is seen a hop late, about half a frame after the
            // attack entered the frame.
            let back = if onset {
                self.hop
            } else {
                2 * self.hop + FLUX_FRAME / 2
            };
            let at = self.written.saturating_sub(back as u64);
            let merge = (MERGE_SECONDS * self.sample_rate) as u64;
            let continues = self
                .last_raw
                .is_some_and(|last| at.saturating_sub(last) <= merge);
            self.last_raw = Some(self.last_raw.map_or(at, |last| last.max(at)));
            if !continues {
                // A strum cut short by this one settles on what it got.
                self.settle_interrupted(at);
                // A new strum replaces the one being reported on.
                self.current = Some(Current {
                    onset: at,
                    stage: 0,
                    next_live: 0,
                    strings: vec![],
                    background_db: background,
                    first_notes: None,
                });
            }
        }

        if self.envelope.len() > EARLIER_HOPS.1 {
            self.envelope.remove(0);
        }
        self.envelope.push(db);
    }
}

/// Spectral flux state (see `FLUX_FRAME`).
#[derive(Debug, Clone)]
struct Flux {
    fft: Fft,
    window: Vec<f32>,
    buf: Vec<Complex32>,
    /// Bin range 80 Hz–4 kHz.
    bins: (usize, usize),
    /// The last `FLUX_LAG + 1` spectra (dB over `bins`), oldest first.
    spectra: Vec<Vec<f32>>,
    /// Recent flux values, newest last.
    values: Vec<f32>,
}

impl Flux {
    fn new(sample_rate: f32) -> Self {
        let bin_hz = sample_rate / FLUX_FRAME as f32;
        Self {
            fft: Fft::new(FLUX_FRAME),
            window: (0..FLUX_FRAME)
                .map(|i| {
                    let t = std::f32::consts::TAU * i as f32 / FLUX_FRAME as f32;
                    0.5 - 0.5 * t.cos()
                })
                .collect(),
            buf: vec![Complex32::new(0.0, 0.0); FLUX_FRAME],
            bins: (
                (80.0 / bin_hz).ceil() as usize,
                ((4000.0 / bin_hz) as usize).min(FLUX_FRAME / 2 - 1),
            ),
            spectra: Vec::with_capacity(FLUX_LAG + 1),
            values: Vec::with_capacity(FLUX_MEDIAN_HOPS + 3),
        }
    }

    /// Adds the newest frame; returns its flux (0 until enough history).
    fn push(&mut self, frame: impl Iterator<Item = f32>) -> f32 {
        for ((b, x), w) in self.buf.iter_mut().zip(frame).zip(&self.window) {
            *b = Complex32::new(x * w, 0.0);
        }
        self.fft.forward(&mut self.buf);
        let (lo, hi) = self.bins;
        let mut spectrum = if self.spectra.len() > FLUX_LAG {
            self.spectra.remove(0)
        } else {
            vec![0.0; hi - lo + 1]
        };
        for (d, c) in spectrum.iter_mut().zip(&self.buf[lo..=hi]) {
            *d = 10.0 * (c.norm_sqr() + 1e-12).log10();
        }
        let value = match self.spectra.first() {
            Some(old) if self.spectra.len() == FLUX_LAG => {
                let top = spectrum
                    .iter()
                    .chain(old.iter())
                    .copied()
                    .fold(f32::MIN, f32::max);
                let floor = top - FLUX_RANGE_DB;
                spectrum
                    .iter()
                    .zip(old)
                    .map(|(&new, &old)| (new.max(floor) - old.max(floor)).max(0.0))
                    .sum::<f32>()
                    / spectrum.len() as f32
            }
            _ => 0.0,
        };
        self.spectra.push(spectrum);
        if self.values.len() > FLUX_MEDIAN_HOPS + 2 {
            self.values.remove(0);
        }
        self.values.push(value);
        value
    }
}

#[derive(Debug, Clone, Copy)]
struct Biquad {
    b: [f32; 3],
    a: [f32; 2],
    x: [f32; 2],
    y: [f32; 2],
}

impl Biquad {
    fn highpass(cutoff: f32, sample_rate: f32) -> Self {
        let w = std::f32::consts::TAU * cutoff / sample_rate;
        let (cos, alpha) = (w.cos(), w.sin() / std::f32::consts::SQRT_2);
        let a0 = 1.0 + alpha;
        Self {
            b: [
                (1.0 + cos) / 2.0 / a0,
                -(1.0 + cos) / a0,
                (1.0 + cos) / 2.0 / a0,
            ],
            a: [-2.0 * cos / a0, (1.0 - alpha) / a0],
            x: [0.0; 2],
            y: [0.0; 2],
        }
    }

    fn process(&mut self, x: f32) -> f32 {
        let y = self.b[0] * x + self.b[1] * self.x[0] + self.b[2] * self.x[1]
            - self.a[0] * self.y[0]
            - self.a[1] * self.y[1];
        self.x = [x, self.x[0]];
        self.y = [y, self.y[0]];
        y
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synth::{PluckedString, Rng, highpass, render};

    const RATE: f32 = 48_000.0;

    /// Strums 3 s apart with the given detunes; `muted[k]` mutes a string in
    /// strum k. Returns the signal and the strum onsets.
    fn strums(
        instrument: Instrument,
        detunes: &[f32],
        muted: &[Option<usize>],
        seed: u64,
    ) -> (Vec<f32>, Vec<f32>) {
        let mut rng = Rng::new(seed);
        let mut notes = Vec::new();
        let mut onsets = Vec::new();
        for (k, mute) in muted.iter().enumerate() {
            let t0 = 0.8 + 3.0 * k as f32;
            onsets.push(t0);
            let mut onset = t0;
            for (i, (t, c)) in instrument.strings().iter().zip(detunes).enumerate() {
                let f = t.frequency * 2f32.powf(c / 1200.0);
                if *mute != Some(i) {
                    notes.push(PluckedString {
                        frequency: f,
                        level: 0.06 * 10f32.powf(rng.uniform(-8.0, 0.0) / 20.0),
                        onset,
                        inharmonicity: if instrument == Instrument::Guitar {
                            1e-4
                        } else {
                            2e-5
                        },
                        decay: rng.uniform(0.9, 1.4) * (110.0 / f).powf(0.4),
                        rolloff: rng.uniform(0.7, 1.1),
                        pluck_position: rng.uniform(0.12, 0.22),
                        polarization_cents: rng.uniform(0.5, 2.0),
                    });
                }
                onset += rng.uniform(0.008, 0.03);
            }
        }
        let seconds = 0.8 + 3.0 * muted.len() as f32 + 0.5;
        let mut x = render(&notes, RATE, seconds, 0.002, &mut rng);
        highpass(&mut x, 100.0, RATE);
        (x, onsets)
    }

    fn listen_all(instrument: Instrument, analysis: Analysis, x: &[f32]) -> Vec<StrumReport> {
        let mut listener = StrumListener::new(RATE, instrument);
        listener.set_analysis(analysis);
        x.chunks(1024).filter_map(|c| listener.push(c)).collect()
    }

    /// The first settled report of each strum.
    fn listen(instrument: Instrument, x: &[f32]) -> Vec<StrumReport> {
        let mut out: Vec<StrumReport> = vec![];
        for r in listen_all(instrument, Analysis::Tuning, x) {
            if r.settled
                && out
                    .last()
                    .is_none_or(|l| l.onset_seconds != r.onset_seconds)
            {
                out.push(r);
            }
        }
        out
    }

    #[test]
    fn reports_each_strum_with_verdicts_and_presence() {
        let detunes = [1.0, -2.0, 0.0, 3.0, 9.0, -12.0];
        // Muting damps a string; the synthesizer can't damp a ringing note, so
        // mute in the first strum (nothing earlier still rings).
        let muted = [Some(2), None, None];
        let (x, onsets) = strums(Instrument::Guitar, &detunes, &muted, 1);
        let reports = listen(Instrument::Guitar, &x);
        assert_eq!(reports.len(), onsets.len());
        for ((r, &t), mute) in reports.iter().zip(&onsets).zip(&muted) {
            assert!(
                (r.onset_seconds as f32 - t).abs() < 0.05,
                "onset {} vs {t}",
                r.onset_seconds
            );
            for (i, (s, &c)) in r.strings.iter().zip(&detunes).enumerate() {
                if *mute == Some(i) {
                    assert_eq!(s.present, Some(false), "muted string {i}");
                    continue;
                }
                assert_ne!(s.present, Some(false), "string {i} played");
                let cents = s.cents.expect("reading");
                assert!(
                    (cents - c).abs() < 3.0,
                    "string {i}: {cents:+.1} vs {c:+.1}"
                );
            }
        }
    }

    #[test]
    fn ukulele_strums() {
        let detunes = [-15.0, 4.0, 0.0, 7.0];
        let muted = [Some(3), None];
        let (x, _) = strums(Instrument::Ukulele, &detunes, &muted, 2);
        let reports = listen(Instrument::Ukulele, &x);
        assert_eq!(reports.len(), 2);
        assert_eq!(reports[0].strings[3].present, Some(false));
        for (s, &c) in reports[1].strings.iter().zip(&detunes) {
            assert_eq!(s.present, Some(true));
            assert!((s.cents.unwrap() - c).abs() < 3.0);
        }
    }

    /// The browser tests' progression: ideal strings (10 harmonics, no
    /// noise), one chord every 1.5 s, each faded out just before the next.
    fn progression(frets: &[[u32; 4]], rounds: usize, lead: usize, rate: f32) -> Vec<f32> {
        let opens = [392.0f32, 261.63, 329.63, 440.0];
        let step = (1.5 * rate) as usize;
        let mut x = vec![0.0f32; lead + step * frets.len() * rounds];
        for k in 0..frets.len() * rounds {
            let begin = lead + k * step;
            for (s, &f) in frets[k % frets.len()].iter().enumerate() {
                let freq = opens[s] * 2f32.powf(f as f32 / 12.0);
                let start = begin + ((0.01 + 0.015 * s as f32) * rate) as usize;
                let end = begin + step;
                let tau_string = 1.2 * (110.0 / freq).powf(0.4);
                for h in 1..=10 {
                    let amp = 0.05 / (h as f32).powf(0.9);
                    let tau = tau_string / (1.0 + 0.35 * (h - 1) as f32);
                    for (i, v) in x.iter_mut().enumerate().take(end).skip(start) {
                        let t = (i - start) as f32 / rate;
                        let fade = ((end - i) as f32 / (0.01 * rate)).min(1.0);
                        *v += fade
                            * amp
                            * (-t / tau).exp()
                            * (std::f32::consts::TAU * freq * h as f32 * t).sin();
                    }
                }
            }
        }
        x
    }

    #[test]
    fn ringing_chords_do_not_start_new_strums() {
        // C Am F G: one onset per strum, wherever the hops fall.
        let frets = [[0, 0, 0, 3], [2, 0, 0, 0], [2, 0, 1, 0], [0, 2, 3, 2]];
        let mut extra = vec![];
        for (rate, lead) in [44_100.0, 48_000.0]
            .into_iter()
            .flat_map(|r| (0..1024).step_by(256).map(move |l| (r, l)))
        {
            let x = progression(&frets, 2, lead, rate);
            let mut listener = StrumListener::new(rate, Instrument::Ukulele);
            listener.set_analysis(Analysis::Chord);
            let mut onsets: Vec<f64> = vec![];
            for r in x.chunks(1024).filter_map(|c| listener.push(c)) {
                if r.settled && onsets.last() != Some(&r.onset_seconds) {
                    onsets.push(r.onset_seconds);
                }
            }
            if onsets.len() != 8 {
                extra.push((rate, lead, onsets));
            }
        }
        assert!(extra.is_empty(), "{extra:?}");
    }

    #[test]
    fn noise_and_single_decays_do_not_report() {
        let mut rng = Rng::new(9);
        let x = render(&[], RATE, 5.0, 0.002, &mut rng);
        assert!(listen(Instrument::Guitar, &x).is_empty());
    }

    #[test]
    fn first_report_comes_quickly_then_settles() {
        for (instrument, max_first) in [(Instrument::Ukulele, 0.25), (Instrument::Guitar, 0.33)] {
            let n = instrument.strings().len();
            let (x, onsets) = strums(instrument, &vec![0.0; n], &[None], 3);
            let mut listener = StrumListener::new(RATE, instrument);
            let mut times = vec![];
            for (k, c) in x.chunks(1024).enumerate() {
                if let Some(r) = listener.push(c) {
                    times.push((((k + 1) * 1024) as f32 / RATE - onsets[0], r.settled));
                }
            }
            let (first, settled) = times[0];
            assert!(!settled);
            assert!(
                first <= max_first,
                "{instrument:?} first report after {first}"
            );
            assert!((first - listener.latency_seconds()).abs() < 0.05);
            let settle = times.iter().find(|t| t.1).expect("settled").0;
            assert!((settle - listener.settled_seconds()).abs() < 0.05);
        }
    }

    #[test]
    fn early_reports_never_call_a_played_string_muted() {
        let detunes = [-15.0, 4.0, 0.0, 7.0];
        let (x, _) = strums(Instrument::Ukulele, &detunes, &[None, None], 4);
        for r in listen_all(Instrument::Ukulele, Analysis::Tuning, &x) {
            for (s, &c) in r.strings.iter().zip(&detunes) {
                assert_ne!(s.present, Some(false));
                if let Some(cents) = s.cents {
                    // Late live windows see decayed, beating strings.
                    assert!(
                        (cents - c).abs() < 4.0,
                        "{cents} vs {c} settled {}",
                        r.settled
                    );
                }
            }
        }
    }

    #[test]
    fn live_updates_follow_a_peg_being_turned() {
        // One ukulele A4 plucked, then its pitch glides up 20 cents while it
        // rings (rendered as a chirp; the synthesizer has no glide).
        let mut rng = Rng::new(5);
        let mut x = render(&[], RATE, 3.0, 0.002, &mut rng);
        let onset = (0.8 * RATE) as usize;
        let mut phase = 0.0f32;
        for (i, y) in x.iter_mut().enumerate().skip(onset) {
            let t = (i - onset) as f32 / RATE;
            let cents = if t < 0.6 {
                0.0
            } else {
                (20.0 * (t - 0.6) / 0.6).min(20.0)
            };
            let f = 440.0 * 2f32.powf(cents / 1200.0);
            phase = (phase + std::f32::consts::TAU * f / RATE) % std::f32::consts::TAU;
            let env = 0.2 * (-t / 2.0).exp();
            *y += env * (phase.sin() + 0.4 * (2.0 * phase).sin() + 0.2 * (3.0 * phase).sin());
        }
        let reports = listen_all(Instrument::Ukulele, Analysis::Tuning, &x);
        assert!(reports.len() > 10, "{} reports", reports.len());
        let first = reports.iter().find(|r| r.settled).unwrap().strings[3];
        assert!(first.cents.unwrap().abs() < 3.0);
        assert_eq!(reports[0].strings[0].present, None, "G4 pending at first");
        let settled = reports.iter().find(|r| r.settled).unwrap();
        assert_eq!(settled.strings[0].present, Some(false), "G4 not heard");
        let last = reports.last().unwrap().strings[3];
        assert!(
            (last.cents.unwrap() - 20.0).abs() < 3.0,
            "live reading {:?}",
            last.cents
        );
    }

    #[test]
    fn chord_analysis_reports_the_chord() {
        use crate::synth::chord_strum;
        let mut rng = Rng::new(6);
        let frets = [Some(2), Some(0), Some(1), Some(0)]; // F
        let mut strum = chord_strum(Instrument::Ukulele, &frets, 5.0, &mut rng);
        for s in &mut strum {
            s.onset += 0.8;
        }
        let x = render(&strum, RATE, 2.5, 0.002, &mut rng);
        let reports = listen_all(Instrument::Ukulele, Analysis::Chord, &x);
        assert_eq!(reports.len(), 2, "two stages, no live updates");
        assert!(reports[1].settled);
        let chord = reports[1].chord.as_ref().expect("chord");
        assert_eq!(chord.frets, frets);
        assert!(reports[1].strings.is_empty());
    }

    #[test]
    fn keeps_up_with_continuous_strumming() {
        // C, Am, F, G strummed every 0.35 s, each note ringing on (the
        // envelope barely rises between strums); every strum must settle.
        use crate::synth::chord_strum;
        let mut rng = Rng::new(8);
        let chords = [[0, 0, 0, 3], [2, 0, 0, 0], [2, 0, 1, 0], [0, 2, 3, 2]];
        let mut notes = vec![];
        let mut truth = vec![];
        for (k, chord) in chords.iter().cycle().take(12).enumerate() {
            let frets: Vec<Option<u8>> = chord.iter().map(|&f| Some(f)).collect();
            let t = 0.5 + 0.35 * k as f32;
            let mut strum = chord_strum(Instrument::Ukulele, &frets, 3.0, &mut rng);
            for s in &mut strum {
                s.onset += t;
            }
            notes.extend(strum);
            truth.push((t, frets));
        }
        let x = render(&notes, RATE, 5.5, 0.002, &mut rng);
        let settled: Vec<StrumReport> = listen_all(Instrument::Ukulele, Analysis::Chord, &x)
            .into_iter()
            .filter(|r| r.settled)
            .collect();
        let found = truth
            .iter()
            .filter(|(t, _)| {
                settled
                    .iter()
                    .any(|r| (r.onset_seconds as f32 - t).abs() < 0.06)
            })
            .count();
        assert!(found >= 11, "{found}/12 strums settled");
    }
}
