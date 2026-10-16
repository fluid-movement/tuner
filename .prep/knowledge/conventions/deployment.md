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
  - .github/workflows
  - web/scripts/build-wasm.js
  - web/playwright.config.ts
confirmed_commit: 37831a0c45ca4b5b67e0dfb5845aa8962c438d95
---

The app is a static site shipped as a Docker image. GitHub Actions builds it and pushes it to GHCR; the user's Coolify instance only pulls and runs it. Never build on the server: the VPS has 4 GB RAM shared with other services and on-server builds have crashed it. Setup steps for Coolify and GitHub are in the README.

## CI (`.github/workflows/ci.yml`)

- `check` (pushes and PRs): Rust stable + wasm target, Node 24, pnpm 12.11.2, just; runs `just check`. Playwright uses the runner's preinstalled Google Chrome. Uploads `web/test-results/` on failure.
- `image` (main, after check): buildx with GHA cache, pushes `ghcr.io/fluid-movement/tuner:latest` and `:<sha>`, passing `GIT_SHA`.
- `deploy` (after image, environment `production`): calls `secrets.COOLIFY_WEBHOOK` with `secrets.COOLIFY_TOKEN` as bearer, polls `vars.PROD_URL/version.txt` until it equals the commit SHA (10 min max), then runs Playwright with `E2E_BASE_URL`. Skips with a notice when any of the three is missing.
- Validate workflow edits with `docker run --rm -v "$PWD":/repo --workdir /repo rhysd/actionlint:1.7.7`. A `run:` value containing `: ` (e.g. an `Authorization: Bearer` header) must be a block scalar.

## Image

1. `rust:1.98-slim` + `wasm32-unknown-unknown` builds `tuner-wasm`.
2. `node:24-slim` with pnpm pinned to the local version (`npm install -g pnpm@12.11.2`; keep in sync with the lockfile) installs with `--frozen-lockfile` and runs `pnpm build`. `TUNER_WASM_PREBUILT` points `web/scripts/build-wasm.js` at the wasm from stage 1, so the Node stage needs no Rust.
3. `nginx:1.29-alpine` serves `web/build` on port 80 with `deploy/nginx.conf`, plus a Docker `HEALTHCHECK` on `/healthz`. Build arg `GIT_SHA` (default `dev`) is written to `/version.txt`.

`.dockerignore` keeps `target/`, `node_modules/`, build output, `recordings/`, `.prep` and `.git` out of the context. `just docker` builds and runs the image on port 8080.

## nginx rules

- Rely on the stock `mime.types` for `application/wasm`. Never add a `types {}` block in the server config: it replaces the whole table and HTML/JS go out as `application/octet-stream`.
- `/_app/immutable/` (content-hashed, includes the worklet bundle): `max-age=31536000, immutable`. HTML, `/wasm/tuner.wasm` and other static files: `no-cache` (revalidate; the wasm URL is stable across releases).
- `add_header` in a `location` drops server-level headers, so the security headers (`X-Content-Type-Options`, `Referrer-Policy`, `Permissions-Policy: microphone=(self)`) are repeated where needed.
- gzip covers text, JSON, SVG and wasm. Unknown paths fall back to `index.html`.

## Coolify

A *Docker Image* resource on `ghcr.io/fluid-movement/tuner:latest` (the package is public, so no registry credentials), port 80 exposed, domain with HTTPS (required for the microphone on phones), health check `/healthz`, auto deploy off; CI triggers deploys through the resource's deploy webhook. Coolify's proxy terminates TLS.

## Testing a deployment

`E2E_BASE_URL=<url> pnpm exec playwright test` (in `web/`) runs the browser tests against any running deployment instead of a local preview build; the fake-microphone tests work remotely too.
