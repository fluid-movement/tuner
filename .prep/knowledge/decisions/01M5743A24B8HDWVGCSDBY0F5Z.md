---
type: decision
title: 'WASM size budget: 160 KB raw, 60 KB gzipped'
description: The strum analysis makes tuner.wasm 147 KB stripped (55 KB gzipped) instead of the guessed 80 KB; accepted, further cuts need sort/fmt hand-tuning
date: "2026-10-18"
issue: 01M509SC05XTTN7PXR0KDWT9PY
scope:
  - Cargo.toml
  - crates/tuner-wasm
---

With the strum listener exported, tuner.wasm is 163 KB, 147 KB with `strip = true` in the release profile (the name section was 16 KB), 55 KB gzipped as nginx serves it. Sections: code 133 KB, data 12 KB. opt-level "s"/"z" made it larger (182/200 KB). The 80 KB criterion was a guess made before the strum code existed (baseline 39 KB single-string, 337 KB with rustfft). The remaining bulk is the band fit, per-type sort instantiations, the wasm libm and panic formatting; cutting it means hand-tuning those for diminishing returns, so the budget is 160 KB raw / 60 KB gzipped.
