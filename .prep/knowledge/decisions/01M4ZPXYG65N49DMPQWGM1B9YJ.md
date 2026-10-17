---
type: decision
title: "McLeod Pitch Method (NSDF) for monophonic detection"
description: "McLeod Pitch Method (NSDF) for monophonic detection"
date: "2026-10-15"
issue: 01M509SBE2JC07NJX9MBS21FMP
---

Use MPM: a normalized square difference function over lags limited to the search range, key-maxima peak picking with a 0.9 threshold, and parabolic interpolation. It is accurate to well under a cent on plucked strings, resists octave errors when the second harmonic dominates (common on a guitar's low E), and gives a clarity value for gating. Direct time-domain computation is about 1.6M multiply-adds per 2048-sample frame at 48 kHz, which is fine for WASM; an FFT-based autocorrelation can come later if profiling demands it. Alternatives: YIN (similar quality, no natural clarity in 0..1), plain FFT peak (poor low-frequency resolution, octave errors), HPS (coarse).
