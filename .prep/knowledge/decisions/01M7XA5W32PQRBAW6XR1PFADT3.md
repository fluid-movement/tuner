---
type: decision
title: 'Strum onsets: spectral flux alongside the envelope rule, with a slow noise floor'
description: The envelope rule (8 dB rise) misses re-strums over ringing strings; spectral flux (mean dB rise over 80 Hz-4 kHz vs 30 ms earlier, peak >= 4 dB and 1.5 dB over the recent median) catches them. Its loudness gate uses a 30 s noise floor because the 2 s onset background climbs to the music during continuous playing. FLUX_MIN 2-3 dB let partial beating trigger onsets; 4 dB did not. The envelope rule stays for strums from silence.
date: "2026-11-20"
issue: 01M7X8P8E9KFN53VNQ8F1SFEE8
scope:
  - crates/tuner-core/src/strum.rs
---


