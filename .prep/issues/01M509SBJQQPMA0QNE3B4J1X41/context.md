See [Overview](/overview.md), [Audio pipeline](/components/audio-pipeline.md), [Pitch, notes and tunings](/conventions/pitch-and-tunings.md).

- `crates/tuner-wasm`: add tuning exports (`tuner_string_count`, `tuner_string_midi`, `tuner_string_frequency`, `tuner_nearest_string`, `tuner_cents_between`) so string tables and nearest-string logic stay in Rust. The main thread instantiates its own copy of the module (same fetched bytes) for these pure lookups.
- `web/src/lib/tuner/`: a smoother (median of recent frames in cents, then exponential smoothing; resets on a jump of more than ~40 cents or after ~250 ms without a reading) and the in-tune rule (|cents| ≤ 5, with hysteresis so it doesn't flicker), both unit-tested.
- `web/src/lib/components/`: cents meter (SVG needle, −50..+50), note display, string row highlighting the nearest string, instrument toggle (persisted to localStorage, wrapped in try/catch).
- Playwright: fake-mic tones for guitar and ukulele strings, including a detuned one, check string, direction and in-tune state.
