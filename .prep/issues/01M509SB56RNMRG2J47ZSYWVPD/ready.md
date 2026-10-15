---
contract:
  id: 2392fa25db8759c065612ab6c0e752ea170226ba234235e456e31918268dc4c4
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
    - id: 475bbf758949abe4b08aa17502b475f208edd17711a0be462d66b0130eb925e9
      kind: acceptance
      text: cargo test passes for the workspace
    - id: ea398a6eff13c1f7d0c2d33116464f7f657295aad8f6e646de722afe892a371b
      kind: acceptance
      text: pnpm build in web/ builds the wasm and produces a static site containing static/wasm/tuner.wasm
    - id: 473d0a6aaf780a36eedb2d71adeac3ac913afe2c7b78909d332cec5d430147c9
      kind: acceptance
      text: The dev page loads tuner.wasm and shows a value returned by a tuner-wasm export
    - id: e5f892177c7e9b52f5a636d939a90020b2014d5b00d14c97fdacaa1bafe4fd91
      kind: acceptance
      text: pnpm test (Vitest) and pnpm check pass in web/
by: claude-code/opus-5.5
at: 2026-10-15T17:32:05Z
baseline: 20261015-173205
---
