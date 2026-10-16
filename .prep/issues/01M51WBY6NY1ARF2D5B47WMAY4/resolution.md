---
contract:
  id: 1d7abd9794508fea85037b84ec5b881d3ce228eeef6c20b303184f4e91e61858
  verification: cargo test (Rust core, incl. synthetic-signal pitch tests) + frontend unit tests; manual check with a real instrument for detection changes
  documentation: review affected knowledge entries via prep knowledge review on every completion
  ancestors:
    - 01M509SB0RMTVQ2PY4M21KKQAA
  requirements:
    - id: 0ed80fd753f73dbec8cfa49b721c5a6108f8f66de4875b158daaf9b0bc8e3aea
      kind: verification
      text: cargo test (Rust core, incl. synthetic-signal pitch tests) + frontend unit tests; manual check with a real instrument for detection changes
    - id: 5e5cff922ec43ceb8a22e0bea73e715418c9c2439bbb4c6322f3bb900c0dd43f
      kind: documentation
      text: review affected knowledge entries via prep knowledge review on every completion
    - id: f2d75eba11353969ea0ef3437e450159c5a6bedd6753d851f723906d42aea682
      kind: dod
      text: prep check passes
    - id: e2fd88029a0cdcfbbeb670dbec3ea75f5c02f5cb0d04ab873d908df7ace8191f
      kind: acceptance
      text: docker build succeeds from a clean checkout and the container serves the app on port 80 with tuner.wasm as application/wasm
    - id: fad41bfaba38d105302288ddee6ef786011767ee5942c8fa0e6f6e0c9a3abdaf
      kind: acceptance
      text: Hashed assets are served immutable; index.html and tuner.wasm are revalidated; /healthz returns 200
    - id: 8c5e9da4a3f13edb33d2fb4330149e88384fc92893988dac1089ab67860ef071
      kind: acceptance
      text: just docker builds and runs the image locally; just check still passes
    - id: fa29a66961d54d3b5d02f9b3ead2c355c41b3b21abdf0a6a68561f69d336325b
      kind: acceptance
      text: README documents the Coolify setup (build pack, port, domain, health check, auto deploy)
verification:
  - id: a604d64372640e99404968f4f628727f2ec55eb1f2203293f0db9fccfb3715fa
    contract: 1d7abd9794508fea85037b84ec5b881d3ce228eeef6c20b303184f4e91e61858
    requirement: 0ed80fd753f73dbec8cfa49b721c5a6108f8f66de4875b158daaf9b0bc8e3aea
    result: pass
    method: automated
    command: just check
    observation: cargo test all ok; vitest 13 passed; no detection changes
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-16T08:16:59.640989Z
    scopes:
      - Dockerfile
      - deploy
      - .dockerignore
      - web/scripts/build-wasm.js
      - web/playwright.config.ts
      - README.md
      - justfile
    snapshot: f0b18d294539ebe11005a1d1b7d2b7127aa1fdf78022188307d2773b917b6a1b
  - id: a115f53192f904596325bcdde5971af3295f7b705e101a524e1405692ebbdfaf
    previous: a604d64372640e99404968f4f628727f2ec55eb1f2203293f0db9fccfb3715fa
    contract: 1d7abd9794508fea85037b84ec5b881d3ce228eeef6c20b303184f4e91e61858
    requirement: 5e5cff922ec43ceb8a22e0bea73e715418c9c2439bbb4c6322f3bb900c0dd43f
    result: pass
    method: manual
    observation: Knowledge reviews for deployment, overview, audio-pipeline (updated and confirmed)
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-16T08:16:59.880108Z
    scopes:
      - Dockerfile
      - deploy
      - .dockerignore
      - web/scripts/build-wasm.js
      - web/playwright.config.ts
      - README.md
      - justfile
    snapshot: f0b18d294539ebe11005a1d1b7d2b7127aa1fdf78022188307d2773b917b6a1b
  - id: 9706cc3e6097c38970cbd31ca64649eb85e560f32183262d1cb8e21e789453fc
    previous: a115f53192f904596325bcdde5971af3295f7b705e101a524e1405692ebbdfaf
    contract: 1d7abd9794508fea85037b84ec5b881d3ce228eeef6c20b303184f4e91e61858
    requirement: f2d75eba11353969ea0ef3437e450159c5a6bedd6753d851f723906d42aea682
    result: pass
    method: automated
    command: prep check
    observation: 0 errors, 0 warnings
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-16T08:17:00.071315Z
    scopes:
      - Dockerfile
      - deploy
      - .dockerignore
      - web/scripts/build-wasm.js
      - web/playwright.config.ts
      - README.md
      - justfile
    snapshot: f0b18d294539ebe11005a1d1b7d2b7127aa1fdf78022188307d2773b917b6a1b
  - id: 71eb43fac4c752e5a0801461ed5d6bb19119e740a5e1bd1150f73c8e3c109a0e
    previous: 9706cc3e6097c38970cbd31ca64649eb85e560f32183262d1cb8e21e789453fc
    contract: 1d7abd9794508fea85037b84ec5b881d3ce228eeef6c20b303184f4e91e61858
    requirement: e2fd88029a0cdcfbbeb670dbec3ea75f5c02f5cb0d04ab873d908df7ace8191f
    result: pass
    method: automated
    command: docker build -t tuner . && docker run -p 8089:80 tuner; E2E_BASE_URL=http://localhost:8089 pnpm exec playwright test
    observation: Image builds (61.7 MB); / text/html, tuner.wasm application/wasm (gzip); Playwright 7/7 pass against the container
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-16T08:17:00.246839Z
    scopes:
      - Dockerfile
      - deploy
      - .dockerignore
      - web/scripts/build-wasm.js
      - web/playwright.config.ts
      - README.md
      - justfile
    snapshot: f0b18d294539ebe11005a1d1b7d2b7127aa1fdf78022188307d2773b917b6a1b
  - id: f05fff521b5dcecd195d35ee016612af8d0f26eb32a557766cceb9e13ff7270f
    previous: 71eb43fac4c752e5a0801461ed5d6bb19119e740a5e1bd1150f73c8e3c109a0e
    contract: 1d7abd9794508fea85037b84ec5b881d3ce228eeef6c20b303184f4e91e61858
    requirement: fad41bfaba38d105302288ddee6ef786011767ee5942c8fa0e6f6e0c9a3abdaf
    result: pass
    method: automated
    command: curl -sI on /, /wasm/tuner.wasm, /_app/immutable/*.js, worklet, /healthz
    observation: 'immutable assets and worklet: max-age=31536000, immutable; / and wasm: no-cache; /healthz 200 ok; Docker health check configured'
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-16T08:17:00.435457Z
    scopes:
      - Dockerfile
      - deploy
      - .dockerignore
      - web/scripts/build-wasm.js
      - web/playwright.config.ts
      - README.md
      - justfile
    snapshot: f0b18d294539ebe11005a1d1b7d2b7127aa1fdf78022188307d2773b917b6a1b
  - id: de40aae0407329ae0ad6dca04e271aee5542f72e1b1bc6fc0e5307e0c8f4e2b5
    previous: f05fff521b5dcecd195d35ee016612af8d0f26eb32a557766cceb9e13ff7270f
    contract: 1d7abd9794508fea85037b84ec5b881d3ce228eeef6c20b303184f4e91e61858
    requirement: 8c5e9da4a3f13edb33d2fb4330149e88384fc92893988dac1089ab67860ef071
    result: pass
    method: automated
    command: just docker; curl localhost:8080/healthz; just check
    observation: just docker served /healthz ok and application/wasm on 8080; just check passed (lint, tests, build, 7 e2e)
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-16T08:17:00.613485Z
    scopes:
      - Dockerfile
      - deploy
      - .dockerignore
      - web/scripts/build-wasm.js
      - web/playwright.config.ts
      - README.md
      - justfile
    snapshot: f0b18d294539ebe11005a1d1b7d2b7127aa1fdf78022188307d2773b917b6a1b
  - id: 006d4336251acfd64e84e66656e79fac20f973c6450204c8f521bf99046c6c05
    previous: de40aae0407329ae0ad6dca04e271aee5542f72e1b1bc6fc0e5307e0c8f4e2b5
    contract: 1d7abd9794508fea85037b84ec5b881d3ce228eeef6c20b303184f4e91e61858
    requirement: fa29a66961d54d3b5d02f9b3ead2c355c41b3b21abdf0a6a68561f69d336325b
    result: pass
    method: manual
    observation: 'README Deployment (Coolify) section: source, resource, Dockerfile build pack, port 80, domain/HTTPS, /healthz, deploy, verification, auto deploy vs CI webhook'
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-16T08:17:00.784858Z
    scopes:
      - Dockerfile
      - deploy
      - .dockerignore
      - web/scripts/build-wasm.js
      - web/playwright.config.ts
      - README.md
      - justfile
    snapshot: f0b18d294539ebe11005a1d1b7d2b7127aa1fdf78022188307d2773b917b6a1b
knowledge_reviews:
  - id: 822928a8e1644c6d419a9902557304111ec07b0e9817a0375bcc0dd8cd67c48a
    contract: 1d7abd9794508fea85037b84ec5b881d3ce228eeef6c20b303184f4e91e61858
    kind: updated
    entry: /conventions/deployment.md
    entry_revision: 6deaab77391be5364046f17234af3df099e4daa20703e393c66682bc493383ac
    observation: 'New entry: image stages, nginx rules, Coolify, E2E_BASE_URL'
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-16T08:16:32Z
    scopes:
      - .dockerignore
      - Dockerfile
      - README.md
      - deploy
      - justfile
      - web/playwright.config.ts
      - web/scripts/build-wasm.js
    snapshot: f0b18d294539ebe11005a1d1b7d2b7127aa1fdf78022188307d2773b917b6a1b
  - id: 6c05ee6655c9986b434e92613b8ec5b34345098ead87d769cd8f9cf542292057
    previous: 822928a8e1644c6d419a9902557304111ec07b0e9817a0375bcc0dd8cd67c48a
    contract: 1d7abd9794508fea85037b84ec5b881d3ce228eeef6c20b303184f4e91e61858
    kind: updated
    entry: /overview.md
    entry_revision: cf897d09e6a667cbcaaaa5251ada34e2386084b5f7f3142077b9d915063b72b8
    observation: Running section links Deployment
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-16T08:16:32Z
    scopes:
      - .dockerignore
      - Dockerfile
      - README.md
      - deploy
      - justfile
      - web/playwright.config.ts
      - web/scripts/build-wasm.js
    snapshot: f0b18d294539ebe11005a1d1b7d2b7127aa1fdf78022188307d2773b917b6a1b
  - id: d16f4b109a20c871214344ac7c8f2f2b55cfafbdb6ede8b611b22df695c95cd8
    previous: 6c05ee6655c9986b434e92613b8ec5b34345098ead87d769cd8f9cf542292057
    contract: 1d7abd9794508fea85037b84ec5b881d3ce228eeef6c20b303184f4e91e61858
    kind: updated
    entry: /components/audio-pipeline.md
    entry_revision: 13a7e39cedf4d90f43792ca1b51107641ab54d159b41060436d24da13a608734
    observation: Testing mentions E2E_BASE_URL
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-16T08:16:32Z
    scopes:
      - .dockerignore
      - Dockerfile
      - README.md
      - deploy
      - justfile
      - web/playwright.config.ts
      - web/scripts/build-wasm.js
    snapshot: f0b18d294539ebe11005a1d1b7d2b7127aa1fdf78022188307d2773b917b6a1b
generation: 1
outcome: done
by: claude-code/opus-5.5
at: 2026-10-16T08:17:00Z
documentation:
  entries:
    - /conventions/deployment.md
    - /overview.md
    - /components/audio-pipeline.md
dod:
  - prep check passes
---
