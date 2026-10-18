---
type: convention
title: Testing
description: Test layers, generated signals, diagnostic examples and local/CI commands
status: draft
generated:
  by: codex/implementer
  at: 2026-10-18T09:21:42Z
scope:
  - crates/*/src/**/*.rs
  - crates/*/tests/**/*.rs
  - crates/tuner-core/examples/*.rs
  - web/src/**/*.spec.ts
  - web/e2e/*.ts
  - web/playwright.config.ts
  - web/vite.config.ts
  - web/package.json
  - justfile
  - .github/workflows/ci.yml
  - README.md
---

Run commands below from the repository root unless a command explicitly changes into web. `just setup` installs the wasm32 target and frozen pnpm dependencies. Rust, Node 24, pnpm, just and installed Google Chrome are the README prerequisites. Tests use generated signals; synthetic success is not a guarantee for real microphones or instruments.

## Rust unit and integration tests

`cargo test` runs the workspace unit, integration and documentation tests; `just test` adds setup and the web Vitest suite. Target one crate with `cargo test -p tuner-core`, `cargo test -p tuner-wasm` or `cargo test -p tuner-lab`.

Core unit tests in `crates/tuner-core/src` cover FFT against a naive DFT, sine bins and size validation; pitch detection for standard/detuned strings, weak fundamentals, noise and invalid frames; note/cents/tuning mapping; deterministic synthesis; tracker gate/lock/follow/hold behavior, quiet plucks, changes, noise and octave slips; polyphonic collisions, spectrum/kernel accuracy, detuning and muted strings; and streaming strum reports, latency and noise rejection. See [Pitch core](/components/pitch-core.md), [Note tracker](/components/note-tracker.md) and [Polyphonic string estimation](/components/poly-estimation.md).

`cargo test -p tuner-core --test no_alloc` runs the dedicated allocation integration tests in `crates/tuner-core/tests/no_alloc.rs`. A thread-local counter in a custom global allocator checks that PitchDetector::detect on tone/silence and MonoTuner::process through lock/release allocate zero times after construction/preparation. This checks these mono processing paths, not all DSP or construction.

Native unit tests in `crates/tuner-wasm/src/lib.rs` exercise the C ABI: version, frequency-to-note mapping, tunings, mono tracking/holding/reset and streaming strum exports. These complement the web tests loading the actual wasm32 module.

`cargo test -p tuner-lab --test synthetic_takes` writes generated takes to temporary WAV files and analyzes them through the recording path. It covers guitar and ukulele detuning, event order/onset timing, per-pluck string identity/accuracy/display latency/coverage, strum agreement/presence/accuracy, stereo 24-bit 44.1 kHz mono mixing, and rejection of filenames without an instrument name. This fixture suite uses WAV; it does not assert FLAC decoding. See [Recording analysis](/components/tuner-lab.md).

## Synthetic generators and diagnostic examples

`crates/tuner-core/src/synth.rs` provides deterministic PCG32 randomness, plucked strings with inharmonic partials, spectral rolloff/pluck position, decay, polarization beating, noise, highpass filtering and randomized strums. Run its unit tests with `cargo test -p tuner-core synth::tests`. `crates/tuner-lab/src/demo.rs` assembles plucks then repeated strums with known truth: `just analyze-demo` writes example takes to recordings/demo, and `just analyze recordings/demo` analyzes them. These are native, local diagnostics.

The examples print metrics rather than act as pass/fail regression assertions; they are not executed by `just check`:

- `cargo run --release -p tuner-core --example poly_eval -- [method] [count] [scenario-filter]`: comb/resolved/all (default all), count default 200, substring filter default empty. Compares guitar/ukulele synthetic strum errors, misses, wrong tuning verdicts, timings and muted/played levels across detuning/noise/window/beating/muting scenarios. Set POLY_DEBUG=1 for errors over 5 cents.
- `cargo run --release -p tuner-core --example presence_eval -- [count]`: default 200; reports played/muted prominence distributions with and without a previous ringing strum, and a 20 dB clean-partial threshold diagnostic. Guitar B3/E4 lack clean partials and are not judged by that threshold.
- `cargo run --release -p tuner-core --example track_eval`: reports display coverage over the first two seconds, maximum shown pitch error and first-reading latency for every guitar/ukulele string at several signal/noise levels with a phone-like highpass. No argument parser.

Bracketed arguments above are optional placeholders, not literal shell arguments. These examples generate local signals and are distinct from external-service live evaluations.

## Vitest

`cd web && pnpm test` builds WASM through pretest and runs Vitest once. `just test` includes this after Rust tests. `web/vite.config.ts` defines one Node project named server, includes src/**/*.{test,spec}.{js,ts}, excludes Svelte component test files and requires assertions. There is currently no browser/component Vitest project.

The three spec files cover FrameAccumulator ordering/overlap/ring wrapping and invalid hops; loading web/static/wasm/tuner.wasm and calling exports, mono tracking/holding/reset, note names and standard/nearest tunings; and InTuneLatch entry/exit hysteresis and reset. See [Audio pipeline](/components/audio-pipeline.md). For watch mode, first run `cd web && pnpm build:wasm && pnpm test:watch`; test:watch itself has no WASM pre-hook.

## Playwright fake microphone

`E2E_PORT=4202 just e2e` runs local browser tests with setup. Equivalently, after setup, `cd web && E2E_PORT=4202 pnpm test:e2e`. `web/playwright.config.ts` uses installed Chrome (channel chrome), builds the site and launches `pnpm preview --port <port> --strictPort`; default port is 4173. Give each worktree its own E2E_PORT.

At configuration time `web/e2e/wav.ts` writes mono 16-bit WAV fixtures under web/test-results. Each project launches Chrome with fake device/UI flags and its own looping audio file, and selects tagged tests with an exact tag boundary. Projects are a2 (110 Hz), e2 (82.40689 Hz), a2-flat (-10 cents), c4 (261.62557 Hz), pluck (quiet decaying A2 repeated every 3 seconds), strum-guitar (mixed detuning) and strum-uke-muted (mixed detuning plus muted A4). Tones use harmonic sums and near-seamless loop lengths; plucks/strums use decaying harmonics, with strummed strings offset by 15 ms. These browser fixtures are separate from the Rust synth generator.

`pipeline.test.ts` covers Start/Stop, the microphone→AudioWorklet/WASM→UI reading and denied permission. `tuner.test.ts` covers standard/flat notes and selected strings, ukulele, persisted instrument, 375 px viewport (including a screenshot), quiet-pluck display continuity, level and debug output. `strum.test.ts` checks per-string verdicts/cents including muted presence and persisted mode/instrument.

To target one project: `cd web && E2E_PORT=4202 pnpm test:e2e --project=pluck`. For an already running deployment, `cd web && E2E_BASE_URL=https://your-host pnpm exec playwright test` skips the local build/preview server and points baseURL at that host; fake-mic projects remain active. This is an explicit external-target mode, not part of local validation. In CI's production deploy job it runs after the live version matches the commit; see [Deployment](/conventions/deployment.md).

## Aggregate checks and CI

`E2E_PORT=4202 just check` runs lint, test, build and e2e: cargo fmt --check, cargo clippy --all-targets -- -D warnings, Svelte checking, Prettier/ESLint, workspace Rust tests, Vitest, production build and Playwright. `.github/workflows/ci.yml` runs `just check` for pushes to main and pull requests on ubuntu-24.04 using installed Chrome, and uploads web/test-results on failure. Standalone `just lint`, `just test`, `just build` and `E2E_PORT=4202 just e2e` run the respective layers. `prep check` validates the knowledge/issue records separately from application checks.
