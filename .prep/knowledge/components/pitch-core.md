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
confirmed_commit: e7dc5754fa285b7054afb660c6a8f6499b76642c
---

`crates/tuner-core/src/pitch.rs` detects the pitch of one monophonic frame. Pure Rust; the WASM wrapper lives in `crates/tuner-wasm`. The app does not use the detector's own gates: it runs a permissive detector under the [Note tracker](/components/note-tracker.md).

## API

- `PitchConfig::new(sample_rate)` gives the defaults: 2048-sample frames, 60–1000 Hz search range, `min_rms` 0.003, `min_clarity` 0.8, `peak_threshold` 0.9, `lowpass_frequency` `Some(1500.0)`. `PitchConfig::permissive(sample_rate)` is the same without the level and clarity cut-offs (only digital silence is rejected), for use with a tracker.
- `PitchDetector::new(config)` allocates every buffer and panics if the frame cannot hold two periods of `min_frequency` plus the filter settling time.
- `detect(&mut self, frame)` takes exactly `frame_size` samples and returns `Option<PitchEstimate { frequency, clarity }>`.
- `level()` is the RMS of the last frame's analysed signal (DC removed, after the low-pass), set whether or not a pitch was found. Trackers gate on it, so broadband noise above the band doesn't count.
- `set_lowpass(Option<f32>)` changes the cutoff for later frames (allocation-free); the tracker narrows it to the low partials of the tracked note.
- Nothing allocates after construction (`tests/no_alloc.rs`, per-thread counting allocator), so it all runs on the audio thread.

## Algorithm

1. Subtract the frame mean (DC). A DC offset carries no pitch, and low-passing it makes the filter ring at its cutoff, which looks perfectly periodic.
2. `min_rms` gate on the DC-free frame.
3. 4th-order Butterworth low-pass (two RBJ biquads) from zero state. The first max(3 ms, 1.5 cutoff periods) are dropped while it settles (capped so two periods of `min_frequency` remain). This removes noise far above the fundamentals, which otherwise shifts the NSDF peak.
4. McLeod normalized square difference function (NSDF), computed directly for lags `sample_rate/max_frequency ..= sample_rate/min_frequency` (±1 for interpolation), with prefix sums of squares for the denominator.
5. Key maxima (highest point of each positive region); the first one ≥ `peak_threshold` × the highest is chosen. This avoids octave errors when the 2nd or 3rd harmonic is stronger than the fundamental.
6. Least-squares parabola over ±lag/20 around the peak gives the fractional lag and the clarity. Falls back to a 3-point fit for short lags.

## Accuracy and cost

- Clean tones: within ±1 cent at every guitar and ukulele string, at 44.1 and 48 kHz (with the default 1.5 kHz band; the short settle time is what keeps low notes accurate there, so don't lengthen it for the wide band).
- ~30 dB SNR: within ±1 cent. ~10 dB SNR white noise: single frames within ±6 cents, unbiased on average. That is close to the Cramér-Rao bound for one 2048-sample frame, so extra precision comes from tracking across frames.
- Real strings are stiff: upper partials run sharp, so a wide band reads low strings a few cents sharp right after the pluck. Narrowing the band to ~4.5× the fundamental (the tracker does this) fixes that.
- About 1.1 ms per frame natively in release (direct NSDF is O(frame × lags)).

Tests generate their signals in Rust (sines, harmonic stacks, seeded uniform noise); there are no audio fixtures. See [Pitch, notes and tunings](/conventions/pitch-and-tunings.md) for mapping frequencies to strings.
