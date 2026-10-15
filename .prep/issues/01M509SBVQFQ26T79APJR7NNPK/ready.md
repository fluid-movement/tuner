---
contract:
  id: 0e485cbdbaf8f859ca8eaeac54a96d0160b2d81a8c78eeb26ef8b5f667976ac9
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
    - id: a487ebde9f2960f2928f70a112ba75bb3dd891e9f6b56983c1ded1bb9cf9c89b
      kind: acceptance
      text: A synthetic strum generator (inharmonic, decaying partials, onset spread, level variation, noise) exists in tuner-core with tests
    - id: 4776b8bea6286e2cff540dfe3f5b92f7b321e82acdd82443a8851d532fe3e3c5
      kind: acceptance
      text: At least two approaches are implemented and compared on the same synthetic strums with reported error statistics
    - id: 379044e474a568aae5b19be8d5fed9412bbaaace43b53283668f14c6466663a4
      kind: acceptance
      text: Findings record the chosen approach, accuracy, cost per analysis, known failure cases and a recommendation for the strum-check issue
    - id: e1157bcde549697843cf5d196b0ddb975119e6ce9524d4f8901e1ba658d89a66
      kind: acceptance
      text: cargo test and cargo clippy pass
by: claude-code/opus-5.5
at: 2026-10-15T18:03:14Z
baseline: 20261015-180313
---
