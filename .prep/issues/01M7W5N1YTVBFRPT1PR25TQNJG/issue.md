---
title: 'Chord mode: strum a chord, show its tab and name'
kind: code
parent: 01M509SB0RMTVQ2PY4M21KKQAA
tags:
  - chord
priority: high
---

## Requirement

A new mode: strum a chord, the app shows the chord name (e.g. "Am7") and a tab of what was played (fret per string, x for muted). Detection reuses the strum check pipeline (Web Worker, onset detection, poly estimation), but each string can now sound at any fret, not just near its open pitch. Result stays on screen until the next strum. It is the base for later features (chord history, key detection, next-chord suggestions), so each result is a structured value (per-string fret or muted, pitch classes, root, quality), kept in a short history in memory.

Proposed approach: score each voicing in a dictionary of common shapes (open chords and movable shapes up to fret ~7–12) against the strum spectrum with the existing resolved estimator, pick the best, and report "unknown" below a confidence threshold. Free-form per-string fret search is a fallback for voicings outside the dictionary, if feasible. Instrument assumed in standard tuning.

## Open questions

- Ukulele only first (4 strings, smaller search, no B3/E4 presence problem), guitar later — or both from the start?
- Is dictionary-of-voicings acceptable, i.e. unusual voicings may come out as the nearest common shape or "unknown"?
- Which chord qualities matter for v1: major, minor, 7, maj7, m7, sus2/sus4, dim, aug?
- Tab orientation: horizontal ASCII tab (strings as lines) or a chord diagram (fretboard box), or both?
