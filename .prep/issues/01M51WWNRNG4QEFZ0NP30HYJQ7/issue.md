---
title: 'CI: GitHub Actions checks, image build to GHCR, Coolify deploy'
kind: code
parent: 01M509SB0RMTVQ2PY4M21KKQAA
depends_on:
  - 01M51WBY6NY1ARF2D5B47WMAY4
tags:
  - deploy
priority: high
---

The Coolify VPS has 4 GB RAM and has been knocked over by on-server builds before, so images are built in CI, not on the server. GitHub Actions runs `just check` on every push and pull request. On pushes to main that pass, it builds the Docker image and pushes it to GHCR (public, like the repo), then triggers Coolify's deploy webhook so Coolify only pulls and restarts the image. After the deploy, CI waits until the live site serves the new version and runs the browser tests against it. Deploy steps skip cleanly while the Coolify secrets are not configured. Document the Coolify (Docker Image resource) and GitHub (secrets, variables) setup.

## Open questions
