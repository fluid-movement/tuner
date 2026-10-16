---
contract:
  id: 975f5e873af11e01593eb2ed600659928e1b5bc3ad92a5463368ae69385b524a
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
    - id: 90a2e5be5c9f9813a3d05a1387858d05630c59c23fc6d56d29b97c2195d84e7c
      kind: acceptance
      text: Simulated plucks at about -34 dBFS peak with -50 dBFS noise and a 120 Hz low cut show a reading (tracking or held) for at least 95% of the first 2 s on every guitar and ukulele string, with tracked readings within ±3 cents
    - id: b423c159cfd2f58587e47d7eabd95dcf00e2e56c32292ff93d3cd39b557595bc
      kind: acceptance
      text: Noise-only input never locks onto a note; a pluck is shown within 100 ms; switching to another string retargets within 150 ms
    - id: 390fb4e002b056c952d7ba80961b29ff5e0a2fe080f87db951e8dc12308b156c
      kind: acceptance
      text: After the string fades the last reading is held (dimmed) for about 3 s, then cleared
    - id: 548419116e5be8f23dfe54e04bca10fe6e33f48436594997292299f2e4e18a28
      kind: acceptance
      text: The page shows an input level meter and a ?debug overlay with raw frequency, clarity, level and noise floor
    - id: 35dd40d2d247e83ea2c2445d5c5dc5dd5feb02164a00d4cadf43312e31f50164
      kind: acceptance
      text: 'Playwright: a decaying fake-mic pluck stays on screen through its decay; existing e2e tests still pass'
    - id: 932e0c5f0a775832dc9a5f652a0cd426cb6876e9618c027c43c2508ecbe7eae6
      kind: acceptance
      text: just check passes
by: claude-code/opus-5.5
at: 2026-10-16T08:48:56Z
baseline: 20261016-084855
---
