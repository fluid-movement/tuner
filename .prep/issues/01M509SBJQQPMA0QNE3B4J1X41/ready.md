---
contract:
  id: 796095dcc36a0e9dd82ab6560700c51e0d9da6e56fdded6106431d896cad182f
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
    - id: adff3bf1e6c4447245f8d0f0ff28aee41595a970d9c337672ebdaf03cba7b9af
      kind: acceptance
      text: 'Real-instrument check (deferred from pitch detection 01M509SBE2JC07NJX9MBS21FMP): every string of a real acoustic guitar and ukulele reads the right string and settles in tune against a reference tuner'
    - id: 70f6f90f564c98ffaf8662d2fa5465db506307cd25bbbe5186a44cb8c828a3e4
      kind: acceptance
      text: Selecting guitar or ukulele shows that instrument's strings; the choice survives a reload
    - id: ddd39f635ce1ad129cdab68f685bb655e36deefcecc3d2e73bedf5d4b1dc3f67
      kind: acceptance
      text: Playwright with a fake-mic E2 (82.41 Hz) on guitar highlights string E2 and shows in tune; a 10-cent-flat A2 shows flat and not in tune; a C4 on ukulele highlights C4
    - id: 61cab9916608e0e77c636353730930edfdbc7814f2790bd9d6e2c941816cff3c
      kind: acceptance
      text: 'Smoother and in-tune rule are unit-tested: outlier rejection, reset on jump, timeout, hysteresis'
    - id: c780691250b946d5cccc3f19ced68665554dece0883fbfda39591d43e5d5a346
      kind: acceptance
      text: The layout works at 375 px width without horizontal scrolling
    - id: ee525140e34acf6f0b81417505088c031f409062b752c3a9de76313ce838af61
      kind: acceptance
      text: cargo test, cargo clippy, pnpm test, pnpm check, pnpm lint, pnpm build and pnpm test:e2e pass
by: claude-code/opus-5.5
at: 2026-10-15T17:49:59Z
baseline: 20261015-174958
---
