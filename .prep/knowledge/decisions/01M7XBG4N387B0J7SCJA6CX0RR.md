---
type: decision
title: 'Chord mode: reject voices, glides and single pitches before naming'
description: 'User reported chords shown with nothing played. Beyond the confidence gate, detect_chord now also rejects: a strong peak below the instrument''s lowest note (voices), readings whose notes are all one pitch''s overtones (hummed or sung notes, single plucked strings), fundamentals wider than a steady string''s peak (glides), and fundamentals less than 20 dB above local background; the listener requires -55 dBFS for an onset and stage agreement. A single plucked note no longer shows in chord mode (chord mode is for chords). Alternatives: a higher onset threshold alone would also drop quiet real strums.'
date: "2026-11-20"
issue: 01M7XAFZMQ2EBTBQ8K65PD9P86
scope:
  - crates/tuner-core/src/chord.rs
---


