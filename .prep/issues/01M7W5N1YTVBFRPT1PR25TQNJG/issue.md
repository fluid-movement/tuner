---
title: 'Chord mode: strum a chord, show its tab and name'
kind: code
parent: 01M509SB0RMTVQ2PY4M21KKQAA
tags:
  - chord
priority: high
---

## Requirement

A new mode: strum a chord, the app shows the chord name, a chord diagram and a horizontal tab (highest string on top, x for muted). Ukulele only for now (standard tuning). Detection reuses the strum pipeline (Web Worker, onset detection, staged reports: 0.17 s then 0.34 s windows). Each settled chord is kept as a structured value (frets per string, root, quality) in a short in-memory history shown as chips: the base for chord chains, key detection and next-chord suggestions.

Approach: exhaustive search over every string's frets 0–12 (or muted) whose fundamental is a strong spectral peak, scored by the share of peak weight their partials explain, with small tie-breaks for muted strings, high frets and a bonus for common shapes (the same notes can be fingered several ways and the sound can't tell them apart). Qualities: major, minor, 7, maj7, m7, sus2, sus4, dim, aug; other note sets show "?".

## Open questions
