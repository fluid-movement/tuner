---
type: decision
title: The audio thread only measures the level
description: The single-string tuner is gone (2026-11-24), so single-note pitch detection is off by default in the worklet; setPitch stays in the engine API. Chord recognition and the strum check run in the strum worker.
date: "2026-11-24"
scope:
  - web/src/lib/audio/**
supersedes:
  - 01M7XCFJTX1WEP5RNE74KYDWGD
---


