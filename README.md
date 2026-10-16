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

## Deployment (CI → GHCR → Coolify)

Images are built in GitHub Actions, not on the server (the VPS has 4 GB RAM and on-server builds have taken it down). `.github/workflows/ci.yml`:

1. **check** (every push and PR): `just check` (lint, unit tests, build, Playwright in the runner's Chrome).
2. **image** (main only): builds the root `Dockerfile` and pushes `ghcr.io/fluid-movement/tuner:latest` and `:<commit sha>`.
3. **deploy** (main only): calls Coolify's deploy webhook, waits until `$PROD_URL/version.txt` shows the new commit, then runs the browser tests against production. It skips with a notice until the secrets below exist.

The image: Rust builds the wasm, Node/pnpm builds the static site, nginx serves it on port 80 (`deploy/nginx.conf`: `application/wasm`, gzip, immutable hashed assets, `no-cache` HTML/wasm/`version.txt`, `/healthz`). `just docker` builds and runs it on http://localhost:8080.

### Coolify (v4), once

1. **Make the package public** after the first image push: GitHub → your profile/org → *Packages* → `tuner` → *Package settings* → *Change visibility* → Public. Then the server pulls without credentials.
2. **New resource** in your project/environment → *Docker Image* → `ghcr.io/fluid-movement/tuner:latest`.
3. **Network**: *Ports Exposes* `80`; leave *Ports Mappings* empty.
4. **Domain**: `https://tuner.example.com`, with a DNS A/AAAA record pointing at the server. HTTPS is required for microphone access.
5. **Health check**: path `/healthz`, port `80`.
6. **Deploy** once by hand to check it comes up.
7. **Webhook**: in the resource's *Webhooks* tab copy the *Deploy Webhook* URL. Under *Keys & Tokens* → *API tokens* create a token with deploy permission.

If you created a repository-based resource earlier, delete it (or turn off its auto deploy) so the server never builds.

### GitHub, once

*Settings* → *Environments* → `production` (created by the first deploy run, or create it):

- secret `COOLIFY_WEBHOOK`: the deploy webhook URL
- secret `COOLIFY_TOKEN`: the API token
- variable `PROD_URL`: `https://tuner.example.com` (no trailing slash)

From then on every green push to `main` deploys, and a failed check never reaches production. To test a deployment by hand: `cd web && E2E_BASE_URL=https://tuner.example.com pnpm exec playwright test`.
