See [Overview](/overview.md). Lives in `crates/tuner-core`; the WASM exports and the worklet are the audio pipeline issue's job.

- `pitch` module: `PitchDetector::new(PitchConfig)` preallocates all buffers; `detect(&mut self, frame: &[f32]) -> Option<PitchEstimate>` returns `frequency` (Hz) and `clarity` (0..1) and allocates nothing, because it will run on the audio thread.
- Default config: 2048-sample frames, 60–1000 Hz search range (detuned low E down to open high frets), an RMS silence gate and a minimum clarity.
- `tuning` module: A4 = 440 Hz equal temperament; `Note::from_frequency` (MIDI number, name, octave, cents); `Instrument::{Guitar, Ukulele}` with standard tunings (E2 A2 D3 G3 B3 E4; re-entrant G4 C4 E4 A4); `nearest_string` returns the target string and the cents offset.
- Tests generate signals in Rust (no fixtures): sines at every string frequency, harmonic-rich tones with a weak fundamental, additive noise, silence, and pure noise.
