---
type: convention
title: Build and tooling
description: Cargo workspace, WASM/static build, just recipes and version sources
status: draft
generated:
  by: codex/implementer
  at: 2026-10-18T09:22:16Z
scope:
  - Cargo.toml
  - Cargo.lock
  - crates/*/Cargo.toml
  - web/package.json
  - web/pnpm-*.yaml
  - web/.npmrc
  - web/scripts/build-wasm.js
  - web/vite.config.ts
  - web/src/routes/+layout.ts
  - justfile
  - Dockerfile
  - .github/workflows/ci.yml
  - README.md
---

The root justfile coordinates native Rust, WASM and the static SvelteKit site. The [stack decision](/decisions/01M4ZPXYG12WVJMAHKVGWTN0ME.md) records the architecture choice; [Deployment](/conventions/deployment.md) covers publishing/hosting and [Testing](/conventions/testing.md) covers test commands.

## Cargo workspace

`Cargo.toml` uses resolver 3 and includes crates/tuner-core, crates/tuner-wasm and crates/tuner-lab. All inherit version 0.1.0, edition 2024 and MIT licensing. tuner-core is platform-independent DSP with an internal FFT and num-complex 0.4.6 (default features disabled, std enabled). tuner-wasm depends on the core and builds cdylib and rlib with raw C-ABI exports and no JS imports. tuner-lab is an unpublished native CLI depending on the core, claxon 0.4.3, hound 3.5.1, serde 1 with derive, and serde_json 1. Manifests specify dependency requirements; Cargo.lock records exact resolutions. See [Pitch core](/components/pitch-core.md), [Audio pipeline](/components/audio-pipeline.md) and [Recording analysis](/components/tuner-lab.md).

The root release profile sets opt-level=3, lto=true, codegen-units=1, panic="abort" and strip=true. The manifest notes that stripping removes native symbols and the WASM name section (about 16 KB).

## WASM pipeline and Docker prebuilt path

`web/scripts/build-wasm.js` runs `cargo build -p tuner-wasm --target wasm32-unknown-unknown --release` from the repository root. It copies target/wasm32-unknown-unknown/release/tuner_wasm.wasm to web/static/wasm/tuner.wasm, creating the destination directory. With TUNER_WASM_PREBUILT set it copies that file instead and exits without running Cargo; relative prebuilt paths resolve from the process working directory.

Docker first compiles WASM in rust:1.98-slim. Its node:24-slim stage installs pnpm@12.11.2, installs frozen web dependencies, copies WASM to /tmp/tuner.wasm, sets TUNER_WASM_PREBUILT=/tmp/tuner.wasm and runs pnpm build. Thus the package's build pre-hook works without Rust in the Node stage. nginx:1.29-alpine serves the resulting web/build. See [Deployment](/conventions/deployment.md) for headers, health checks, image publishing and Coolify.

## Web setup and version sources

web/package.json defines the private ESM package tuner-web 0.0.1. Declared requirements include Svelte ^5.57.1, SvelteKit ^3.0.0, static adapter ^4.0.0, Svelte Vite plugin ^7.2.0, Vite ^8.3.0, TypeScript ^6.0.3, Vitest ^4.1.8 and exact Playwright 1.64.1. The manifest also lists lint/type tooling versions; web/pnpm-lock.yaml locks dependency resolutions. web/.npmrc enables engine-strict; web/pnpm-workspace.yaml allows esbuild's install build.

predev, prebuild, pretest and precheck each invoke build:wasm. dev/build/preview run Vite; prepare runs svelte-kit sync with a fallback; check runs sync then svelte-check with tsconfig.json; check:watch adds watch mode. lint runs Prettier check then ESLint; format writes Prettier formatting. test runs Vitest once; test:watch invokes Vitest directly without the pretest hook; test:e2e invokes Playwright.

web/vite.config.ts configures SvelteKit with adapter-static and forces runes for application files except node_modules. web/src/routes/+layout.ts prerenders all routes. Static output goes to web/build. Installable/offline PWA support remains roadmap work.

README prerequisites are rustup, Node 24, pnpm, just and installed Google Chrome for browser tests. CI uses moving stable Rust with wasm32, rustfmt and clippy, Node major 24 and pnpm 12.11.2. Docker pins Rust 1.98, Node major 24, pnpm 12.11.2 and nginx 1.29 using image tags/install commands, not image digests. There is no repository rust-toolchain file or packageManager field pinning local Rust/pnpm. The CI runner (ubuntu-24.04) and action versions are recorded in .github/workflows/ci.yml; Rust/web dependency resolutions are in their lockfiles.

## Just recipes

Run from the root; recipes use bash with -euo pipefail. The default recipe lists commands.

| Recipe | Current behavior |
| --- | --- |
| just setup | Add wasm32-unknown-unknown target; install web dependencies with pnpm install --frozen-lockfile. |
| just dev [args...] | Setup, then pnpm dev with forwarded arguments; predev builds WASM. |
| just build | Setup, then pnpm build (WASM and static site). |
| just preview | Build, then serve production output. |
| just test | Setup, cargo test, then pnpm test (including WASM pretest). |
| just e2e | Setup, then pnpm test:e2e. Use E2E_PORT per worktree. |
| just lint | Setup, cargo fmt --check, clippy --all-targets with warnings denied, pnpm check and pnpm lint. |
| just check | Combine lint, test, build and e2e; this is CI's check command. |
| just analyze [args...] | Run tuner-lab natively in release mode with forwarded arguments. |
| just analyze-demo [dir] | Generate synthetic takes; default directory recordings/demo. |
| just docker | Build image tagged tuner and run it on localhost:8080. |
| just fmt | Format Rust and web sources. |
| just clean | Cargo clean; remove web/build, web/.svelte-kit, web/static/wasm and web/test-results. |

For this worktree, full validation is `E2E_PORT=4202 just check`; see [Testing](/conventions/testing.md) for individual layers and E2E_BASE_URL.
