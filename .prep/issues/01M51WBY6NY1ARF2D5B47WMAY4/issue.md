---
title: Deploy to Coolify with a Dockerfile
kind: code
parent: 01M509SB0RMTVQ2PY4M21KKQAA
tags:
  - deploy
priority: high
---

Deploy the app to the user's Coolify instance from the GitHub repo (fluid-movement/tuner) so it is reachable over HTTPS on a real domain, which also enables microphone access on phones. Coolify builds from a Dockerfile in the repo: a Rust stage builds the wasm, a Node/pnpm stage builds the static SvelteKit site, and a small web server stage serves it with correct MIME types (application/wasm) and caching (immutable hashed assets, revalidated HTML and wasm). The image must build and serve locally with `docker build` / `docker run`, and a `just` recipe should wrap that. Document the Coolify setup steps. CI (GitHub Actions running `just check`, then triggering the Coolify deploy) is a follow-up.

## Open questions
