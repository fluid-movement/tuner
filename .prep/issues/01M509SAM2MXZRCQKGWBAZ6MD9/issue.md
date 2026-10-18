---
title: Document foundations
kind: code
parent: 01M509K4D1QQSP51QA9Z553JAA
tags:
  - bootstrap
---

Write two draft knowledge entries describing the project as it is now: /conventions/build-and-tooling.md (Cargo workspace and crates, wasm build pipeline incl. the prebuilt path used by Docker, pnpm/SvelteKit setup, justfile recipes, release profile incl. strip, versions and where they are pinned) and /conventions/testing.md (what each test layer covers and how to run it: Rust unit/integration tests incl. the allocation tests, synthetic-signal generators, evaluation examples poly_eval/presence_eval/track_eval, tuner-lab tests, Vitest, Playwright fake-mic projects incl. E2E_PORT and E2E_BASE_URL, CI's just check). Link both from /overview.md. The stack choices are already recorded as decision records (e.g. "Stack: SvelteKit + Rust/WASM in an AudioWorklet, static PWA"); link that decision from the overview instead of writing a separate stack entry.

## Open questions
