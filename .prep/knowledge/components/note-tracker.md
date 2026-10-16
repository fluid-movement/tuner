---
type: component
title: Note tracker
description: 'tuner-core NoteTracker and MonoTuner: adaptive noise floor, confirmed onset or steady-pitch lock, follow with octave folding, retarget, hold; band narrowing; behaviour numbers and pitfalls'
status: draft
generated:
  by: claude-code/opus-5.5
  at: 2026-10-16T09:13:13Z
scope:
  - crates/tuner-core/src/tracker.rs
  - crates/tuner-core/examples/track_eval.rs
---

`crates/tuner-core/src/tracker.rs` turns per-frame [Pitch core](/components/pitch-core.md) estimates into what the display shows. It exists because a decaying string gets quieter and less periodic within a second: with fixed thresholds, plucks at realistic phone levels were shown for only 25–60% of their first two seconds (user feedback: "hard to register a string, the dial disappears quickly"). It is meant to be reused per string by the strum mode.

## API

- `MonoTuner::new(sample_rate, hop)`: a `PitchDetector` with `PitchConfig::permissive` plus a `NoteTracker`; `process(frame) -> TrackerOutput` once per hop; `frame_size()`, `reset()`. This is what the WASM handle runs.
- `NoteTracker::new(TrackerConfig::new(frame_seconds))`, `push(estimate, level) -> TrackerOutput`, `reseed(frequency)`, `reset()`.
- `TrackerOutput { state: Idle | Tracking | Holding, locked, frequency (smoothed; Some while tracking or holding), raw_frequency, clarity, level_db, floor_db }`.
- Allocation-free after construction.

## Behaviour (defaults)

- **Noise floor** (dB): initialised from the first frame if it is unpitched, else −90 dBFS. Falls fast (halfway per frame) when the level drops below it. While nothing is tracked it rises with τ 0.5 s on unpitched frames and 4 s on pitched ones. Never rises while tracking. On every lock it is pushed to at least 12 dB below the note.
- **Onset lock**: clarity ≥ 0.85 and level ≥ floor + 12 dB on 2 consecutive frames at the same pitch (±35 cents). The first frame usually straddles the attack.
- **Steady lock** (only from Idle, not Holding): clarity ≥ 0.95 at ≥ −60 dBFS for 0.15 s at a stable pitch, for notes without an onset (already sounding at start, bowed, reference tones). Holding is excluded because it would relock on the biased tail of the note just released.
- **Follow**: clarity ≥ 0.6, level ≥ floor + 10 dB, within ±35 cents of the tracked pitch after folding octave slips. The 10 dB margin stops the tracker chasing noisy tail readings: it releases and holds instead.
- **Retarget**: 2 consecutive onset-quality frames elsewhere lock the new note (switching strings).
- **Release** after 0.25 s without a followable frame, into **Holding** for 3 s (the display keeps the last value, dimmed), then Idle.
- **Smoothing**: median of 5 frames in cents, then EMA τ 0.12 s; reset on every lock.
- **Band narrowing** (`MonoTuner`): while tracking or holding, the detector's low-pass is 4.5× the tracked frequency (350 Hz … 1.5 kHz). On a lock the same frame is analysed again in the narrow band and `reseed` replaces the first reading, so stiff-string upper partials don't bias low strings sharp.

## Measured (`cargo run --release -p tuner-core --example track_eval`)

Synthetic plucks on every guitar and ukulele string through a phone-like 120 Hz low cut, from loud (−20 dBFS peak, −60 dBFS noise) to very quiet in a noisy room (−40 / −50): a reading is shown for ≥ 97% of the first 2 s (the rest is the lock delay), the worst shown error is ≤ 2.1 cents, the first reading arrives 49–71 ms after the pluck. Unit tests cover quiet-pluck coverage, noise never locking, lock latency, retargeting within 150 ms, hold timing, octave folding, a note already sounding at start, and steady-pitch locking.

## Pitfalls

- Gate on `PitchDetector::level()` (in-band), not broadband RMS: white noise spread over 24 kHz hides a string whose energy sits below 1.5 kHz.
- Never initialise the floor from a confidently pitched first frame, and lower it on lock; otherwise a note that is sounding when listening starts is treated as background forever.
- Synthetic results are optimistic; tune thresholds against real recordings (recording-analysis issue).
