**Question.** Given one strum of all strings of a guitar or ukulele in (roughly) standard tuning, how can we estimate each string's pitch well enough to say in tune / flat / sharp, within the AudioWorklet budget?

**Scope.** Known instrument and tuning (instrument detection is a later issue). Strings may be detuned by up to about ±50 cents. The answer is an algorithm, evidence from a synthetic evaluation harness, and a prototype in `tuner-core` that the strum-check issue can build on. UI and WASM wiring are out of scope.

**Known difficulties.**
- Harmonic collisions inside one instrument: on guitar E2's 3rd harmonic (247.2 Hz) sits on B3 (246.9), its 4th on E4 (329.6), and A2's 3rd (330.0) also on E4; every partial of E4 lies on E2's series. Ukulele is easier (re-entrant, narrow range) but C4·3 ≈ G5 and similar overlaps exist.
- Frequency resolution: 10 cents at 82 Hz is 0.5 Hz, so FFT bins need long windows or sub-bin estimation.
- Inharmonicity (partials slightly sharp), different decay rates per string and partial, strum onset spread (10–40 ms), unequal string levels, noise.
- Budget: analysis may run less often than mono mode (a few times per second), but must stay well under the 2.7 ms render quantum when spread out, or run in chunks.

**Answered when** findings name the chosen approach with measured accuracy on synthetic strums (per-string error distribution, miss rate, effect of the collisions above), its cost, its limits, and a recommendation for the strum-check UI (what precision to promise).
