---
type: component
title: Audio pipeline
description: 'Mic → AudioWorklet → tuner WASM → main thread: tuner-wasm C ABI, FrameAccumulator framing, worklet message protocol, startTuner API, and how it is tested'
status: draft
generated:
  by: claude-code/opus-5.5
  at: 2026-10-15T17:48:17Z
scope:
  - crates/tuner-wasm
  - web/src/lib/audio
  - web/src/lib/wasm
  - web/e2e
  - web/playwright.config.ts
---

Moves microphone audio to the [Pitch core](/components/pitch-core.md) on the audio thread and delivers readings to the UI. Constraints that shaped it are in [WASM inside an AudioWorklet](/pitfalls/audioworklet-wasm.md).

## Flow

1. `startTuner(events)` (`web/src/lib/audio/tuner-engine.ts`, called from a click) creates the `AudioContext`, fetches `tuner.wasm`, opens the mic without processing, adds the worklet module and creates the node with `processorOptions: { wasmBytes, hop: 1024 }`. It returns `{ sampleRate, stop() }`; `describeStartError` turns failures (e.g. `NotAllowedError`) into user-facing text.
2. `TunerProcessor` (`tuner-processor.ts`) instantiates the module, wraps it in `WasmPitchDetector`, and feeds 128-sample render quanta into a `FrameAccumulator`.
3. `FrameAccumulator` (`frame-accumulator.ts`) emits the latest 2048 samples every 1024 (~21 ms at 48 kHz, ~47 readings/s); it never allocates after construction.
4. Each frame is analysed and posted over the port (`messages.ts`): `{ type: 'ready', frameSize }`, `{ type: 'pitch', reading }` with `reading: null` when nothing is detected, `{ type: 'error', message }`.

A `PitchReading` is `{ frequency, clarity, midi, cents }`; note naming is `noteName(midi)` in `web/src/lib/wasm/tuner.ts`.

## WASM interface

`crates/tuner-wasm` exports `tuner_version`, `tuner_note_midi(f)`, `tuner_note_cents(f)`, and a handle API: `tuner_new(sampleRate)`, `tuner_frame_size`, `tuner_input` (pointer to the input frame in linear memory), `tuner_detect` (1 if found), `tuner_frequency`, `tuner_clarity`, `tuner_free`. `TunerExports` in `web/src/lib/wasm/tuner.ts` must match. Readings are not smoothed here; smoothing belongs to the UI.

## Testing

- Rust tests in `tuner-wasm` call the exports natively.
- Vitest: `frame-accumulator.spec.ts` (framing, ring wrap) and `tuner.spec.ts` (the built wasm detects 110 Hz; `pnpm test` builds the wasm first).
- Playwright (`pnpm test:e2e`, `web/e2e/`): runs the production build in the installed Chrome (`channel: 'chrome'`) with a fake microphone playing a generated 110 Hz WAV (`e2e/wav.ts`), and checks the page shows A2; a second test stubs `getUserMedia` to check the denied-permission message.
