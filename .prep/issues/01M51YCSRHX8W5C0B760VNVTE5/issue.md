---
title: 'Note tracking for single-string tuning: adaptive gate, lock and follow, hold'
kind: code
parent: 01M509SB0RMTVQ2PY4M21KKQAA
tags:
  - iter1
priority: critical
---

User feedback from the live site (2026-10-16): plucks are hard to register, the initial pluck shows briefly and the dial disappears quickly; overall a poor experience. A synthetic probe at realistic phone levels (pluck peak ≈ −34 dBFS, noise −50..−60 dBFS, 120 Hz mic low cut) confirms it: the stateless detector shows a reading for only 25–60% of the first 2 s after a pluck and E4 drops out after ~0.5 s, because of the absolute −50 dBFS RMS gate, the fixed 0.8 clarity threshold and a 250 ms display timeout. Loosening thresholds alone admits wrong readings in noise.

Replace that with a note tracker in tuner-core, shared later by strum mode: an adaptive noise floor instead of an absolute gate; a confident onset to lock onto a note, then following it with much weaker readings that stay near the locked pitch (octave slips folded back); quick retargeting when a different string is plucked; and holding the last reading dimmed for a few seconds after the string fades. The UI shows tracking vs held state, an input level meter, and an optional debug overlay (raw frequency, clarity, level, noise floor). Measure the experience with simulated plucks at realistic levels.

## Open questions
