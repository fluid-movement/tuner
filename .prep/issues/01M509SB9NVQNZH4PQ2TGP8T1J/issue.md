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

Capture the microphone with getUserMedia, run an AudioWorkletProcessor that hosts the WASM module, analyse frames there with a stub analyser, and post results to the main thread. Start audio on a user gesture.

## Open questions
