---
type: feature
title: Single-string tuner
description: 'Iteration 1 behaviour: nearest string for the chosen instrument, cents meter, smoothing (median + EMA, reset, timeout), in-tune hysteresis, instrument persistence'
status: draft
generated:
  by: claude-code/opus-5.5
  at: 2026-10-15T17:58:39Z
scope:
  - web/src/routes
  - web/src/lib/tuner
  - web/e2e/tuner.test.ts
  - crates/tuner-core/src/tracker.rs
confirmed_commit: 80df2418afdd067977dd296d843a9c4155ca655e
---

The main page (`web/src/routes/+page.svelte`) tunes one string at a time.

## Behaviour

- The user picks Guitar or Ukulele (standard tuning). The choice is stored in `localStorage` under `tuner.instrument`; storage failures are ignored.
- **Start** (a user gesture) starts the [Audio pipeline](/components/audio-pipeline.md); **Stop** releases the microphone.
- Every ~21 ms the [Note tracker](/components/note-tracker.md) reports a state and a smoothed frequency. The page maps the frequency to the nearest string of the instrument by pitch (`Tunings.nearest`, backed by `tuner-core`, see [Pitch, notes and tunings](/conventions/pitch-and-tunings.md)) and shows the detected note, the target string highlighted in the string row, the offset in cents from that string on the meter, and the frequency.
- **Tracking**: the reading follows the ringing string. **Holding**: the string has faded; the last reading stays for about 3 s, dimmed, with a "Hold" badge, so it can be read after looking up from the peg. **Idle**: nothing shown.
- Status text: "Press Start and play a string", "Play a string", "In tune", "Flat · tune up", "Sharp · tune down" (+ "Hold" badge while holding). The meter clamps at ±50 cents (the needle dims when pegged).
- A level meter under the strings shows the input level above the estimated background noise, with a mark at the 12 dB a pluck needs to lock, so the user can see the microphone hears them.
- `?debug` in the URL shows the tracker's raw frequency, clarity, level, noise floor and state.

## In-tune rule

`web/src/lib/tuner/in-tune.ts` `InTuneLatch`: in tune when |cents| ≤ 5, out again only beyond 7, so the indicator does not flicker. Smoothing of the pitch itself happens in the tracker (Rust), not in the page.

## Tests

Unit tests in `in-tune.spec.ts`. Playwright (`e2e/tuner.test.ts`) uses fake-mic sounds: steady tones (in-tune E2 on guitar, A2 10 cents flat, C4 on ukulele) and the `pluck` project, a quiet (≈ −34 dBFS) decaying A2 pluck re-plucked every 3 s, which must stay on screen through its decay (also checks the level meter and `?debug`). Plus instrument persistence across reload and a 375 px layout check that saves `test-results/phone.png`. Tests poll until the smoothed display converges rather than reading the first value.
