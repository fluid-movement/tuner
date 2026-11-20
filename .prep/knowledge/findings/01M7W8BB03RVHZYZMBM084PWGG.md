---
type: finding
title: 'Strum window lengths: what each instrument needs'
description: 'poly_eval near ±10 (200 strums): ukulele pitch errors p90 <= 0.24 c even at 4096 samples (85 ms); guitar needs 16384 (8192: D3 29/146 wrong verdicts, E2 p90 1.9 c). presence_eval: ukulele presence reliable from 16384 (8192 misses 13/158 played G4; 4096 misses up to half), guitar validated at 32768. chord_eval (30 ukulele chords x 20, 48 kHz): chord name 97-98% at 8192, ~100% at 12288-16384; exact tab ~95% (misses are equivalent fingerings of the same notes, e.g. E 4442 vs 1402); ~2.5 ms native per chord analysis. All synthetic; real recordings still pending.'
date: "2026-11-20"
issue: 01M7W5N1W8MKMQC8GX957A6GVB
scope:
  - crates/tuner-core/src/{strum,chord}.rs
---


