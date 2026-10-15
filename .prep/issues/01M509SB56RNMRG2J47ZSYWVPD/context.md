See [Overview](/overview.md). Greenfield repo; this issue creates the skeleton every later issue builds on.

- Root `Cargo.toml` workspace with `crates/tuner-core` (lib, no deps, `#![no_std]`-friendly not required) and `crates/tuner-wasm` (`cdylib`, depends on tuner-core, exposes a raw C ABI).
- `web/`: SvelteKit 3.1 (Svelte 5) via `pnpm dlx sv create`, TypeScript, `@sveltejs/adapter-static`, Vitest.
- Wasm build: `cargo build -p tuner-wasm --target wasm32-unknown-unknown --release`, copied to `web/static/wasm/tuner.wasm` by a `web` script (`pnpm build:wasm`) that `dev` and `build` run first.
- Placeholder export (e.g. `tuner_version()`) loaded on the page proves the pipeline end to end.
