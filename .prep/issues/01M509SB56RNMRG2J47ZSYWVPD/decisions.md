## D1: Raw C-ABI wasm, no wasm-bindgen
date: 2026-10-15

The module will run inside an AudioWorkletGlobalScope, which lacks fetch and (in some browsers) TextDecoder, which wasm-bindgen's JS glue relies on. Plain `extern "C"` exports with a shared Float32 buffer in linear memory need no glue: the main thread fetches the bytes, posts them to the worklet, and the worklet instantiates them. It also avoids wasm-pack/wasm-bindgen-cli tooling; only the `wasm32-unknown-unknown` rustup target is needed. Alternatives: wasm-bindgen `--target web` with polyfills (more glue, version-pinned CLI), wasm-pack (same glue issue).
