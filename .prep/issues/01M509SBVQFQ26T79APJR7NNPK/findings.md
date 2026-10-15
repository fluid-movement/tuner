## Answer

Use **target-aware spectral analysis with collision resolution** ("resolved", `tuner_core::poly::estimate_resolved`):

1. One Blackman–Harris windowed FFT of a ~0.68 s frame (32768 samples at 48 kHz, zero-padded ×2), starting ~80 ms after the last string's onset.
2. For every string, list which of its first 8 partials collide (within 35 cents) with another string's partials at the target tuning (`clean_partials`, computed from the targets, so alternate tunings work automatically).
3. **Strings with clean partials** (guitar E2 A2 D3 G3; every ukulele string): scan ±60 cents for the best harmonic-comb score over the clean partials, refine each clean partial by log-parabolic peak interpolation, then fit fundamental *and inharmonicity B* jointly (`ln(p_n/n) ≈ ln f0 + B·n²/2`).
4. **Strings with no clean partial** (guitar B3 and E4 — every partial of E4 lies on E2's series, B3 sits on E2's 3rd harmonic): for partials 1–3, take the band of bins around them and fit the exact windowed-spectrum model (closed form for a damped exponential under the BH window) with the already-known strings' partials at positions predicted from their fitted f0 and B, plus one unknown component. Amplitudes come from (ridge-regularized) complex least squares; the unknown frequency minimises the residual, searched coarse (2 cents) then fine (0.25 cents) around the three best minima, over three damping values. The three partial estimates are combined by a weighted median, weighted by the energy each explains beyond the known partials.

## Evidence (synthetic strums, 300 per scenario)

Generator: `tuner_core::synth` — per-partial decay, stiff-string inharmonicity (guitar 2e-5..2e-4), pluck-position nulls, random levels (−12..0 dB), 8–35 ms onset spread, Gaussian noise. Harness: `cargo run --release -p tuner-core --example poly_eval [comb|resolved|all] [count] [filter]`. "Wrong verdict" = in tune (±5 cents) / flat / sharp differs from the truth, ignoring truths within 1.5 cents of the boundary.

Guitar, strings within ±10 cents (the realistic tuning situation):

| | E2–G3 p90 | B3 p90 / max | E4 p90 / max | B3 wrong | E4 wrong |
|---|---|---|---|---|---|
| comb only | ≤0.05 c | 4.0 / 9.7 c | 3.6 / 10.9 c | 19/209 (9%) | 18/211 (9%) |
| **resolved** | ≤0.04 c | **0.7 / 1.5 c** | **0.7 / 4.9 c** | **0/209** | **0/211** |

Resolved, harder conditions (guitar): noisy (≈20 dB lower SNR): B3/E4 p90 0.7 c, 0 wrong. Random detune ±50 c: p90 ≤0.4 c, max 2.4 c, 0 wrong. Short 0.34 s window: E4 p90 1.4 c, 2/198 wrong (max 15.5 c). Beating partials (two polarizations, second component +0.5..4 c at 40%): all strings p90 1.1–2.1 c (partly definitional — the perceived pitch moves), B3 3/219 and E4 5/209 wrong (max 30 c).

Ukulele: comb alone is enough (no string lacks clean partials); p90 ≤0.08 c in every scenario, 0 wrong except beating (≤1 c p90, 0 wrong).

## Cost

Native release: comb 1.3 ms, resolved 5.6 ms per analysis (guitar). WASM (release, Node): comb 2.8 ms, resolved 7.8 ms — after replacing `norm()` (software `hypot` in wasm) with `norm_sqr()` in the inner loops, which was a 9× slowdown. `rustfft` grows tuner.wasm from 39 KB to 337 KB.

## Limits and open problems

- **Synthetic only.** No body resonances, sympathetic strings, pick noise or real decay curves. Real recordings may behave worse, especially for B3/E4.
- **Beating partials** are the main remaining failure mode for B3/E4 (unmodelled second components push the unknown off).
- **String presence.** A muted/missed string still gets an estimate; its reported level reaches −5 dB on guitar (played strings go down to −29 dB), so level alone cannot flag it.
- **Window length** trades latency for accuracy; below ~0.5 s E4 outliers grow.
- Tried and rejected: kernel-derivative columns for known partials (to absorb their frequency/envelope error) — worse (they also absorb the unknown); amplitude-weighted combination — ill-conditioned near-coincident fits dominate; unregularized LS — opposite-phase blow-ups (60+ cent outliers).

## Recommendations for the strum-check issue (01M509SC05XTTN7PXR0KDWT9PY)

1. Promote `poly` as is (resolved for guitar; it reduces to comb for ukulele). Promise **per-string verdicts (in tune / flat / sharp at ±5 cents) and whole-cent readouts**; send the user to single-string mode for fine work on B3/E4 when they read within ~3 cents of the boundary.
2. Run it **off the audio thread**: the worklet forwards audio (postMessage batches, avoiding the COOP/COEP headers a SharedArrayBuffer would need on static hosting) to a Web Worker running its own wasm instance; analyse ~4 times a second.
3. Add **strum onset detection** to place the window (≥80 ms after the last onset, ~0.68 s long) and to reset results on a new strum.
4. Add **string presence detection** (onset energy at each string's clean partials, or harmonic consistency of the fitted peaks) before showing a verdict.
5. Replace rustfft with a small power-of-two FFT to keep the module small.
6. Add a dev-only **"record strum → download WAV"** button and build a small corpus of real guitar/ukulele strums as regression fixtures before tuning thresholds.
