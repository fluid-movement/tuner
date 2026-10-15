# Tuner

Browser tuner for acoustic guitar and ukulele. Pitch analysis runs in a Rust core compiled to WebAssembly inside an AudioWorklet; the UI is a SvelteKit 3 static site.

- `crates/tuner-core`: pure Rust pitch analysis
- `crates/tuner-wasm`: raw C-ABI WASM exports of the core
- `web/`: SvelteKit app (pnpm)

## Quickstart

Requires Rust (rustup), Node 24, pnpm, [just](https://github.com/casey/just) and Google Chrome (for browser tests).

```sh
just dev     # install what's needed, build the wasm, start the dev server
just check   # lint, unit tests, production build and browser tests
```

Other recipes: `just --list` (`build`, `preview`, `test`, `e2e`, `lint`, `fmt`, `clean`). `just dev --host` serves on your network; phones need HTTPS for microphone access, so test there with a tunnel or a deployed build.
