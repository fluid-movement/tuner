---
title: 'Intonation hint in chord mode: per-string cents from the chord analysis'
kind: code
parent: 01M509SB0RMTVQ2PY4M21KKQAA
tags:
  - chord
---

## Requirement

detect_chord already locates each played string's fundamental; report its offset in cents from the fretted pitch. When a string is consistently off (|cents| > 15 on a settled chord), chord mode shows a hint naming the string and direction (e.g. "C string ~20 c flat — check tuning in Strum mode"), since a detuned ukulele makes every chord sound wrong.

## Open questions
