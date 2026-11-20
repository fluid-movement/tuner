---
title: Chord mode ignores non-musical sounds (confidence gate)
kind: code
parent: 01M509SB0RMTVQ2PY4M21KKQAA
tags:
  - chord
priority: high
---

## Requirement

Review: 60 s of noise bursts (claps, knocks, hiss) produce 81 "chords". A chord is only reported when the spectrum is clearly harmonic and explained by the chosen strings (a confidence gate on detect_chord); otherwise the report says no chord ("Couldn't make out a chord"). Real strums, including quiet and slightly detuned ones, must still pass.

## Open questions
