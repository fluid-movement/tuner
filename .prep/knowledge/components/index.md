# components

## Entries

* [Audio pipeline](/components/audio-pipeline.md) - Mic → AudioWorklet → tuner WASM → main thread: tuner-wasm C ABI, FrameAccumulator framing, worklet message protocol, startTuner API, and how it is tested
* [Pitch core](/components/pitch-core.md) - tuner-core PitchDetector: MPM pitch detection with low-pass prefilter and least-squares peak fit; API, defaults, accuracy limits and cost
* [Polyphonic string estimation](/components/poly-estimation.md) - tuner-core poly (per-string pitch from one strum: comb for clean strings, band least-squares for colliding ones) and synth (synthetic strums) plus the poly_eval harness; accuracy, cost, pitfalls
* [UI](/components/ui.md) - Svelte 5 components (CentsMeter, StringRow, InstrumentToggle), page state flow, CSS tokens and dark mode, layout rules
