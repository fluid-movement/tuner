---
type: convention
title: Deployment
description: Multi-stage Dockerfile (Rust wasm → pnpm static build → nginx), nginx headers and caching, Coolify setup, E2E_BASE_URL for testing a deployment
status: draft
generated:
  by: claude-code/opus-5.5
  at: 2026-10-16T08:16:15Z
scope:
  - Dockerfile
  - deploy
  - .dockerignore
  - web/scripts/build-wasm.js
  - web/playwright.config.ts
---

The app is a static site. Production runs it from the root `Dockerfile` on the user's Coolify instance; the step-by-step Coolify setup is in the README.

## Image

1. `rust:1.98-slim` + `wasm32-unknown-unknown` builds `tuner-wasm`.
2. `node:24-slim` with pnpm pinned to the local version (`npm install -g pnpm@12.11.2`; keep in sync with the lockfile) installs with `--frozen-lockfile` and runs `pnpm build`. `TUNER_WASM_PREBUILT` points `web/scripts/build-wasm.js` at the wasm from stage 1, so the Node stage needs no Rust.
3. `nginx:1.29-alpine` serves `web/build` on port 80 with `deploy/nginx.conf`, plus a Docker `HEALTHCHECK` on `/healthz`.

`.dockerignore` keeps `target/`, `node_modules/`, build output, `recordings/`, `.prep` and `.git` out of the context. `just docker` builds and runs the image on port 8080.

## nginx rules

- Rely on the stock `mime.types` for `application/wasm`. Never add a `types {}` block in the server config: it replaces the whole table and HTML/JS go out as `application/octet-stream`.
- `/_app/immutable/` (content-hashed, includes the worklet bundle): `max-age=31536000, immutable`. HTML, `/wasm/tuner.wasm` and other static files: `no-cache` (revalidate; the wasm URL is stable across releases).
- `add_header` in a `location` drops server-level headers, so the security headers (`X-Content-Type-Options`, `Referrer-Policy`, `Permissions-Policy: microphone=(self)`) are repeated where needed.
- gzip covers text, JSON, SVG and wasm. Unknown paths fall back to `index.html`.

## Coolify

Dockerfile build pack, port 80 exposed, domain with HTTPS (required for the microphone on phones), health check `/healthz`. Coolify's proxy terminates TLS. Auto deploy on push via the GitHub App until CI takes over deploys through Coolify's deploy webhook.

## Testing a deployment

`E2E_BASE_URL=<url> pnpm exec playwright test` (in `web/`) runs the browser tests against any running deployment instead of a local preview build; the fake-microphone tests work remotely too.
