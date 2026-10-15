---
contract:
  id: e304c2306f19bf580c27dec1c765820111d406be8af5f1b7e9be7294ae310623
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
    - id: 05a0f606567262d11ee0ce36cf1b66af16fb615ae20b91ff129806fdaa58e45e
      kind: acceptance
      text: just dev starts the dev server from a clean checkout (setup runs automatically)
    - id: 2a8974af01fb2d5e41ad097086a27b70700312dccee07a7987f7f64eb245d6d8
      kind: acceptance
      text: just check runs cargo test, clippy, fmt check, pnpm test, check, lint, build and e2e, and passes
    - id: edbea6276406c5033d5f03f267cb854881231c6e8c3b71755f8dd546cfa0e66f
      kind: acceptance
      text: just --list documents every recipe; README shows the 1–2 command quickstart
by: claude-code/opus-5.5
at: 2026-10-15T17:56:15Z
baseline: 20261015-175615
---
