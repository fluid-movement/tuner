# components

## Entries

* [Audio pipeline](/components/audio-pipeline.md) - Mic → AudioWorklet → tuner WASM → main thread: tuner-wasm C ABI, FrameAccumulator framing, worklet message protocol, startTuner API, and how it is tested
* [Note tracker](/components/note-tracker.md) - tuner-core NoteTracker and MonoTuner: adaptive noise floor, confirmed onset or steady-pitch lock, follow with octave folding, retarget, hold; band narrowing; behaviour numbers and pitfalls
* [Pitch core](/components/pitch-core.md) - tuner-core PitchDetector: MPM pitch detection with low-pass prefilter and least-squares peak fit; API, defaults, accuracy limits and cost
* [Polyphonic string estimation](/components/poly-estimation.md) - tuner-core poly (per-string pitch from one strum: comb for clean strings, band least-squares for colliding ones) and synth (synthetic strums) plus the poly_eval harness; accuracy, cost, pitfalls
* [Recording analysis (tuner-lab)](/components/tuner-lab.md) - Native crate and CLI that splits real guitar/ukulele takes into plucks and strums, reports single-string experience and checks strum estimates against pluck truth; recording protocol, metrics, tests
* [UI](/components/ui.md) - Svelte 5 components (CentsMeter, StringRow, InstrumentToggle), page state flow, CSS tokens and dark mode, layout rules
