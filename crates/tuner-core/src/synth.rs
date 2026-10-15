//! Synthetic plucked-string signals for tests and evaluation. Not used by
//! the tuner at runtime.
//!
//! A string is a sum of partials `n = 1, 2, …` at `n·f0·sqrt(1 + B·n²)`
//! (stiff-string inharmonicity `B`), with amplitudes shaped by a spectral
//! roll-off and the pluck position (a pluck at fraction `p` of the length
//! suppresses partials near multiples of `1/p`), and exponential decays that
//! get faster for higher partials.

use crate::tuning::Instrument;
use std::f32::consts::{PI, TAU};

/// One plucked string within a signal.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PluckedString {
    /// Fundamental frequency in Hz (before inharmonicity).
    pub frequency: f32,
    /// Peak amplitude of the fundamental partial's envelope scale.
    pub level: f32,
    /// Start time in seconds.
    pub onset: f32,
    /// Inharmonicity coefficient `B` (≈1e-5 for nylon, ≈1e-4 for steel).
    pub inharmonicity: f32,
    /// Decay time constant of the fundamental in seconds.
    pub decay: f32,
    /// Spectral roll-off exponent: partial `n` is scaled by `n^-rolloff`.
    pub rolloff: f32,
    /// Pluck position as a fraction of the string length (0..0.5).
    pub pluck_position: f32,
    /// Real strings vibrate in two polarizations with slightly different
    /// frequencies, so partials beat. Each partial gets a second component
    /// this many cents higher, at 40% amplitude with a longer decay. 0 = off.
    pub polarization_cents: f32,
}

impl PluckedString {
    /// Frequency of partial `n` (1-based).
    pub fn partial_frequency(&self, n: u32) -> f32 {
        let n = n as f32;
        n * self.frequency * (1.0 + self.inharmonicity * n * n).sqrt()
    }
}

/// Small deterministic PRNG (PCG32) so signals are reproducible.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        let mut rng = Self(
            seed.wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407),
        );
        rng.next_u32();
        rng
    }

    pub fn next_u32(&mut self) -> u32 {
        let old = self.0;
        self.0 = old
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        xorshifted.rotate_right((old >> 59) as u32)
    }

    /// Uniform in `lo..hi`.
    pub fn uniform(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * (self.next_u32() >> 8) as f32 / (1u32 << 24) as f32
    }

    /// Standard normal (Box–Muller).
    pub fn normal(&mut self) -> f32 {
        let u1 = self.uniform(f32::EPSILON, 1.0);
        let u2 = self.uniform(0.0, 1.0);
        (-2.0 * u1.ln()).sqrt() * (TAU * u2).cos()
    }
}

/// Renders `strings` plus white Gaussian noise of RMS `noise_rms`.
pub fn render(
    strings: &[PluckedString],
    sample_rate: f32,
    seconds: f32,
    noise_rms: f32,
    rng: &mut Rng,
) -> Vec<f32> {
    let n = (seconds * sample_rate) as usize;
    let mut out = vec![0.0f32; n];
    let max_partial_hz = (sample_rate / 2.0).min(6000.0);
    for s in strings {
        let start = (s.onset * sample_rate) as usize;
        for k in 1u32.. {
            let f = s.partial_frequency(k);
            if f >= max_partial_hz {
                break;
            }
            let pluck = (PI * k as f32 * s.pluck_position).sin().abs();
            let amp = s.level * (k as f32).powf(-s.rolloff) * pluck;
            if amp < 1e-5 {
                continue;
            }
            // Higher partials decay faster.
            let tau = s.decay / (1.0 + 0.35 * (k - 1) as f32);
            let phase = rng.uniform(0.0, TAU);
            let w = TAU * f / sample_rate;
            let d = (-1.0 / (tau * sample_rate)).exp();
            let mut components = vec![(w, d, amp, phase)];
            if s.polarization_cents > 0.0 {
                let w2 = w * 2f32.powf(s.polarization_cents / 1200.0);
                let d2 = (-1.0 / (1.5 * tau * sample_rate)).exp();
                components.push((w2, d2, 0.4 * amp, rng.uniform(0.0, TAU)));
            }
            for (w, d, amp, phase) in components {
                let mut env = amp;
                for (i, y) in out.iter_mut().enumerate().skip(start) {
                    *y += env * (w * (i - start) as f32 + phase).sin();
                    env *= d;
                    if env < 1e-6 {
                        break;
                    }
                }
            }
        }
    }
    if noise_rms > 0.0 {
        for y in &mut out {
            *y += noise_rms * rng.normal();
        }
    }
    out
}

/// Settings for [`random_strum`].
#[derive(Debug, Clone, Copy)]
pub struct StrumOptions {
    /// Each string is detuned uniformly within ±this many cents.
    pub max_detune_cents: f32,
    /// Time between successive strings in the strum, in seconds (range).
    pub onset_spread: (f32, f32),
    /// String levels in dB relative to the loudest (range).
    pub level_db: (f32, f32),
    /// Inharmonicity coefficient range.
    pub inharmonicity: (f32, f32),
}

impl StrumOptions {
    pub fn for_instrument(instrument: Instrument) -> Self {
        match instrument {
            Instrument::Guitar => Self {
                max_detune_cents: 50.0,
                onset_spread: (0.008, 0.035),
                level_db: (-12.0, 0.0),
                inharmonicity: (2e-5, 2e-4),
            },
            // Nylon/fluorocarbon strings: lower inharmonicity.
            Instrument::Ukulele => Self {
                max_detune_cents: 50.0,
                onset_spread: (0.008, 0.03),
                level_db: (-10.0, 0.0),
                inharmonicity: (5e-6, 5e-5),
            },
        }
    }
}

/// A downstroke strum over all strings of `instrument` with random detuning
/// and playing variation. Strings are returned in string order with their
/// true fundamentals.
pub fn random_strum(
    instrument: Instrument,
    options: &StrumOptions,
    rng: &mut Rng,
) -> Vec<PluckedString> {
    let targets = instrument.strings();
    // A downstroke hits strings in physical order: low to high on guitar,
    // G4 C4 E4 A4 (string order) on ukulele.
    let mut onset = rng.uniform(0.0, 0.01);
    targets
        .iter()
        .map(|t| {
            let detune = rng.uniform(-options.max_detune_cents, options.max_detune_cents);
            let frequency = t.frequency * 2f32.powf(detune / 1200.0);
            let s = PluckedString {
                frequency,
                level: 10f32.powf(rng.uniform(options.level_db.0, options.level_db.1) / 20.0)
                    * 0.15,
                onset,
                inharmonicity: rng.uniform(options.inharmonicity.0, options.inharmonicity.1),
                // Low strings ring longer.
                decay: rng.uniform(0.8, 1.6) * (110.0 / frequency).powf(0.4),
                rolloff: rng.uniform(0.6, 1.2),
                pluck_position: rng.uniform(0.1, 0.25),
                polarization_cents: 0.0,
            };
            onset += rng.uniform(options.onset_spread.0, options.onset_spread.1);
            s
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PitchConfig, PitchDetector};

    #[test]
    fn rng_is_deterministic_and_in_range() {
        let (mut a, mut b) = (Rng::new(5), Rng::new(5));
        for _ in 0..1000 {
            let x = a.uniform(-2.0, 3.0);
            assert_eq!(x, b.uniform(-2.0, 3.0));
            assert!((-2.0..3.0).contains(&x));
        }
        let mean = (0..20_000).map(|_| a.normal()).sum::<f32>() / 20_000.0;
        assert!(mean.abs() < 0.03);
    }

    #[test]
    fn inharmonic_partials_are_sharp() {
        let s = PluckedString {
            frequency: 100.0,
            level: 1.0,
            onset: 0.0,
            inharmonicity: 1e-4,
            decay: 1.0,
            rolloff: 1.0,
            pluck_position: 0.2,
            polarization_cents: 0.0,
        };
        assert!((s.partial_frequency(1) - 100.005).abs() < 1e-3);
        assert!(s.partial_frequency(10) > 1000.0 * 1.004);
    }

    #[test]
    fn a_single_rendered_string_is_detected_at_its_fundamental() {
        let rate = 48_000.0;
        let mut rng = Rng::new(1);
        let s = PluckedString {
            frequency: 110.0,
            level: 0.3,
            onset: 0.05,
            inharmonicity: 1e-4,
            decay: 1.2,
            rolloff: 0.8,
            pluck_position: 0.15,
            polarization_cents: 0.0,
        };
        let signal = render(&[s], rate, 0.5, 0.001, &mut rng);
        let start = (0.2 * rate) as usize;
        let est = PitchDetector::new(PitchConfig::new(rate))
            .detect(&signal[start..start + 2048])
            .expect("pitch");
        assert!((est.frequency - 110.0).abs() < 0.2, "{}", est.frequency);
    }

    #[test]
    fn strums_follow_string_order_with_spread_onsets() {
        let mut rng = Rng::new(9);
        for inst in [Instrument::Guitar, Instrument::Ukulele] {
            let strum = random_strum(inst, &StrumOptions::for_instrument(inst), &mut rng);
            assert_eq!(strum.len(), inst.strings().len());
            for (s, t) in strum.iter().zip(inst.strings()) {
                let cents = crate::tuning::cents_between(s.frequency, t.frequency);
                assert!(cents.abs() <= 50.0);
            }
            assert!(strum.windows(2).all(|w| w[1].onset > w[0].onset));
        }
    }
}
