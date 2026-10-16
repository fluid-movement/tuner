---
title: 'Analyse real recordings: split takes into plucks and strums, score poly against single-string truth'
kind: code
parent: 01M509SB0RMTVQ2PY4M21KKQAA
tags:
  - strum
priority: high
---

The user will record takes of guitar and ukulele (WAV/FLAC, mono, 44.1/48 kHz) into a gitignored `recordings/` folder. Each take holds one tuning state: each string plucked separately (low to high, ~2 s each), then 3–4 strums (~3 s each), without touching the pegs. File names describe the take (e.g. `guitar_04_e4-flat.wav`).

Build a tool that, per file: detects onsets and segments the take; classifies segments as single plucks or strums; measures each single pluck with the monophonic PitchDetector to get per-string ground truth; runs the polyphonic estimator on each strum with the window placement recommended by the strum research; and reports per-string error and verdict agreement, plus aggregate statistics across takes. It should also report how the single-string detector behaves on the real plucks (stability over the ring).

The results decide how the strum check is tuned (window length, presence detection, B3/E4 handling) and which recordings become committed regression fixtures.

## Open questions
- Which takes become committed fixtures (trimmed, or via Git LFS)?
