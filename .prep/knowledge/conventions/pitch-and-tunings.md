---
type: convention
title: Pitch, notes and tunings
description: A4=440 equal temperament, cents sign convention, note naming, and the standard guitar/ukulele tuning tables in tuner-core::tuning
status: draft
generated:
  by: claude-code/opus-5.5
  at: 2026-10-15T17:42:57Z
scope:
  - crates/tuner-core/src/tuning.rs
confirmed_commit: e7dc5754fa285b7054afb660c6a8f6499b76642c
---

`crates/tuner-core/src/tuning.rs` holds everything about notes and tunings.

- 12-tone equal temperament with A4 = 440 Hz (`A4_FREQUENCY`); MIDI numbering, A4 = 69.
- Cents: `cents_between(frequency, reference) = 1200·log2(frequency/reference)`. Positive means sharp, negative flat.
- `Note::from_frequency` rounds to the nearest MIDI note; `cents` is in −50..+50. Names use sharps (`C#`, never `Db`), octaves use scientific pitch notation (middle C = C4).
- `Instrument::strings()` returns standard tuning in string order, so string indices match how players number pegs on screen:

| Instrument | Strings (index 0 first) |
|---|---|
| Guitar | E2 82.41, A2 110.00, D3 146.83, G3 196.00, B3 246.94, E4 329.63 |
| Ukulele (re-entrant) | G4 392.00, C4 261.63, E4 329.63, A4 440.00 |

- `Instrument::nearest_string(frequency)` compares by cents, not Hz, and returns `(index, target, cents)`.
- E4 exists on both instruments, so a single E4 cannot identify the instrument.
- Only standard tunings exist so far; alternate tunings will extend `Instrument`/tuning tables rather than replace them.
