## D1: Build images in GitHub Actions, Coolify only pulls
date: 2026-10-16

The 4 GB VPS runs other services and on-server Docker builds (Rust release build with LTO, pnpm install, Vite) have crashed it before. GitHub runners build with layer caching and push to GHCR; Coolify's Docker Image resource just pulls ~62 MB. The package is public because the repo is, so the server needs no registry credentials. A `version.txt` baked from the commit SHA lets CI detect when the new release is live before smoke-testing it. Alternatives: building on the server with memory limits (still competes with services), a separate build server (more infrastructure).
