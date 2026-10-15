---
type: component
title: Polyphonic string estimation
description: 'tuner-core poly (per-string pitch from one strum: comb for clean strings, band least-squares for colliding ones) and synth (synthetic strums) plus the poly_eval harness; accuracy, cost, pitfalls'
status: draft
generated:
  by: claude-code/opus-5.5
  at: 2026-10-15T18:29:42Z
scope:
  - crates/tuner-core/src/poly.rs
  - crates/tuner-core/src/synth.rs
  - crates/tuner-core/examples/poly_eval.rs
---

`crates/tuner-core/src/poly.rs` estimates every string's fundamental from one strum, given the instrument's tuning targets. It is not yet wired into WASM or the UI (that is the strum-check issue). Monophonic detection is the separate [Pitch core](/components/pitch-core.md).

## API

- `PolyConfig::new(sample_rate, instrument)`: ±60-cent search, 8 partials per string, default inharmonicity (guitar 8e-5, ukulele 2e-5), 35-cent collision threshold.
- `estimate_resolved(frame, instrument, &config) -> Vec<Option<StringEstimate>>`, in string order; `StringEstimate { frequency, cents, level_db, inharmonicity }`. Expects ~0.68 s (32768 samples at 48 kHz) starting after the strum's last onset. `estimate_comb` is the simpler first stage, kept for comparison.
- `clean_partials(targets, config)` lists, per string, which partials don't collide with other strings' partials. On standard guitar B3 and E4 have none (E4 lies on E2's harmonic series; B3 on E2·3); every ukulele string has some.

## How it works

1. `Spectrum`: Blackman–Harris window, ×2 zero padding, complex bins and dB.
2. Comb stage (clean strings): best harmonic-comb score over clean partials within ±60 cents, log-parabolic peak refinement per partial, then joint fit of f0 and B (`fit_partials`).
3. Resolve stage (strings without clean partials, lowest first): in a band around each of partials 1–3, fit the exact windowed damped-exponential spectrum (`kernel_row`, closed form via geometric series with a per-bin recurrence) of the known strings' predicted partials plus one unknown, by ridge-regularized complex least squares (`BandFit`); search the unknown frequency coarse-to-fine over three dampings; combine partials by weighted median, weighted by energy explained beyond the known partials.

## Accuracy and cost

Synthetic, guitar within ±10 cents: E2–G3 p90 ≤0.04 cents, B3/E4 p90 ≈0.7 cents with no wrong in-tune/flat/sharp verdicts in 300 strums; ukulele p90 ≤0.08 cents. Beating partials (string polarizations) are the main weakness for B3/E4. Cost: ~5.6 ms native, ~7.8 ms in WASM per analysis; too slow for the audio thread, fine in a Worker a few times per second. Full numbers and recommendations are in the findings of research issue 01M509SBVQFQ26T79APJR7NNPK.

## Pitfalls

- Use `norm_sqr()`, not `norm()`, in inner loops: `norm()` is `hypot`, which is a slow software routine in WASM (it made the resolver 9× slower there).
- Keep the ridge in `regularize`: without it near-coincident components get huge opposite-phase amplitudes and the unknown lands 60+ cents off. 1e-3 of the mean diagonal was best; 1e-2 hurts near-coincident cases.
- Weight candidates by explained energy, not amplitude, for the same reason.
- Giving known partials extra derivative columns was tried and made things worse.
- `rustfft` adds ~300 KB to the wasm module; a small radix-2 FFT would avoid that.

## Synthetic strums and evaluation

`crates/tuner-core/src/synth.rs`: `PluckedString` (inharmonic partials, per-partial decay, pluck-position nulls, optional polarization beating), `render`, `random_strum(instrument, &StrumOptions, &mut Rng)` and a deterministic PCG `Rng`. `cargo run --release -p tuner-core --example poly_eval [comb|resolved|all] [count] [scenario-filter]` prints per-string error quantiles, misses and wrong verdicts across scenarios (random ±50, near ±10, noisy, short window, beating partials, one string muted); `POLY_DEBUG=1` prints outlier strums.
