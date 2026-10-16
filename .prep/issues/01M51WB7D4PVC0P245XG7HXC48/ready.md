---
contract:
  id: c3356c859166a9d7067b5d5fe694ff7e4fd703cc5b66019006a02e122d9e1309
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
    - id: 91fe8280ebdd6d0fe8c2a732973df35e75eb82bdc20590822112c9f489f998c9
      kind: acceptance
      text: tuner-lab reads WAV and FLAC (mono or stereo, 44.1/48 kHz) and segments a take into plucks and strums, reporting the classification
    - id: 86dd641d1dae95b57795c41ab202b1cfb7285f8c59fcb6ebe6fc8c52cd3bfac4
      kind: acceptance
      text: On synthetic takes, every pluck and strum is found and classified correctly, pluck truth is within ±2 cents of the rendered frequencies, and strum verdicts agree with the truth
    - id: b191a3472d2ef6f0acb9fe73828fec37296709a6086585eb4c6bdb390ce10331
      kind: acceptance
      text: Per-file and aggregate reports cover single-string experience (latency, shown share, spread, drift) and strum error/verdict agreement; --json output exists
    - id: 54626f249505a46fc2c66cb497d987f74200fba542e0668d46f92fa5252e3c69
      kind: acceptance
      text: just analyze runs the tool on recordings/ (or a given directory) and explains the recording protocol when it is empty
    - id: bd159bb3007442da21e933008db435fc5f6562288ab2b8046eaa001e86912f99
      kind: acceptance
      text: cargo test, clippy and just check pass; tuner-core gains no new dependencies
by: claude-code/opus-5.5
at: 2026-10-16T10:20:12Z
baseline: 20261016-102011
---
