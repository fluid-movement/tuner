---
type: component
title: Pitch core
description: 'tuner-core PitchDetector: MPM pitch detection with low-pass prefilter and least-squares peak fit; API, defaults, accuracy limits and cost'
status: draft
generated:
  by: claude-code/opus-5.5
  at: 2026-10-15T17:43:15Z
scope:
  - crates/tuner-core/src/pitch.rs
  - crates/tuner-core/tests
---

`crates/tuner-core/src/pitch.rs` detects the pitch of one monophonic frame. Pure Rust, no dependencies; the WASM wrapper lives in `crates/tuner-wasm`.

## API

- `PitchConfig::new(sample_rate)` gives the defaults: 2048-sample frames, 60–1000 Hz search range, `min_rms` 0.003 (silence gate on the raw frame), `min_clarity` 0.8, `peak_threshold` 0.9, `lowpass_frequency` `Some(1500.0)`.
- `PitchDetector::new(config)` allocates every buffer and panics if the frame cannot hold two periods of `min_frequency` plus 3 ms of filter settling.
- `detect(&mut self, frame)` takes exactly `frame_size` samples and returns `Option<PitchEstimate { frequency, clarity }>`. It never allocates (`tests/no_alloc.rs` checks this with a counting global allocator), so it can run on the audio thread.

## Algorithm

1. RMS gate on the raw frame.
2. 4th-order Butterworth low-pass (two RBJ biquads) from zero state; the first 3 ms are dropped while it settles. This removes noise far above the fundamentals, which otherwise shifts the NSDF peak.
3. McLeod normalized square difference function (NSDF), computed directly for lags `sample_rate/max_frequency ..= sample_rate/min_frequency` (±1 for interpolation), with prefix sums of squares for the denominator.
4. Key maxima (highest point of each positive region); the first one ≥ `peak_threshold` × the highest is chosen. This avoids octave errors when the 2nd or 3rd harmonic is stronger than the fundamental.
5. Least-squares parabola over ±lag/20 around the peak gives the fractional lag and the clarity. Falls back to a 3-point fit for short lags.

## Accuracy and cost

- Clean tones: within ±1 cent at every guitar and ukulele string, at 44.1 and 48 kHz.
- ~30 dB SNR: within ±1 cent. ~10 dB SNR white noise: single frames within ±6 cents, unbiased on average. That is close to the Cramér-Rao bound for one 2048-sample frame, so extra precision must come from smoothing across frames, not from the detector.
- About 1.1 ms per frame natively in release (direct NSDF is O(frame × lags)). If WASM proves too slow, an FFT-based autocorrelation is the next step.

Tests generate their signals in Rust (sines, harmonic stacks, seeded uniform noise); there are no audio fixtures. See [Pitch, notes and tunings](/conventions/pitch-and-tunings.md) for mapping frequencies to strings.
