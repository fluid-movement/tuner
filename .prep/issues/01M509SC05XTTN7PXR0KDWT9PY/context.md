See [Polyphonic string estimation](/components/poly-estimation.md) (findings of 01M509SBVQFQ26T79APJR7NNPK), [Note tracker](/components/note-tracker.md), [Audio pipeline](/components/audio-pipeline.md), [Recording analysis](/components/tuner-lab.md).

Rust (tuner-core):
- `strum` module: `StrumListener` fed with audio chunks; keeps ~1.2 s of history; detects strum onsets from the in-band envelope (the tuner-lab rule: rise ≥ 8 dB within 100 ms, ≥ 15 dB above background, ≥ 3 dB above the level 100–300 ms earlier; onsets within 80 ms merge); once 0.12 s + one analysis window has passed after an onset, runs `poly::estimate_resolved` and emits a `StrumReport` (per string: present?, cents, level).
- Presence detection (research recommendation 4): a string counts as played only if its energy rose at the strum onset (clean partials, or the resolved unknown's explained energy for strings without clean partials). Measure on the poly_eval "one string muted" scenario.
- Replace `rustfft` with a small iterative radix-2 FFT (recommendation 5) to bring tuner.wasm back near 40–60 KB; verify against a naive DFT and that poly_eval numbers are unchanged.

Web:
- A dedicated Web Worker hosts its own wasm instance with a `StrumListener`; the worklet sends it 1024-sample chunks over a `MessageChannel` port (main thread only wires the ports), so the audio thread never runs the ~8 ms analysis (recommendation 2).
- Mode switch (String / Strum) next to the instrument toggle, remembered like the instrument. Strum view: one row per string (name, in tune / flat / sharp / not heard, cents, small offset bar), the time since the last strum, and a prompt to strum.
- Playwright: a fake-mic WAV of repeated synthetic strums with known detunes (e.g. guitar E4 −12, B3 +9, rest in tune) checks the verdicts; one with a muted string checks "not heard".
