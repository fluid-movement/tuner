---
type: decision
title: "Dockerfile build pack, nginx static server"
description: "Dockerfile build pack, nginx static server"
date: "2026-10-16"
issue: 01M51WBY6NY1ARF2D5B47WMAY4
---

Coolify's Nixpacks/static build packs cannot install a Rust wasm toolchain and pnpm in one build, so the repo ships its own multi-stage Dockerfile and Coolify uses the Dockerfile build pack. nginx:alpine serves the static output (small image, full control over MIME and cache headers). Coolify's proxy (Traefik/Caddy) terminates TLS. Alternatives: building in CI and pushing an image to a registry (more moving parts; can come with the CI issue), Caddy as the file server (equivalent, nginx chosen for familiarity in Coolify setups).
