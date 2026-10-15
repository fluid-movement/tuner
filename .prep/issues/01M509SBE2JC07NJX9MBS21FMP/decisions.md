## D1: McLeod Pitch Method (NSDF) for monophonic detection
date: 2026-10-15

Use MPM: a normalized square difference function over lags limited to the search range, key-maxima peak picking with a 0.9 threshold, and parabolic interpolation. It is accurate to well under a cent on plucked strings, resists octave errors when the second harmonic dominates (common on a guitar's low E), and gives a clarity value for gating. Direct time-domain computation is about 1.6M multiply-adds per 2048-sample frame at 48 kHz, which is fine for WASM; an FFT-based autocorrelation can come later if profiling demands it. Alternatives: YIN (similar quality, no natural clarity in 0..1), plain FFT peak (poor low-frequency resolution, octave errors), HPS (coarse).

## D2: Low-pass prefilter, least-squares peak fit, realistic noise criterion
date: 2026-10-15

Single-frame MPM with a 3-point parabolic fit was off by up to ~20 cents with white noise at 10 dB SNR: broadband noise puts sample-level wiggles on the broad NSDF peak of low notes. Changes: (1) a 4th-order Butterworth low-pass at 1.5 kHz on each frame (fundamentals are below 1 kHz), skipping the first 3 ms while it settles; (2) a least-squares parabola over ±lag/20 around the peak instead of 3 points. Worst case is now ~4–5 cents per frame at 10 dB SNR.

That is near the Cramér-Rao bound (≈2 cents standard deviation for one 2048-sample frame of 110 Hz at that SNR), so the original "±3 cents at 10 dB" criterion is not physically achievable per frame for low strings. Replaced with: ±6 cents per frame and |mean| ≤ 1 cent over 20 frames at 10 dB, and ±1 cent at 30 dB (realistic for a plucked string near a phone mic). Precision beyond that comes from smoothing across frames in the UI.

Cost: ~1.1 ms per frame natively (release), fine for an AudioWorklet with a 1024-sample hop.
