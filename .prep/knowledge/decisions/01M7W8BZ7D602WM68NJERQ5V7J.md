---
type: decision
title: 'Chord detection: exhaustive fret search with a common-shape prior'
description: Score every per-string fret (0-12 or muted) whose fundamental is a strong peak by explained spectral peak weight, instead of a fixed voicing dictionary (misses uncommon voicings) or per-string poly estimation (built for near-open strings). Identical-sounding fingerings are broken by small penalties (muted, high frets) and a bonus for COMMON_UKULELE_SHAPES, since audio can't tell them apart.
date: "2026-11-20"
issue: 01M7W5N1YTVBFRPT1PR25TQNJG
scope:
  - crates/tuner-core/src/chord.rs
---


