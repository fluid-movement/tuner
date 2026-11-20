---
type: finding
title: Peak prominence must be one-sided for semitone neighbours
description: 'chord.rs find_peaks compared a peak with the median of both sides; a note a semitone above another (F4 over E4 in Fmaj7 2413) had half its neighbourhood inside the lower note''s main lobe and was dropped, so Fmaj7 read as 5458 (F5) in ~1/3 of strums. Using the quieter side''s median fixed it: chord_eval notes 99.0 -> 100% at 16384, 90 -> 98% at 8192.'
date: "2026-11-20"
issue: 01M7X3VRM4DW3WXC572XN22PM6
scope:
  - crates/tuner-core/src/chord.rs
---


