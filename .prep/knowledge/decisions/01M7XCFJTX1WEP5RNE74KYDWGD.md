---
type: decision
title: Audio thread runs pitch detection only in String mode
description: Chord and Strum modes only need the level for the meter, so the worklet calls MonoTuner::process_level (wasm tuner_set_pitch 0) and posts every other frame; this saves ~35 ms of analysis per audio second (native) on the audio thread. Switching back to String mode re-enables detection; a note already sounding locks via the steady path (0.15 s).
date: "2026-11-20"
issue: 01M7XAFZWM7YT3JAEFAWNGPXQV
scope:
  - web/src/lib/audio/tuner-processor.ts
---


