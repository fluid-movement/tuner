---
title: Scaffold Cargo workspace and SvelteKit 3 app with wasm build
kind: code
parent: 01M509SB0RMTVQ2PY4M21KKQAA
tags:
  - iter1
priority: high
---

Create a Cargo workspace with crates/tuner-core (pure Rust) and crates/tuner-wasm (WASM wrapper), and web/ as a SvelteKit 3 app (latest release, pnpm, static adapter, TypeScript). Wire the wasm build into the web dev/build so the app can load the module.

## Open questions
