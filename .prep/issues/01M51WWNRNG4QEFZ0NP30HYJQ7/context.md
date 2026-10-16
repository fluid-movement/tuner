See [Deployment](/conventions/deployment.md) and [Overview](/overview.md).

- `.github/workflows/ci.yml` with jobs `check` (Rust stable + wasm target, clippy, rustfmt; Node 24; pnpm 12.11.2; just; Swatinem/rust-cache and pnpm store cache; `just check` using the runner's preinstalled Google Chrome for Playwright; upload Playwright output on failure), `image` (main only, needs check: buildx, GHCR login with GITHUB_TOKEN, push `:latest` and `:<sha>`, GHA layer cache, `GIT_SHA` build arg), `deploy` (needs image: curl `secrets.COOLIFY_WEBHOOK` with `secrets.COOLIFY_TOKEN` as bearer; poll `vars.PROD_URL/version.txt` until it equals the commit SHA; run Playwright with `E2E_BASE_URL`).
- Dockerfile: `ARG GIT_SHA` written to `version.txt` in the site root (served `no-cache`).
- Coolify switches from the repository resource to a *Docker Image* resource on `ghcr.io/fluid-movement/tuner:latest` with auto deploy off.
