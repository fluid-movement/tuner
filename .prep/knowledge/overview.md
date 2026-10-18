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

The architecture choice is recorded in [Stack: SvelteKit + Rust/WASM in an AudioWorklet, static PWA](/decisions/01M4ZPXYG12WVJMAHKVGWTN0ME.md); the PWA part remains planned.

- `crates/tuner-core`: pure Rust DSP, tested natively with synthetic signals. See [Pitch core](/components/pitch-core.md), [Note tracker](/components/note-tracker.md) (what the display shows over a pluck's life), [Pitch, notes and tunings](/conventions/pitch-and-tunings.md) and, for strums, [Polyphonic string estimation](/components/poly-estimation.md). Depends only on `num-complex`; the FFT is implemented in the core.
- `crates/tuner-wasm`: raw C-ABI WASM exports of the core, with no JS imports.
- `crates/tuner-lab`: native CLI that checks the core against real recordings (`just analyze`). See [Recording analysis](/components/tuner-lab.md).
- `web/`: SvelteKit 3 app (Vite, static adapter, pnpm), built as a prerendered static site; installable/offline PWA support is planned. See [UI](/components/ui.md) and [Single-string tuner](/features/single-string-tuner.md).

Audio flows microphone → `getUserMedia` → AudioWorkletProcessor hosting the WASM module (detection and note tracking off the main thread) → results posted over the worklet port → Svelte UI. See [Audio pipeline](/components/audio-pipeline.md) and [WASM inside an AudioWorklet](/pitfalls/audioworklet-wasm.md).

## Running

The root `justfile` is the entry point: `just dev` installs the wasm target and web dependencies, builds the wasm and starts Vite; `just check` runs fmt/clippy, svelte-check/eslint/prettier, Rust and Vitest unit tests, the production build and the Playwright browser tests. `just --list` shows the rest. Production is a Docker image deployed on Coolify; see [Deployment](/conventions/deployment.md).

See [Build and tooling](/conventions/build-and-tooling.md) for workspace, versions and build paths, and [Testing](/conventions/testing.md) for test layers, diagnostic examples and commands.

## Roadmap

1. Iteration 1: single-string tuner (nearest string, cents deviation).
2. Polyphonic strum check: strum all strings, see per string which are in tune.
3. Automatic instrument detection (guitar vs ukulele) from the frequencies present.

The roadmap is tracked as prep issues; feature entries are written as features land.
