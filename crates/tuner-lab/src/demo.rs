//! Synthetic takes that follow the recording protocol: every string plucked
//! on its own (low to high, 2 s apart), then strums 3 s apart, with the same
//! tuning throughout. Used by the tests and by `tuner-lab demo`.

use crate::audio::Audio;
use tuner_core::synth::{PluckedString, Rng, highpass, render};
use tuner_core::tuning::Instrument;

pub const PLUCK_SPACING: f32 = 2.0;
pub const STRUM_SPACING: f32 = 3.0;
pub const FIRST_ONSET: f32 = 0.5;

/// A take and the true fundamental of each string (in string order).
pub struct SyntheticTake {
    pub audio: Audio,
    pub frequencies: Vec<f32>,
    /// Onset times of the plucks, then the strums.
    pub pluck_onsets: Vec<f32>,
    pub strum_onsets: Vec<f32>,
}

pub fn synthetic_take(
    instrument: Instrument,
    detune_cents: &[f32],
    strums: usize,
    seed: u64,
) -> SyntheticTake {
    synthetic_take_at(instrument, detune_cents, strums, seed, 48_000.0)
}

pub fn synthetic_take_at(
    instrument: Instrument,
    detune_cents: &[f32],
    strums: usize,
    seed: u64,
    rate: f32,
) -> SyntheticTake {
    let targets = instrument.strings();
    assert_eq!(detune_cents.len(), targets.len());
    let mut rng = Rng::new(seed);
    let frequencies: Vec<f32> = targets
        .iter()
        .zip(detune_cents)
        .map(|(t, c)| t.frequency * 2f32.powf(c / 1200.0))
        .collect();
    let inharmonicity = match instrument {
        Instrument::Guitar => 1e-4,
        Instrument::Ukulele => 2e-5,
    };
    let string = |frequency: f32, onset: f32, level: f32, rng: &mut Rng| PluckedString {
        frequency,
        level,
        onset,
        inharmonicity,
        decay: rng.uniform(0.9, 1.4) * (110.0 / frequency).powf(0.4),
        rolloff: rng.uniform(0.7, 1.1),
        pluck_position: rng.uniform(0.12, 0.22),
        polarization_cents: rng.uniform(0.5, 2.0),
    };

    let mut notes = Vec::new();
    let mut pluck_onsets = Vec::new();
    let mut t = FIRST_ONSET;
    for &f in &frequencies {
        notes.push(string(f, t, 0.08, &mut rng));
        pluck_onsets.push(t);
        t += PLUCK_SPACING;
    }
    let mut strum_onsets = Vec::new();
    for _ in 0..strums {
        strum_onsets.push(t);
        let mut onset = t;
        for &f in &frequencies {
            let level = 0.06 * 10f32.powf(rng.uniform(-8.0, 0.0) / 20.0);
            notes.push(string(f, onset, level, &mut rng));
            onset += rng.uniform(0.008, 0.03);
        }
        t += STRUM_SPACING;
    }
    let mut samples = render(&notes, rate, t + 0.5, 0.002, &mut rng);
    highpass(&mut samples, 100.0, rate);
    SyntheticTake {
        audio: Audio {
            samples,
            sample_rate: rate,
        },
        frequencies,
        pluck_onsets,
        strum_onsets,
    }
}
