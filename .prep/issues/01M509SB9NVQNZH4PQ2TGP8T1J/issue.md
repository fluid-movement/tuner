---
title: 'Audio pipeline: mic to AudioWorklet to WASM round trip'
kind: code
parent: 01M509SB0RMTVQ2PY4M21KKQAA
depends_on:
  - 01M509SB56RNMRG2J47ZSYWVPD
tags:
  - iter1
priority: high
---

Capture the microphone with getUserMedia (no echo cancellation, noise suppression or auto gain), run an AudioWorkletProcessor that hosts the tuner WASM module and runs tuner-core's PitchDetector on overlapping frames, and post pitch results to the main thread. Audio starts on a user gesture. The page shows the raw readings (frequency, clarity, nearest note) so the pipeline can be checked; the real tuner UI is a separate issue.

## Open questions
