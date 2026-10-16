See [Overview](/overview.md) (Running) and [Audio pipeline](/components/audio-pipeline.md).

- `Dockerfile` at the repo root, three stages: `rust:1.98-slim` + `wasm32-unknown-unknown` builds `tuner-wasm`; `node:24-slim` with pnpm (version pinned to the lockfile's) runs `pnpm install --frozen-lockfile` and `pnpm build` with the prebuilt wasm (the `build:wasm` script must skip cargo when told the wasm is prebuilt); `nginx:alpine` serves `web/build` on port 80.
- `deploy/nginx.conf`: `application/wasm` type, gzip for text/wasm, `Cache-Control: public, max-age=31536000, immutable` for `/_app/immutable/`, `no-cache` for HTML and `/wasm/`, SPA-safe 404 handling for the static adapter output, a `/healthz` endpoint for Coolify's health check.
- `.dockerignore` keeps `target/`, `node_modules/`, build output, recordings and `.prep` out of the context.
- `just docker` builds and runs the image locally on port 8080.
