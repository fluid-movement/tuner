---
type: overview
title: Tuner
description: 'Browser tuner for acoustic guitar and ukulele: Rust/WASM pitch core in an AudioWorklet, SvelteKit 3 PWA'
status: draft
generated:
  by: claude-code/opus-5.5
  at: 2026-10-15T17:29:15Z
---

A browser-based tuner for stringed instruments, mainly acoustic guitar and ukulele. Pitch analysis runs in a Rust core compiled to WebAssembly so the DSP can go beyond a plain JS tuner. Standard tunings only for now: guitar E2 A2 D3 G3 B3 E4, ukulele re-entrant G4 C4 E4 A4.

## Architecture

- `crates/tuner-core`: pure Rust DSP with no web dependencies, tested natively with synthetic signals. See [Pitch core](/components/pitch-core.md) and [Pitch, notes and tunings](/conventions/pitch-and-tunings.md).
- `crates/tuner-wasm`: thin WASM wrapper around the core.
- `web/`: SvelteKit 3 app (Vite, static adapter, pnpm), shipped as an installable offline PWA.

Audio flows microphone → `getUserMedia` → AudioWorkletProcessor hosting the WASM module (analysis off the main thread) → results posted over the worklet port → Svelte UI.

## Roadmap

1. Iteration 1: single-string tuner (nearest string, cents deviation).
2. Polyphonic strum check: strum all strings, see per string which are in tune.
3. Automatic instrument detection (guitar vs ukulele) from the frequencies present.

The roadmap is tracked as prep issues; feature entries are written as features land.
