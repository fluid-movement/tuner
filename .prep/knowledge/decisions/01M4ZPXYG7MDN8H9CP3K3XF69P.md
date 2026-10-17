---
type: decision
title: "Low-pass prefilter, least-squares peak fit, realistic noise criterion"
description: "Low-pass prefilter, least-squares peak fit, realistic noise criterion"
date: "2026-10-15"
issue: 01M509SBE2JC07NJX9MBS21FMP
---

Single-frame MPM with a 3-point parabolic fit was off by up to ~20 cents with white noise at 10 dB SNR: broadband noise puts sample-level wiggles on the broad NSDF peak of low notes. Changes: (1) a 4th-order Butterworth low-pass at 1.5 kHz on each frame (fundamentals are below 1 kHz), skipping the first 3 ms while it settles; (2) a least-squares parabola over ±lag/20 around the peak instead of 3 points. Worst case is now ~4–5 cents per frame at 10 dB SNR.

That is near the Cramér-Rao bound (≈2 cents standard deviation for one 2048-sample frame of 110 Hz at that SNR), so the original "±3 cents at 10 dB" criterion is not physically achievable per frame for low strings. Replaced with: ±6 cents per frame and |mean| ≤ 1 cent over 20 frames at 10 dB, and ±1 cent at 30 dB (realistic for a plucked string near a phone mic). Precision beyond that comes from smoothing across frames in the UI.

Cost: ~1.1 ms per frame natively (release), fine for an AudioWorklet with a 1024-sample hop.
