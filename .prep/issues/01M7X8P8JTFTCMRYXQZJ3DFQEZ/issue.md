---
title: Key detection and next-chord suggestions from the chord history
kind: code
parent: 01M509SB0RMTVQ2PY4M21KKQAA
tags:
  - chord
  - theory
priority: high
---

## Requirement

The chord history becomes musically useful (the user's stated next step: chain chords, determine the key, suggest next chords). From the recent chords, estimate the key (major and relative minor, weighted by recency and by how well each chord fits the key's diatonic chords and tonic/dominant functions) with a confidence; label each history chord with its Roman numeral in that key; suggest likely next chords (common-practice and pop progressions: V→I, IV→V, ii→V, vi→IV …) with a playable ukulele shape for each, found algorithmically (low frets, small span, all chord tones). Consecutive repeats of the same chord collapse into one history entry. Theory lives in web/src/lib/theory (decision 01M7X3VYJ9506CARD0JB5WDKZM).

## Open questions
