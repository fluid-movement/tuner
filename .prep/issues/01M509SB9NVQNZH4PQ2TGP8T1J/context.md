See [Overview](/overview.md), [Pitch core](/components/pitch-core.md), and the raw C-ABI decision on the scaffold issue 01M509SB56RNMRG2J47ZSYWVPD.

- `crates/tuner-wasm`: a boxed detector handle with `extern "C"` functions: create for a sample rate, frame size, pointer to the input frame in linear memory, detect, read frequency/clarity, free.
- `web/src/lib/audio/`: `tuner-processor.ts` (the AudioWorkletProcessor, bundled by Vite via `?worker&url`), a pure-TS frame accumulator (128-sample render quanta → 2048-sample frames every 1024 samples), and `tuner-engine.ts` on the main thread (getUserMedia, AudioContext, addModule, fetch wasm bytes and pass them in `processorOptions`, receive messages over the port).
- Worklet messages: `{ type: 'ready' }`, `{ type: 'pitch', frequency, clarity }` with `frequency: null` when nothing is detected, `{ type: 'error', message }`.
- The node's output goes through a zero-gain node to the destination so the graph is pulled in every browser.
- Tests: Rust tests call the exports natively; Vitest covers the accumulator and the wasm exports; a Playwright test runs the page in installed Chrome with a generated WAV as the fake microphone.
