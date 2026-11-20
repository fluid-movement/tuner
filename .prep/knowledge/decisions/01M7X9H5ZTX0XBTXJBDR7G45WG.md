---
type: decision
title: tuner-core builds with opt-level s
description: 'Spectral flux onsets pushed tuner.wasm to 162.3 KB / 60.8 KB gz, over the 160/60 KB budget. Building tuner-core with opt-level "s" gives 154.7 / 58.6 KB with no measurable slowdown (chord_eval 2.0 -> 2.3 ms, poly_eval guitar 11.5 -> 9.8 ms per analysis). Alternative considered: hand-shrinking the flux code (a few KB at most).'
date: "2026-11-20"
issue: 01M7X8P8E9KFN53VNQ8F1SFEE8
scope:
  - Cargo.toml
---


