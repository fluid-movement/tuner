---
contract:
  id: b4e2c0eb31ffbfb4bd43e97e1f176fd97dda7f2bcc7d43e57d749ea1c9cd577b
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
    - id: 6c6c786fcf185a914278b9db4b7292189cba2f11f5c54385fbe1299316a4b6c9
      kind: acceptance
      text: tuner-wasm exports create/input pointer/detect/frequency/clarity/free and Rust tests exercise them natively
    - id: bd69f00ef8ed7dfcb04aff3ffd9d2b6cc42743b4267d6270c8c947d2fc1d578b
      kind: acceptance
      text: 'Vitest: the frame accumulator emits 2048-sample frames every 1024 samples in order, and the built wasm detects a 110 Hz tone written into its input buffer'
    - id: 3c0e730426d78500e7b062fdc91b9ef6a086da60064caec57bdd7b5a2d64dc7f
      kind: acceptance
      text: Playwright in Chrome with a fake microphone playing a generated 110 Hz (A2) tone shows about 110 Hz / A2 on the page after pressing Start
    - id: 35c8b6fb6357a0e44436432bb96d01ddc62fbbbb58ef4f088da396a589082f3d
      kind: acceptance
      text: Audio starts only after a user gesture; denied microphone permission shows an error message
    - id: 67dfc282d612dabc4654be0d12594c7da43348a63866bddfc7e499e90c4effdc
      kind: acceptance
      text: cargo test, cargo clippy, pnpm test, pnpm check, pnpm lint and pnpm build pass
by: claude-code/opus-5.5
at: 2026-10-15T17:45:17Z
baseline: 20261015-174516
---
