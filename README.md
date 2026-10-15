# Tuner

Browser tuner for acoustic guitar and ukulele. Pitch analysis runs in a Rust core compiled to WebAssembly inside an AudioWorklet; the UI is a SvelteKit 3 static site.

- `crates/tuner-core` — pure Rust pitch analysis (`cargo test`)
- `crates/tuner-wasm` — raw C-ABI WASM exports of the core
- `web/` — SvelteKit app (pnpm)

Requires Rust with the `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`), Node 24 and pnpm.

```sh
cargo test
cd web
pnpm install
pnpm dev     # builds the wasm, then starts Vite
pnpm build   # static site in web/build
pnpm test && pnpm check && pnpm lint
pnpm test:e2e  # Playwright in installed Chrome with a fake mic
```
