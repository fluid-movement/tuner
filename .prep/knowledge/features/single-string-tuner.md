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
---

The main page (`web/src/routes/+page.svelte`) tunes one string at a time.

## Behaviour

- The user picks Guitar or Ukulele (standard tuning). The choice is stored in `localStorage` under `tuner.instrument`; storage failures are ignored.
- **Start** (a user gesture) starts the [Audio pipeline](/components/audio-pipeline.md); **Stop** releases the microphone.
- Each reading is smoothed, then mapped to the nearest string of the instrument by pitch (`Tunings.nearest`, backed by `tuner-core`, see [Pitch, notes and tunings](/conventions/pitch-and-tunings.md)). The page shows the detected note, the target string highlighted in the string row, the offset in cents from that string on the meter, and the smoothed frequency.
- Status text: "Press Start and play a string", "Play a string", "In tune", "Flat · tune up", "Sharp · tune down". The meter clamps at ±50 cents (the needle dims when pegged).

## Smoothing and in-tune rule

`web/src/lib/tuner/smoother.ts`:

- `PitchSmoother` works in cents: median of the last 5 readings (rejects outliers such as octave slips in the decay), then an exponential moving average with a 120 ms time constant. If the median moves more than 40 cents from the smoothed value it restarts there, so a new string shows within about three readings (~60 ms). Without a reading for 250 ms the display clears.
- `InTuneLatch`: in tune when |cents| ≤ 5, out again only beyond 7, so the indicator does not flicker.

These are presentation choices; the polyphonic strum mode will need its own.

## Tests

Unit tests in `smoother.spec.ts`. Playwright (`e2e/tuner.test.ts`) uses fake-mic tones: in-tune E2 on guitar, A2 10 cents flat, C4 on ukulele, instrument persistence across reload, and a 375 px layout check that saves `test-results/phone.png`. Tests poll until the smoothed display converges rather than reading the first value.
