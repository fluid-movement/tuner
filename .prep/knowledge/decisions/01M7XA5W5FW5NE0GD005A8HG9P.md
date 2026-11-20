---
type: decision
title: 'Chord confidence: tonal crest, explained share, harmonics per string'
description: detect_chord reports nothing unless the spectrum's loudest bin is >= 25 dB above its median (strums >= 39, noise <= 12), the tab explains >= 75% of peak weight (strums >= 0.84) and every played string explains >= 2 peaks. Explained share alone left 8 of 81 noise chords; the crest test removed the rest without touching chord_eval accuracy.
date: "2026-11-20"
issue: 01M7X8P8GDTF2CVYYGE7HPF4N6
scope:
  - crates/tuner-core/src/chord.rs
---


