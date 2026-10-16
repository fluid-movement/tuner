---
type: component
title: Recording analysis (tuner-lab)
description: Native crate and CLI that splits real guitar/ukulele takes into plucks and strums, reports single-string experience and checks strum estimates against pluck truth; recording protocol, metrics, tests
status: draft
generated:
  by: claude-code/opus-5.5
  at: 2026-10-16T10:29:36Z
scope:
  - crates/tuner-lab
  - justfile
---

`crates/tuner-lab` (native only, never built for wasm) checks tuner-core against real recordings. Dependencies: `hound` (WAV), `claxon` (FLAC), `serde`/`serde_json`; tuner-core stays dependency-light.

## Recording protocol

One tuning state per file: pluck each string alone, low to high, ~2 s each, then strum 3–4 times, ~3 s each, without touching the pegs. WAV or FLAC, mono or stereo (mixed to mono), 44.1/48 kHz, no noise suppression or auto gain. The file name starts with the instrument (`guitar`/`gtr`, `ukulele`/`uke`). Recordings live in the gitignored `recordings/`; which ones become committed fixtures is still open.

## Usage

`just analyze [DIR|FILE…] [--json]` (default `recordings/`; Markdown on stdout, per take plus a summary), `just analyze-demo [dir]` writes three synthetic takes (`tuner-lab demo`). With no files it prints the protocol.

## Pipeline

1. `segment::onsets`: 60–1500 Hz envelope in 10 ms hops; an onset rises ≥ 8 dB within 100 ms, is ≥ 15 dB above the take's background (10th percentile) and ≥ 3 dB above its level 100–300 ms earlier (otherwise a beating chord's dip-and-recover reads as a new strum). Onsets within 80 ms merge (a strum's strings); events are ≥ 0.25 s apart.
2. `segment::classify`: counts strings with clean partials that sound (comb level ≥ −20 dB re the strongest partial, from `poly::estimate_comb` 120 ms after the onset): a pluck makes at most one sound (B3/E4 plucks make none), a strum most; with exactly two, clarity ≥ 0.85 means pluck.
3. Plucks (`analyze::analyze_pluck`): a fresh `MonoTuner` from 0.5 s before the onset (as in real use, the previous string may still ring). Truth = median tracked frequency 0.2–1.5 s after the onset, assigned to the nearest string. Experience: first reading of that note (within 50 cents), share of the first 2 s showing it, spread (max − min) and drift (median 1.0–1.5 s minus 0.2–0.5 s) of the shown value.
4. Strums: `poly::estimate_resolved` on 32768 samples (16384 if the segment is short) from 120 ms after the onset; per string error vs the take's pluck truth and in-tune/flat/sharp agreement (±5 cents; truths within 1.5 cents of the boundary aren't judged).

## Tests

`tests/synthetic_takes.rs` renders protocol-shaped takes with `demo::synthetic_take(_at)` (known detunes, onset spread, noise, mic low cut), writes WAV (also stereo 24-bit 44.1 kHz) and runs `analyze_file`: exact event kinds and onsets, pluck truth within ±2 cents, ≥ 90% shown and ≤ 100 ms latency, strum error ≤ 3 cents and no disagreeing verdict. The first demo run found two tracker bugs (see [Note tracker](/components/note-tracker.md) pitfalls) and a segmentation bug.
