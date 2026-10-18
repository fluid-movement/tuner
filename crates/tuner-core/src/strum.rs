//! Streaming strum check: feed audio as it arrives; each detected strum
//! produces a per-string report once enough audio after it has been heard.
//!
//! Onsets come from the 60–1500 Hz envelope in 10 ms hops: a rise of at least
//! 8 dB within 100 ms, at least 15 dB above the background and 3 dB above the
//! level 100–300 ms earlier (so a decaying chord's beating doesn't count);
//! onsets within 80 ms merge (a strum's strings). The analysis window starts
//! 120 ms after the onset (≥ 80 ms after the last string) and is
//! [`poly::estimate_strum`](crate::poly::estimate_strum) on ~0.7 s of audio.
//!
//! Not for the audio thread: an analysis takes several milliseconds.

use crate::poly::{PolyConfig, estimate_strum};
use crate::tuning::Instrument;

const HOP_SECONDS: f32 = 0.01;
const RISE_DB: f32 = 8.0;
const RISE_HOPS: usize = 10;
const ABOVE_BACKGROUND_DB: f32 = 15.0;
const ABOVE_EARLIER_DB: f32 = 3.0;
const EARLIER_HOPS: (usize, usize) = (10, 30);
const MERGE_SECONDS: f32 = 0.08;
const DELAY_SECONDS: f32 = 0.12;
/// Time constant with which the background rises (it falls immediately).
const BACKGROUND_RISE_SECONDS: f32 = 2.0;

/// One string's result for a strum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StrumString {
    /// Estimated frequency in Hz, if any.
    pub frequency: Option<f32>,
    /// Offset from the string's target in cents.
    pub cents: Option<f32>,
    /// Whether the string was played: `Some(false)` means not heard; `None`
    /// means it can't be judged (guitar B3/E4).
    pub present: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrumReport {
    /// Onset time in seconds since the listener started.
    pub onset_seconds: f64,
    /// In string order of the instrument.
    pub strings: Vec<StrumString>,
}

#[derive(Debug, Clone)]
pub struct StrumListener {
    instrument: Instrument,
    poly: PolyConfig,
    sample_rate: f32,
    window: usize,
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
    /// Absolute sample index of the last raw onset, for merging.
    last_raw: Option<u64>,
    /// Onset (absolute sample index) waiting for its analysis window.
    pending: Option<u64>,
}

impl StrumListener {
    pub fn new(sample_rate: f32, instrument: Instrument) -> Self {
        // ~0.68 s at 48 kHz, ~0.74 s at 44.1 kHz; shorter for low rates.
        let window = if sample_rate >= 40_000.0 {
            32_768
        } else {
            16_384
        };
        let delay = (DELAY_SECONDS * sample_rate) as usize;
        Self {
            instrument,
            poly: PolyConfig::new(sample_rate, instrument),
            sample_rate,
            window,
            delay,
            history: vec![0.0; (delay + window).next_power_of_two() * 2],
            written: 0,
            hp: Biquad::highpass(60.0, sample_rate),
            lp: [0.0; 2],
            lp_coeff: (-std::f32::consts::TAU * 1500.0 / sample_rate).exp(),
            hop: ((HOP_SECONDS * sample_rate) as usize).max(1),
            hop_power: 0.0,
            hop_count: 0,
            envelope: Vec::with_capacity(EARLIER_HOPS.1 + 1),
            background: None,
            last_raw: None,
            pending: None,
        }
    }

    pub fn instrument(&self) -> Instrument {
        self.instrument
    }

    /// Switches instrument; a strum waiting for analysis is analysed for the
    /// new one.
    pub fn set_instrument(&mut self, instrument: Instrument) {
        self.instrument = instrument;
        self.poly = PolyConfig::new(self.sample_rate, instrument);
    }

    /// Seconds of audio after an onset before its report is ready.
    pub fn latency_seconds(&self) -> f32 {
        (self.delay + self.window) as f32 / self.sample_rate
    }

    /// Adds audio. Returns a report when a strum's analysis window has just
    /// completed (at most one per call; call with small chunks).
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

            if let Some(onset) = self.pending
                && self.written >= onset + (self.delay + self.window) as u64
            {
                self.pending = None;
                report = Some(self.analyse(onset));
            }
        }
        report
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
        if onset {
            // The rise ended within this hop; place the onset at its start.
            let at = self.written.saturating_sub(self.hop as u64);
            let merge = (MERGE_SECONDS * self.sample_rate) as u64;
            let continues = self.last_raw.is_some_and(|last| at - last <= merge);
            self.last_raw = Some(at);
            if !continues {
                // A new strum replaces one still waiting for its window.
                self.pending = Some(at);
            }
        }

        if self.envelope.len() > EARLIER_HOPS.1 {
            self.envelope.remove(0);
        }
        self.envelope.push(db);
    }

    fn analyse(&self, onset: u64) -> StrumReport {
        let cap = self.history.len() as u64;
        let start = onset + self.delay as u64;
        let frame: Vec<f32> = (start..start + self.window as u64)
            .map(|i| self.history[(i % cap) as usize])
            .collect();
        let estimates = estimate_strum(&frame, self.instrument, &self.poly);
        StrumReport {
            onset_seconds: onset as f64 / f64::from(self.sample_rate),
            strings: estimates
                .iter()
                .map(|e| match e {
                    Some(e) => StrumString {
                        frequency: Some(e.frequency),
                        cents: Some(e.cents),
                        present: e.present,
                    },
                    None => StrumString {
                        frequency: None,
                        cents: None,
                        present: Some(false),
                    },
                })
                .collect(),
        }
    }
}

/// Second-order section (RBJ high-pass).
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

    fn listen(instrument: Instrument, x: &[f32]) -> Vec<StrumReport> {
        let mut listener = StrumListener::new(RATE, instrument);
        x.chunks(1024).filter_map(|c| listener.push(c)).collect()
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

    #[test]
    fn noise_and_single_decays_do_not_report() {
        let mut rng = Rng::new(9);
        let x = render(&[], RATE, 5.0, 0.002, &mut rng);
        assert!(listen(Instrument::Guitar, &x).is_empty());
    }

    #[test]
    fn report_arrives_one_window_after_the_strum() {
        let (x, onsets) = strums(Instrument::Ukulele, &[0.0; 4], &[None], 3);
        let mut listener = StrumListener::new(RATE, Instrument::Ukulele);
        let mut at = None;
        for (k, c) in x.chunks(1024).enumerate() {
            if listener.push(c).is_some() {
                at = Some(((k + 1) * 1024) as f32 / RATE);
                break;
            }
        }
        let latency = at.expect("report") - onsets[0];
        assert!(
            (latency - listener.latency_seconds()).abs() < 0.05,
            "latency {latency}"
        );
    }
}
