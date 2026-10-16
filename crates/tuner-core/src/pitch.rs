//! Monophonic pitch detection with the McLeod Pitch Method (MPM).
//!
//! The normalized square difference function (NSDF) is computed directly in
//! the time domain over the lags of the search range, its key maxima are
//! picked, and the first one above a fraction of the highest is refined with
//! parabolic interpolation. See McLeod & Wyvill, "A smarter way to find pitch"
//! (2005).

/// Leading samples of each filtered frame that are not analysed while the
/// low-pass settles: at least 3 ms, and 1.5 periods of the cutoff for low
/// cutoffs (their impulse response rings longer).
fn settle_samples(cutoff: Option<f32>, sample_rate: f32) -> usize {
    cutoff.map_or(0, |fc| {
        (sample_rate * 0.003f32.max(1.5 / fc)).ceil() as usize
    })
}

/// The peak refinement fits lags within `lag / REFINE_WINDOW_DIVISOR` of the
/// peak: about a twentieth of a period, where the peak is still parabolic.
const REFINE_WINDOW_DIVISOR: usize = 20;

/// Settings for a [`PitchDetector`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PitchConfig {
    /// Sample rate of the frames passed to [`PitchDetector::detect`], in Hz.
    pub sample_rate: f32,
    /// Number of samples per frame. It must cover at least two periods of
    /// `min_frequency`, plus a few ms for the low-pass filter to settle.
    pub frame_size: usize,
    /// Lowest frequency searched, in Hz.
    pub min_frequency: f32,
    /// Highest frequency searched, in Hz.
    pub max_frequency: f32,
    /// Frames whose RMS is below this are treated as silence.
    pub min_rms: f32,
    /// Estimates whose clarity is below this are rejected.
    pub min_clarity: f32,
    /// A key maximum is accepted when it reaches this fraction of the highest
    /// one. Lower values favour shorter periods (higher octaves).
    pub peak_threshold: f32,
    /// Cutoff of the low-pass filter applied to each frame before analysis,
    /// in Hz, or `None` to analyse the raw frame. Removing content far above
    /// the search range keeps broadband noise from shifting the NSDF peak.
    pub lowpass_frequency: Option<f32>,
}

impl PitchConfig {
    /// Defaults for guitar and ukulele at the given sample rate: 2048-sample
    /// frames and a 60–1000 Hz range, which covers a detuned low E up to the
    /// upper frets of a ukulele's A string.
    pub fn new(sample_rate: f32) -> Self {
        Self {
            sample_rate,
            frame_size: 2048,
            min_frequency: 60.0,
            max_frequency: 1000.0,
            min_rms: 0.003,
            min_clarity: 0.8,
            peak_threshold: 0.9,
            lowpass_frequency: Some(1500.0),
        }
    }

    /// Like [`new`](Self::new) but without the level and clarity cut-offs
    /// (only digital silence is rejected), for use with a
    /// [`NoteTracker`](crate::NoteTracker), which decides what to show
    /// relative to the noise floor and the tracked note.
    pub fn permissive(sample_rate: f32) -> Self {
        Self {
            min_rms: 1e-6,
            min_clarity: 0.0,
            ..Self::new(sample_rate)
        }
    }
}

/// A detected pitch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PitchEstimate {
    /// Fundamental frequency in Hz.
    pub frequency: f32,
    /// How periodic the frame is at that frequency, from 0 to 1.
    pub clarity: f32,
}

/// Detects the pitch of single frames. All buffers are allocated up front, so
/// [`detect`](Self::detect) is safe to call on a real-time audio thread.
#[derive(Debug, Clone)]
pub struct PitchDetector {
    config: PitchConfig,
    min_lag: usize,
    max_lag: usize,
    /// Leading filtered samples skipped while the low-pass settles.
    settle: usize,
    /// RMS of the last frame's analysed signal.
    level: f32,
    lowpass: Option<[Biquad; 2]>,
    /// The frame after filtering.
    filtered: Vec<f32>,
    nsdf: Vec<f32>,
    /// Prefix sums of squared samples, for the NSDF denominator.
    energy: Vec<f64>,
}

impl PitchDetector {
    /// # Panics
    ///
    /// If the frame is too short for `min_frequency` or the range is empty.
    pub fn new(config: PitchConfig) -> Self {
        let min_lag = ((config.sample_rate / config.max_frequency).floor() as usize).max(2);
        let max_lag = (config.sample_rate / config.min_frequency).ceil() as usize;
        let settle = settle_samples(config.lowpass_frequency, config.sample_rate);
        assert!(min_lag < max_lag, "empty frequency range");
        assert!(
            2 * max_lag + settle <= config.frame_size,
            "frame_size must cover two periods of min_frequency"
        );
        Self {
            config,
            min_lag,
            max_lag,
            settle,
            level: 0.0,
            // One extra lag on each side so peaks at the range edges can be
            // interpolated.
            lowpass: config
                .lowpass_frequency
                .map(|f| Biquad::butterworth_lowpass_4th(f, config.sample_rate)),
            filtered: vec![0.0; config.frame_size],
            nsdf: vec![0.0; max_lag + 2],
            energy: vec![0.0; config.frame_size + 1],
        }
    }

    pub fn config(&self) -> &PitchConfig {
        &self.config
    }

    /// RMS level of the last frame passed to [`detect`](Self::detect), with
    /// DC removed and after the low-pass filter when one is configured (so
    /// broadband noise above the search range doesn't count). Set whether or
    /// not a pitch was found.
    pub fn level(&self) -> f32 {
        self.level
    }

    /// Changes the low-pass cutoff (Hz) for later frames, or removes the
    /// filter with `None`. Allocation-free, for use on the audio thread: a
    /// tracker narrows the band once it knows the note, so the strongly
    /// inharmonic upper partials of low strings don't bias the pitch sharp.
    pub fn set_lowpass(&mut self, cutoff: Option<f32>) {
        if cutoff != self.config.lowpass_frequency {
            self.config.lowpass_frequency = cutoff;
            self.lowpass =
                cutoff.map(|f| Biquad::butterworth_lowpass_4th(f, self.config.sample_rate));
            // Keep two periods of the lowest frequency analysable.
            self.settle = settle_samples(cutoff, self.config.sample_rate)
                .min(self.config.frame_size - 2 * self.max_lag);
        }
    }

    /// Estimates the pitch of `frame`, which must hold exactly
    /// `config.frame_size` samples. Returns `None` for silence and for frames
    /// without a clear pitch in the search range.
    pub fn detect(&mut self, frame: &[f32]) -> Option<PitchEstimate> {
        assert_eq!(frame.len(), self.config.frame_size, "wrong frame size");
        let n = frame.len();

        // Remove any DC offset first: it carries no pitch, and filtering it
        // makes the low-pass ring (which looks perfectly periodic).
        let mean = frame.iter().map(|&x| f64::from(x)).sum::<f64>() / n as f64;
        let mut power = 0.0f64;
        for (y, &x) in self.filtered.iter_mut().zip(frame) {
            let v = f64::from(x) - mean;
            *y = v as f32;
            power += v * v;
        }
        let rms = (power / n as f64).sqrt() as f32;
        self.level = rms;
        if rms < self.config.min_rms {
            return None;
        }

        if let Some(stages) = &self.lowpass {
            for stage in stages {
                stage.apply(&mut self.filtered);
            }
        }
        let frame = &self.filtered[self.settle..];

        self.energy[0] = 0.0;
        for (i, &x) in frame.iter().enumerate() {
            self.energy[i + 1] = self.energy[i] + f64::from(x) * f64::from(x);
        }
        let n = frame.len();
        self.level = (self.energy[n] / n as f64).sqrt() as f32;

        let first = self.min_lag.saturating_sub(1).max(1);
        let last = self.max_lag + 1;
        self.nsdf.fill(0.0);
        for tau in first..=last {
            self.nsdf[tau] = nsdf_at(frame, &self.energy, tau);
        }

        let peak = self.pick_peak(first, last)?;
        let (tau, clarity) = self.refine(peak);
        if clarity < self.config.min_clarity {
            return None;
        }
        let frequency = self.config.sample_rate / tau;
        if frequency < self.config.min_frequency || frequency > self.config.max_frequency {
            return None;
        }
        Some(PitchEstimate { frequency, clarity })
    }

    /// Finds the key maxima of the NSDF (the highest point of each positive
    /// region) and returns the lag of the first one that reaches
    /// `peak_threshold` of the highest.
    fn pick_peak(&self, first: usize, last: usize) -> Option<usize> {
        let nsdf = &self.nsdf;
        let (lo, hi) = (self.min_lag, self.max_lag);
        // Peaks are only accepted inside the search range; the lag on either
        // side exists for interpolation.
        let is_peak = |t: usize| nsdf[t] > 0.0 && nsdf[t] >= nsdf[t - 1] && nsdf[t] > nsdf[t + 1];

        let mut best = f32::MIN;
        let mut region_peak: Option<usize> = None;
        let mut peaks = [0usize; 64];
        let mut count = 0;
        let mut flush = |peak: &mut Option<usize>, best: &mut f32| {
            if let Some(t) = peak.take() {
                if count < peaks.len() {
                    peaks[count] = t;
                    count += 1;
                }
                *best = best.max(nsdf[t]);
            }
        };

        for t in (first + 1)..last {
            if nsdf[t] <= 0.0 {
                flush(&mut region_peak, &mut best);
                continue;
            }
            if t >= lo && t <= hi && is_peak(t) && region_peak.is_none_or(|p| nsdf[t] > nsdf[p]) {
                region_peak = Some(t);
            }
        }
        flush(&mut region_peak, &mut best);

        let threshold = best * self.config.peak_threshold;
        peaks[..count]
            .iter()
            .copied()
            .find(|&t| nsdf[t] >= threshold)
    }

    /// Locates the vertex of the NSDF peak at lag `t` to a fraction of a
    /// sample. A least-squares parabola over a window proportional to the lag
    /// averages out the small wiggles noise adds to the broad peaks of low
    /// notes, which shift a three-point fit by several cents.
    fn refine(&self, t: usize) -> (f32, f32) {
        let frame = &self.filtered[self.settle..];
        let half = (t / REFINE_WINDOW_DIVISOR).max(1);
        if half == 1 || t + half >= frame.len() {
            return interpolate(self.nsdf[t - 1], self.nsdf[t], self.nsdf[t + 1], t);
        }

        // Fit y = a·d² + b·d + c over d in -half..=half. The offsets are
        // symmetric, so the odd sums vanish.
        let (mut sy, mut sdy, mut sd2y, mut sd2, mut sd4) = (0.0, 0.0, 0.0, 0.0, 0.0);
        for d in -(half as i64)..=half as i64 {
            let y = f64::from(nsdf_at(frame, &self.energy, (t as i64 + d) as usize));
            let d = d as f64;
            sy += y;
            sdy += d * y;
            sd2y += d * d * y;
            sd2 += d * d;
            sd4 += d * d * d * d;
        }
        let count = (2 * half + 1) as f64;
        let a = (sd2y - sd2 * sy / count) / (sd4 - sd2 * sd2 / count);
        if a >= 0.0 {
            return interpolate(self.nsdf[t - 1], self.nsdf[t], self.nsdf[t + 1], t);
        }
        let b = sdy / sd2;
        let c = (sy - a * sd2) / count;
        let shift = (-b / (2.0 * a)).clamp(-(half as f64), half as f64);
        let value = c - b * b / (4.0 * a);
        (t as f32 + shift as f32, (value as f32).min(1.0))
    }
}

/// NSDF at one lag: twice the autocorrelation over the sum of squares of the
/// overlapping parts. `energy` holds prefix sums of squared samples.
fn nsdf_at(frame: &[f32], energy: &[f64], tau: usize) -> f32 {
    let n = frame.len();
    let r: f64 = frame[..n - tau]
        .iter()
        .zip(&frame[tau..])
        .map(|(&a, &b)| f64::from(a) * f64::from(b))
        .sum();
    // Sum of x[j]^2 + x[j+tau]^2 for j in 0..n-tau.
    let m = energy[n - tau] + (energy[n] - energy[tau]);
    if m > 0.0 { (2.0 * r / m) as f32 } else { 0.0 }
}

/// A second-order IIR section (RBJ cookbook coefficients, normalized so
/// `a0 = 1`).
#[derive(Debug, Clone, Copy)]
struct Biquad {
    b: [f32; 3],
    a: [f32; 2],
}

impl Biquad {
    fn lowpass(cutoff: f32, sample_rate: f32, q: f32) -> Self {
        let w = std::f32::consts::TAU * cutoff / sample_rate;
        let alpha = w.sin() / (2.0 * q);
        let cos = w.cos();
        let a0 = 1.0 + alpha;
        let b1 = (1.0 - cos) / a0;
        Self {
            b: [b1 / 2.0, b1, b1 / 2.0],
            a: [-2.0 * cos / a0, (1.0 - alpha) / a0],
        }
    }

    /// Two cascaded sections forming a 4th-order Butterworth low-pass.
    fn butterworth_lowpass_4th(cutoff: f32, sample_rate: f32) -> [Self; 2] {
        [
            Self::lowpass(cutoff, sample_rate, 0.541_196_1),
            Self::lowpass(cutoff, sample_rate, 1.306_563),
        ]
    }

    /// Filters `samples` in place, starting from zero state.
    fn apply(&self, samples: &mut [f32]) {
        let (mut x1, mut x2, mut y1, mut y2) = (0.0, 0.0, 0.0, 0.0);
        for s in samples {
            let x = *s;
            let y =
                self.b[0] * x + self.b[1] * x1 + self.b[2] * x2 - self.a[0] * y1 - self.a[1] * y2;
            (x2, x1, y2, y1) = (x1, x, y1, y);
            *s = y;
        }
    }
}

/// Fits a parabola through three neighbouring values and returns the position
/// and value of its vertex.
fn interpolate(left: f32, mid: f32, right: f32, t: usize) -> (f32, f32) {
    let denom = left - 2.0 * mid + right;
    if denom == 0.0 {
        return (t as f32, mid);
    }
    let shift = 0.5 * (left - right) / denom;
    let value = mid - 0.25 * (left - right) * shift;
    (t as f32 + shift, value.min(1.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tuning::{Instrument, cents_between};
    use std::f32::consts::TAU;

    const RATES: [f32; 2] = [44_100.0, 48_000.0];

    /// Sum of harmonics with the given amplitudes (index 0 is the fundamental).
    fn tone(freq: f32, rate: f32, amplitudes: &[f32], n: usize) -> Vec<f32> {
        (0..n)
            .map(|i| {
                let t = i as f32 / rate;
                amplitudes
                    .iter()
                    .enumerate()
                    .map(|(h, a)| a * (TAU * freq * (h + 1) as f32 * t).sin())
                    .sum()
            })
            .collect()
    }

    /// Deterministic uniform noise in -amp..amp.
    fn noise(n: usize, amp: f32, mut seed: u32) -> Vec<f32> {
        (0..n)
            .map(|_| {
                seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                (seed >> 8) as f32 / (1u32 << 24) as f32 * 2.0 * amp - amp
            })
            .collect()
    }

    fn all_strings() -> impl Iterator<Item = f32> {
        [Instrument::Guitar, Instrument::Ukulele]
            .into_iter()
            .flat_map(|i| i.strings().iter().map(|s| s.frequency))
    }

    fn assert_within(detected: Option<PitchEstimate>, expected: f32, cents: f32) {
        let est = detected.unwrap_or_else(|| panic!("no pitch for {expected} Hz"));
        let off = cents_between(est.frequency, expected);
        assert!(
            off.abs() <= cents,
            "{expected} Hz detected as {} Hz ({off:+.2} cents)",
            est.frequency
        );
    }

    #[test]
    fn detects_sines_at_every_string() {
        for rate in RATES {
            let mut det = PitchDetector::new(PitchConfig::new(rate));
            for f in all_strings() {
                assert_within(det.detect(&tone(f, rate, &[0.5], 2048)), f, 1.0);
            }
        }
    }

    #[test]
    fn detects_detuned_sines() {
        let rate = 48_000.0;
        let mut det = PitchDetector::new(PitchConfig::new(rate));
        for f in [70.0, 82.0, 83.5, 108.0, 263.0, 446.0, 880.0] {
            assert_within(det.detect(&tone(f, rate, &[0.5], 2048)), f, 1.0);
        }
    }

    #[test]
    fn weak_fundamental_is_not_an_octave_error() {
        // Low strings often have a stronger second or third harmonic than
        // fundamental.
        let spectra: [&[f32]; 3] = [
            &[0.2, 1.0, 0.6, 0.3],
            &[0.15, 0.5, 1.0, 0.4, 0.2],
            &[0.3, 0.8, 0.5, 0.6, 0.4, 0.3],
        ];
        for rate in RATES {
            let mut det = PitchDetector::new(PitchConfig::new(rate));
            for f in all_strings() {
                for amps in spectra {
                    let sum: f32 = amps.iter().sum();
                    let scaled: Vec<f32> = amps.iter().map(|a| a * 0.5 / sum).collect();
                    assert_within(det.detect(&tone(f, rate, &scaled, 2048)), f, 1.0);
                }
            }
        }
    }

    /// The test tone plus uniform noise of amplitude `noise_amp`. The tone's
    /// RMS is about 0.21; noise of amplitude 0.12 is 10 dB below it and 0.012
    /// is 30 dB below.
    fn noisy_tone(f: f32, rate: f32, noise_amp: f32, seed: u32) -> Vec<f32> {
        let sig = tone(f, rate, &[0.25, 0.1, 0.06], 2048);
        let n = noise(2048, noise_amp, seed);
        sig.iter().zip(&n).map(|(s, n)| s + n).collect()
    }

    #[test]
    fn detects_through_moderate_noise() {
        for rate in RATES {
            let mut det = PitchDetector::new(PitchConfig::new(rate));
            for (k, f) in all_strings().enumerate() {
                for seed in 0..10 {
                    let x = noisy_tone(f, rate, 0.012, 100 * k as u32 + seed);
                    assert_within(det.detect(&x), f, 1.0);
                }
            }
        }
    }

    #[test]
    fn heavy_noise_stays_near_the_single_frame_limit_without_bias() {
        // At 10 dB SNR the Cramér-Rao bound for one 2048-sample frame of a
        // 110 Hz tone is about 2 cents standard deviation, so single frames
        // may be a few cents off; averaging frames must remove the error.
        for rate in RATES {
            let mut det = PitchDetector::new(PitchConfig::new(rate));
            for (k, f) in all_strings().enumerate() {
                let mut sum = 0.0;
                let frames = 20;
                for seed in 0..frames {
                    let x = noisy_tone(f, rate, 0.12, 1000 * k as u32 + seed);
                    let est = det
                        .detect(&x)
                        .unwrap_or_else(|| panic!("no pitch for {f} Hz"));
                    let off = cents_between(est.frequency, f);
                    assert!(off.abs() <= 6.0, "{f} Hz frame off by {off:+.2} cents");
                    sum += off;
                }
                let mean = sum / frames as f32;
                assert!(mean.abs() <= 1.0, "{f} Hz mean off by {mean:+.2} cents");
            }
        }
    }

    #[test]
    fn silence_and_noise_have_no_pitch() {
        let mut det = PitchDetector::new(PitchConfig::new(48_000.0));
        assert_eq!(det.detect(&[0.0; 2048]), None);
        assert_eq!(
            det.detect(&noise(2048, 0.001, 1)),
            None,
            "below the RMS gate"
        );
        for seed in 1..20 {
            assert_eq!(det.detect(&noise(2048, 0.5, seed)), None, "seed {seed}");
        }
    }

    #[test]
    #[should_panic(expected = "two periods")]
    fn rejects_frames_too_short_for_the_range() {
        PitchDetector::new(PitchConfig {
            frame_size: 1024,
            ..PitchConfig::new(48_000.0)
        });
    }
}
