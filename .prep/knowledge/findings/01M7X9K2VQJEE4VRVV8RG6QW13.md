---
type: finding
title: Muted string vs octave doubling is not separable
description: 'chord_stream_eval with one random string muted: 80% notes right. The misses are a muted A string read as A fret 3 (C5) in C or fret 5 (D5) in G: those notes'' partials are exactly the even partials of C4/D4 already sounding, so a muted string and the octave doubling (the real C chord 0003) produce the same spectrum. The detector prefers ''played'' (MUTED_PENALTY), which is right for the common chords; amplitude cues would be needed to do better.'
date: "2026-11-20"
issue: 01M7X8P8GDTF2CVYYGE7HPF4N6
scope:
  - crates/tuner-core/src/chord.rs
---


