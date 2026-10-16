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

## Deployment (Coolify)

The repo ships a multi-stage `Dockerfile`: Rust builds the wasm, Node/pnpm builds the static site, nginx serves it on port 80 (`deploy/nginx.conf`: correct `application/wasm` type, gzip, immutable caching for hashed assets, `no-cache` for HTML and the wasm, `/healthz`). `just docker` builds and runs it on http://localhost:8080.

In Coolify (v4):

1. **Source**: connect GitHub under *Sources* (GitHub App is best: it enables deploy-on-push) or use a deploy key for the private repo.
2. **New resource** in your project/environment → *Private Repository (with GitHub App)* → `fluid-movement/tuner`, branch `main`.
3. **Build pack**: *Dockerfile*; base directory `/`, Dockerfile location `/Dockerfile`.
4. **Network**: *Ports Exposes* `80`. Leave *Ports Mappings* empty (the proxy routes to the container).
5. **Domain**: set e.g. `https://tuner.example.com` and point a DNS A/AAAA record at the Coolify server; the proxy obtains the TLS certificate. HTTPS is required for microphone access.
6. **Health check**: enable it with path `/healthz`, port `80` (the image also has a Docker `HEALTHCHECK`).
7. **Deploy**. The first build compiles Rust and installs Node dependencies (a few minutes, ~1–2 GB RAM); later builds reuse Docker layer cache where Coolify keeps it.
8. **Check** the live site: open it on a phone and press Start, or run the browser tests against it: `cd web && E2E_BASE_URL=https://tuner.example.com pnpm exec playwright test`.

With the GitHub App, Coolify redeploys on every push to `main`. Once CI exists, switch *Auto Deploy* off and let CI call Coolify's deploy webhook after `just check` passes.
