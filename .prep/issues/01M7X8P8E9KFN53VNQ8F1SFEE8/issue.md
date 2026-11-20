---
title: 'Chord mode keeps up with real strumming: spectral-flux onsets, interrupted strums settle'
kind: code
parent: 01M509SB0RMTVQ2PY4M21KKQAA
tags:
  - chord
  - latency
priority: high
---

## Requirement

Review (2026-11-20, chord_stream_eval): with strums 0.5 s apart only 20% of strums are detected and settled; 0.3 s apart 10%. The envelope onset rule needs an 8 dB rise, but re-strumming over ringing strings barely raises the level; and a strum interrupted by the next one before its last stage never settles, so it never reaches the history. Strums must be detected during continuous playing (spectral flux: new energy across frequency bands), and a strum cut short by the next one settles on the audio it got (if at least the first-stage window). Existing single-strum behaviour, noise rejection of onsets and the strum tuning check must not regress.

## Open questions
