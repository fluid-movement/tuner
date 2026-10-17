---
type: decision
title: "Stack: SvelteKit + Rust/WASM in an AudioWorklet, static PWA"
description: "Stack: SvelteKit + Rust/WASM in an AudioWorklet, static PWA"
date: "2026-10-15"
issue: 01M509K5C9ANY25QH255QYJTP6
---

User choices (2026-10-15):
- Frontend: SvelteKit + Vite with the static adapter (alternatives: plain TS, React).
- Audio path: the WASM pitch core is instantiated inside an AudioWorkletProcessor; results are posted to the main thread (alternative: AnalyserNode polling on the main thread, rejected for latency/jank).
- Deployment: static site + PWA (installable, offline).
- Tunings: standard only for now (guitar EADGBE, ukulele GCEA re-entrant); alternates later.
