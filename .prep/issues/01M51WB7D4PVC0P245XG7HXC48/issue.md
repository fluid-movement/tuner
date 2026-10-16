---
title: 'Analyse real recordings: split takes into plucks and strums, score poly against single-string truth'
kind: code
parent: 01M509SB0RMTVQ2PY4M21KKQAA
tags:
  - strum
priority: high
---

The user will record takes of guitar and ukulele (WAV or FLAC, mono or stereo, 44.1/48 kHz) into a gitignored `recordings/` folder. Each take holds one tuning state: each string plucked separately (low to high, ~2 s each), then 3–4 strums (~3 s each), without touching the pegs. File names start with the instrument and describe the take (e.g. `guitar_04_e4-flat.wav`, `ukulele_02_c-sharp.flac`).

Build a tool that, per file: detects onsets and segments the take; classifies segments as single plucks or strums; runs each single pluck through the single-string tuner (MonoTuner) to get per-string ground truth and to report the single-string experience on real strings (time to first reading, share of the ring with a reading, stability and drift of the shown value); runs the polyphonic estimator on each strum with the window placement recommended by the strum research; and reports per-string strum error and verdict agreement against the pluck truth, plus aggregate statistics across takes.

The tool is built and tested before recordings exist, using synthetic takes rendered to WAV. Which recordings become committed regression fixtures (trimmed, or via Git LFS) is decided in a follow-up once real recordings exist.

## Open questions
