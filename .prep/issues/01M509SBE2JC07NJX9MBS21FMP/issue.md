---
title: Single-pitch detection in tuner-core
kind: code
parent: 01M509SB0RMTVQ2PY4M21KKQAA
depends_on:
  - 01M509SB56RNMRG2J47ZSYWVPD
tags:
  - iter1
priority: high
---

Implement monophonic pitch detection in tuner-core (frame in, frequency + confidence out), robust for acoustic guitar (E2 82 Hz) through ukulele A4 440 Hz, with note/cents mapping against standard tunings. Cover with synthetic-signal tests (sines, harmonics, noise).

## Open questions
