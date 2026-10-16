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
by: claude-code/opus-5.5
at: 2026-10-16T08:13:31Z
baseline: 20261016-081330
---
