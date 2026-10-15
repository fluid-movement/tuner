# components

## Entries

* [Audio pipeline](/components/audio-pipeline.md) - Mic → AudioWorklet → tuner WASM → main thread: tuner-wasm C ABI, FrameAccumulator framing, worklet message protocol, startTuner API, and how it is tested
* [Pitch core](/components/pitch-core.md) - tuner-core PitchDetector: MPM pitch detection with low-pass prefilter and least-squares peak fit; API, defaults, accuracy limits and cost
