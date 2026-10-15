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
confirmed_commit: e7dc5754fa285b7054afb660c6a8f6499b76642c
---

Moves microphone audio to the [Pitch core](/components/pitch-core.md) on the audio thread and delivers readings to the UI. Constraints that shaped it are in [WASM inside an AudioWorklet](/pitfalls/audioworklet-wasm.md). What the page does with readings is in [Single-string tuner](/features/single-string-tuner.md).

## Flow

1. `startTuner(events)` (`web/src/lib/audio/tuner-engine.ts`, called from a click) creates the `AudioContext`, gets the wasm bytes from `loadTunerBytes()` (fetched once, a copy per caller because posting may detach the buffer), opens the mic without processing, adds the worklet module and creates the node with `processorOptions: { wasmBytes, hop: 1024 }`. It returns `{ sampleRate, stop() }`; `describeStartError` turns failures (e.g. `NotAllowedError`) into user-facing text.
2. `TunerProcessor` (`tuner-processor.ts`) instantiates the module, wraps it in `WasmPitchDetector`, and feeds 128-sample render quanta into a `FrameAccumulator`.
3. `FrameAccumulator` (`frame-accumulator.ts`) emits the latest 2048 samples every 1024 (~21 ms at 48 kHz, ~47 readings/s); it never allocates after construction.
4. Each frame is analysed and posted over the port (`messages.ts`): `{ type: 'ready', frameSize }`, `{ type: 'pitch', reading }` with `reading: null` when nothing is detected, `{ type: 'error', message }`.

A `PitchReading` is `{ frequency, clarity, midi, cents }`. Readings are raw; smoothing belongs to the UI.

## WASM interface

`crates/tuner-wasm` exports, mirrored by `TunerExports` in `web/src/lib/wasm/tuner.ts`:

- Detector handle: `tuner_new(sampleRate)`, `tuner_frame_size`, `tuner_input` (pointer to the input frame in linear memory), `tuner_detect` (1 if found), `tuner_frequency`, `tuner_clarity`, `tuner_free`. Wrapped by `WasmPitchDetector`.
- Pure lookups: `tuner_version`, `tuner_note_midi`, `tuner_note_cents`, `tuner_cents_between`, and tunings by instrument id (0 guitar, 1 ukulele): `tuner_string_count`, `tuner_string_midi`, `tuner_string_frequency`, `tuner_nearest_string`. Wrapped by `Tunings`, which the page instantiates on the main thread from the same bytes.

## Testing

- Rust tests in `tuner-wasm` call the exports natively.
- Vitest: `frame-accumulator.spec.ts` (framing, ring wrap) and `tuner.spec.ts` (built wasm detects 110 Hz, tunings and nearest string; `pnpm test` builds the wasm first).
- Playwright (`just e2e`, `web/e2e/`): the production build in the installed Chrome (`channel: 'chrome'`). Chrome takes its fake-microphone file at launch, so `playwright.config.ts` defines one project per tone in `TONES` and tests opt in with a tag such as `@e2`. `e2e/wav.ts` writes the tones with a length holding a whole number of cycles so the looped file has no click.
