---
type: pitfall
title: WASM inside an AudioWorklet
description: Why tuner-wasm has no wasm-bindgen, how the bytes reach the worklet, bundling the processor with Vite, user-gesture and mic constraints
status: draft
generated:
  by: claude-code/opus-5.5
  at: 2026-10-15T17:48:17Z
scope:
  - crates/tuner-wasm
  - web/src/lib/audio
confirmed_commit: 37831a0c45ca4b5b67e0dfb5845aa8962c438d95
---

Running WASM on the audio thread has constraints that are easy to trip over.

- **No fetch, no TextDecoder.** The AudioWorkletGlobalScope lacks them, and wasm-bindgen's JS glue relies on them. So `crates/tuner-wasm` exposes plain `extern "C"` functions and the module has zero imports. Don't add wasm-bindgen or anything that pulls in JS imports (e.g. `getrandom` with the `js` feature, `web-sys`); check with `WebAssembly.Module.imports`.
- **Bytes travel in `processorOptions`.** The main thread fetches `static/wasm/tuner.wasm` and passes the `ArrayBuffer` when creating the `AudioWorkletNode`; the processor instantiates it asynchronously and posts `ready`. `process()` ignores input until then.
- **Memory views go stale.** Linear memory can grow when Rust allocates, which detaches earlier `Float32Array`s. `WasmPitchDetector.detect` creates its input view on every call.
- **The processor must be self-contained.** `tuner-processor.ts` is imported with `?worker&url`, so Vite bundles it and its imports into one IIFE chunk. It may only import plain TS (no `$app/*`, no DOM, no Svelte). Check `build/_app/immutable/workers/` contains no `import`.
- **User gesture.** Create the `AudioContext` and call `getUserMedia` inside a click handler (`startTuner` does both); browsers block either otherwise.
- **Raw mic signal.** Request `echoCancellation`, `noiseSuppression` and `autoGainControl` off; they distort sustained tones.
- **Keep the graph pulled.** The node's output goes through a zero-gain `GainNode` to the destination; a dangling node may not be processed.
- **Panics trap.** Release builds use `panic = "abort"`, so a Rust panic becomes a WASM trap that kills the processor. Only pass frames of `tuner_frame_size` samples.
