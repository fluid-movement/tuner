## Project

A browser-based tuner for stringed instruments, mainly acoustic guitar and ukulele. Pitch analysis runs in a Rust core compiled to WebAssembly, so the DSP can go well beyond what a plain JS tuner does. Iteration 1 is a classic single-string tuner: one note at a time, nearest target string, cents deviation. The end goal is polyphonic: strum all strings at once and see which are in tune and which need tuning, with the instrument (guitar or ukulele) recognised automatically from the frequencies present. Standard tunings only for now (guitar E2 A2 D3 G3 B3 E4; ukulele re-entrant G4 C4 E4 A4); alternate tunings later.

Organisation (planned): a Cargo workspace with `crates/tuner-core` (pure Rust DSP, no web dependencies, tested natively with synthetic signals) and `crates/tuner-wasm` (thin wasm-bindgen/raw-ABI wrapper), plus `web/`, a SvelteKit app (Vite, static adapter) that is shipped as an installable offline PWA. Audio flows microphone → `getUserMedia` → AudioWorkletProcessor, which hosts the WASM module and analyses frames off the main thread → results posted over the worklet port → Svelte UI.

Build and test (planned): `cargo test` for the core; a wasm build step (wasm-pack or cargo + wasm-bindgen) emitting into the web app; `pnpm dev|build|test` in `web/`. Detection changes are also checked manually with a real instrument.

## Proposed entries

### Area A — Foundations (written with the bootstrap)

| type | path | title | description | scope |
|---|---|---|---|---|
| overview | /overview.md | Tuner | What the app is, its architecture and roadmap at a glance | `**` |
| decision | /decisions/stack.md | Stack choice | Why SvelteKit, Rust/WASM in an AudioWorklet and a static PWA | `Cargo.toml`, `web/package.json` |
| convention | /conventions/build-and-tooling.md | Build and tooling | Workspace layout, wasm build pipeline, dev/build/test commands | `Cargo.toml`, `crates/*/Cargo.toml`, `web/*.config.*`, `web/package.json` |
| convention | /conventions/testing.md | Testing | Synthetic-signal tests for DSP, frontend tests, manual instrument checks | `crates/**/tests/**`, `web/src/**/*.test.ts` |

### Area B — Audio and DSP core

| type | path | title | description | scope |
|---|---|---|---|---|
| convention | /conventions/pitch-and-tunings.md | Pitch, notes and tunings | A4=440 Hz, equal temperament, cents, note naming, tuning tables with target frequencies | `crates/tuner-core/src/tuning*` |
| component | /components/pitch-core.md | Pitch core | `tuner-core` API: frame in, pitch estimate (+ confidence) out; algorithm choice (e.g. MPM/YIN) | `crates/tuner-core/**` |
| component | /components/audio-pipeline.md | Audio pipeline | Mic capture, AudioWorklet hosting WASM, frame size, message protocol to the UI | `crates/tuner-wasm/**`, `web/src/lib/audio/**` |
| pitfall | /pitfalls/audioworklet-wasm.md | WASM inside AudioWorklet | No fetch/TextDecoder in the worklet scope; bytes must be sent in and instantiated there; mic permission and AudioContext resume on user gesture | `web/src/lib/audio/**` |

### Area C — App and product features

| type | path | title | description | scope |
|---|---|---|---|---|
| feature | /features/single-string-tuner.md | Single-string tuner | Nearest-string detection, cents display, in-tune threshold and smoothing | `web/src/routes/**`, `web/src/lib/tuner/**` |
| component | /components/ui.md | UI | Svelte component structure, needle/meter rendering, state stores | `web/src/lib/components/**` |
| feature | /features/pwa.md | PWA and offline | Manifest, service worker, caching the wasm and worklet files | `web/static/manifest*`, `web/src/service-worker*` |

Feature entries for the polyphonic strum check and automatic instrument detection are written when those features exist (entries describe current state only); until then the roadmap lives in prep issues.

## Proposed roadmap issues (after bootstrap)

1. Scaffold workspace: Cargo workspace, SvelteKit app, wasm build wired into Vite.
2. Audio pipeline: mic → AudioWorklet → WASM round trip with a stub analyser.
3. Single-pitch detection in `tuner-core` with synthetic-signal tests.
4. Single-string tuner UI (iteration 1 milestone).
5. PWA/offline.
6. Research: polyphonic analysis approach for strummed chords (spectral peaks + harmonic grouping vs. multi-pitch estimation; note E4 is shared by guitar and ukulele).
7. Polyphonic strum check: per-string in/out of tune.
8. Automatic instrument detection (e.g. fundamentals below ~200 Hz → guitar; re-entrant G4 above C4 → ukulele).
