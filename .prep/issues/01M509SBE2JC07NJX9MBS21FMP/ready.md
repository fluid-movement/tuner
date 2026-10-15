---
contract:
  id: df2aa81d7251366cf3bc5d474573bf1f76d3aba34f5c9753feed4f65a9f0c351
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
    - id: 9ef5491702ab91fd117d842685347318a98c1d042cd396813100b385d1422b10
      kind: acceptance
      text: PitchDetector detects pure sines at every guitar and ukulele standard-tuning string within ±1 cent at 44.1 and 48 kHz
    - id: bf01ebcba4fe05aa352ff92177257e869eff2dcbc00cecd76806480cdc1ee834
      kind: acceptance
      text: Harmonic-rich tones with a weak fundamental are detected at the fundamental (no octave error)
    - id: 367d698795f6c73776a5663565ae283822557465e12f2a395e228f8839837ecf
      kind: acceptance
      text: Note mapping and nearest_string return correct note names, octaves and cents for both instruments
    - id: 7c8ddaa80b6a7fb5b803d974b86aa108e3a1cb7236e73a66c29574bbf58fd9bb
      kind: acceptance
      text: detect performs no heap allocation after construction
    - id: 7e2704ce34a49dd60dfc1a0a7371fb3e7e4e210bb648c2673a4e0e00e49a20e0
      kind: acceptance
      text: cargo test and cargo clippy pass with no warnings
    - id: 6ab1f8f73ccae62f5e3d8de8501986ebe3423bacd9eff7b778182df98c101e58
      kind: acceptance
      text: With about 30 dB SNR noise every string is detected within ±1 cent; with 10 dB SNR single frames are within ±6 cents and the mean over 20 frames within ±1 cent; silence and pure noise return None
previous_contracts:
  - id: c66ade0f62ac5db79dfc97a291db1ac3405d9b89e0d5315a147abb8b66b368c0
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
      - id: 9ef5491702ab91fd117d842685347318a98c1d042cd396813100b385d1422b10
        kind: acceptance
        text: PitchDetector detects pure sines at every guitar and ukulele standard-tuning string within ±1 cent at 44.1 and 48 kHz
      - id: bf01ebcba4fe05aa352ff92177257e869eff2dcbc00cecd76806480cdc1ee834
        kind: acceptance
        text: Harmonic-rich tones with a weak fundamental are detected at the fundamental (no octave error)
      - id: 1dcf5455bcd482534e332f1d52c32ab6003d4dcab1e00e37b4fc933732186dc8
        kind: acceptance
        text: Tones with additive noise (about 10 dB SNR) are detected within ±3 cents; silence and pure noise return None
      - id: 367d698795f6c73776a5663565ae283822557465e12f2a395e228f8839837ecf
        kind: acceptance
        text: Note mapping and nearest_string return correct note names, octaves and cents for both instruments
      - id: 7c8ddaa80b6a7fb5b803d974b86aa108e3a1cb7236e73a66c29574bbf58fd9bb
        kind: acceptance
        text: detect performs no heap allocation after construction
      - id: 7e2704ce34a49dd60dfc1a0a7371fb3e7e4e210bb648c2673a4e0e00e49a20e0
        kind: acceptance
        text: cargo test and cargo clippy pass with no warnings
by: claude-code/opus-5.5
at: 2026-10-15T17:36:51Z
baseline: 20261015-173650
---
