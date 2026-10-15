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
verification:
  - id: 386548916e6fe0c1f63b3fbcb7d45ab86b988c8b54c79f82ab6639f267651ed8
    previous: d237859d442f5c69452faf6688e2e18b4de3d1fc70c83aa4d84329504a0dddb5
    contract: df2aa81d7251366cf3bc5d474573bf1f76d3aba34f5c9753feed4f65a9f0c351
    requirement: 0ed80fd753f73dbec8cfa49b721c5a6108f8f66de4875b158daaf9b0bc8e3aea
    result: pass
    method: automated
    command: cargo test && (cd web && pnpm test)
    observation: 'cargo: all test results ok (11 core unit, 1 no_alloc, 1 wasm); vitest 1 passed. Real-instrument check not possible yet (no mic pipeline/UI)'
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-15T17:44:35.480565Z
    scopes:
      - crates/tuner-core
    snapshot: 0d408514bf52a2a68b18eb46c64077ab7abb2fae8f91145aad1c7f48cabb2c4e
  - id: d237859d442f5c69452faf6688e2e18b4de3d1fc70c83aa4d84329504a0dddb5
    previous: fba8c00dfa7d3adb20945668ac36da9d5131f2dec8035b9edd178abe780f8d19
    contract: df2aa81d7251366cf3bc5d474573bf1f76d3aba34f5c9753feed4f65a9f0c351
    requirement: 5e5cff922ec43ceb8a22e0bea73e715418c9c2439bbb4c6322f3bb900c0dd43f
    result: pass
    method: manual
    observation: Knowledge reviews recorded for pitch-core, pitch-and-tunings, overview
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-15T17:43:50.64012Z
    scopes:
      - crates/tuner-core
    snapshot: 0d408514bf52a2a68b18eb46c64077ab7abb2fae8f91145aad1c7f48cabb2c4e
  - id: ab86864c7a1bd12caa1ab3f5174f1509e268ebba049eb744af9b153caf6afe09
    contract: df2aa81d7251366cf3bc5d474573bf1f76d3aba34f5c9753feed4f65a9f0c351
    requirement: f2d75eba11353969ea0ef3437e450159c5a6bedd6753d851f723906d42aea682
    result: pass
    method: automated
    command: prep check
    observation: 0 errors, 0 warnings
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-15T17:43:49.833389Z
    scopes:
      - crates/tuner-core
    snapshot: 0d408514bf52a2a68b18eb46c64077ab7abb2fae8f91145aad1c7f48cabb2c4e
  - id: 44ad55a94e2ef31b8b2b8e42e55f4b6cad2a3bca520e50e0f0cd3ea47b128730
    previous: ab86864c7a1bd12caa1ab3f5174f1509e268ebba049eb744af9b153caf6afe09
    contract: df2aa81d7251366cf3bc5d474573bf1f76d3aba34f5c9753feed4f65a9f0c351
    requirement: 9ef5491702ab91fd117d842685347318a98c1d042cd396813100b385d1422b10
    result: pass
    method: automated
    command: cargo test -p tuner-core
    observation: detects_sines_at_every_string and detects_detuned_sines pass (±1 cent, 44.1/48 kHz)
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-15T17:43:49.946315Z
    scopes:
      - crates/tuner-core
    snapshot: 0d408514bf52a2a68b18eb46c64077ab7abb2fae8f91145aad1c7f48cabb2c4e
  - id: 5be4eb0521a9b1f444f6e9cb62b575bf02e068fec9b593476b280af64e104ce5
    previous: 44ad55a94e2ef31b8b2b8e42e55f4b6cad2a3bca520e50e0f0cd3ea47b128730
    contract: df2aa81d7251366cf3bc5d474573bf1f76d3aba34f5c9753feed4f65a9f0c351
    requirement: bf01ebcba4fe05aa352ff92177257e869eff2dcbc00cecd76806480cdc1ee834
    result: pass
    method: automated
    command: cargo test -p tuner-core
    observation: weak_fundamental_is_not_an_octave_error passes for 3 spectra x all strings x 2 rates
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-15T17:43:50.065406Z
    scopes:
      - crates/tuner-core
    snapshot: 0d408514bf52a2a68b18eb46c64077ab7abb2fae8f91145aad1c7f48cabb2c4e
  - id: da39a9e593c1466951b274cec3d7ec1381b00b73c58435939a79cea536c9b6d0
    previous: 5be4eb0521a9b1f444f6e9cb62b575bf02e068fec9b593476b280af64e104ce5
    contract: df2aa81d7251366cf3bc5d474573bf1f76d3aba34f5c9753feed4f65a9f0c351
    requirement: 367d698795f6c73776a5663565ae283822557465e12f2a395e228f8839837ecf
    result: pass
    method: automated
    command: cargo test -p tuner-core
    observation: 'tuning tests pass: string_frequencies_match_their_midi_notes, notes_and_cents, nearest_guitar_string, nearest_ukulele_string_respects_reentrant_order'
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-15T17:43:50.180332Z
    scopes:
      - crates/tuner-core
    snapshot: 0d408514bf52a2a68b18eb46c64077ab7abb2fae8f91145aad1c7f48cabb2c4e
  - id: 816e3b4179f40394517e6f3bf688a6d43e0583aa5219cf8e3d26c0e2971b47c3
    previous: da39a9e593c1466951b274cec3d7ec1381b00b73c58435939a79cea536c9b6d0
    contract: df2aa81d7251366cf3bc5d474573bf1f76d3aba34f5c9753feed4f65a9f0c351
    requirement: 7c8ddaa80b6a7fb5b803d974b86aa108e3a1cb7236e73a66c29574bbf58fd9bb
    result: pass
    method: automated
    command: cargo test -p tuner-core --test no_alloc
    observation: detect_does_not_allocate passes (counting global allocator, 0 allocations for a tone and silence)
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-15T17:43:50.301808Z
    scopes:
      - crates/tuner-core
    snapshot: 0d408514bf52a2a68b18eb46c64077ab7abb2fae8f91145aad1c7f48cabb2c4e
  - id: 97feef78f742e892d1e7ca34ef084aa5df6cfbeb8e9d5e73c6d8cc22d49ad95c
    previous: 816e3b4179f40394517e6f3bf688a6d43e0583aa5219cf8e3d26c0e2971b47c3
    contract: df2aa81d7251366cf3bc5d474573bf1f76d3aba34f5c9753feed4f65a9f0c351
    requirement: 7e2704ce34a49dd60dfc1a0a7371fb3e7e4e210bb648c2673a4e0e00e49a20e0
    result: pass
    method: automated
    command: cargo test && cargo clippy --all-targets -- -D warnings
    observation: all tests ok, clippy no output
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-15T17:43:50.417317Z
    scopes:
      - crates/tuner-core
    snapshot: 0d408514bf52a2a68b18eb46c64077ab7abb2fae8f91145aad1c7f48cabb2c4e
  - id: fba8c00dfa7d3adb20945668ac36da9d5131f2dec8035b9edd178abe780f8d19
    previous: 97feef78f742e892d1e7ca34ef084aa5df6cfbeb8e9d5e73c6d8cc22d49ad95c
    contract: df2aa81d7251366cf3bc5d474573bf1f76d3aba34f5c9753feed4f65a9f0c351
    requirement: 6ab1f8f73ccae62f5e3d8de8501986ebe3423bacd9eff7b778182df98c101e58
    result: pass
    method: automated
    command: cargo test -p tuner-core
    observation: detects_through_moderate_noise, heavy_noise_stays_near_the_single_frame_limit_without_bias, silence_and_noise_have_no_pitch pass
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-15T17:43:50.528469Z
    scopes:
      - crates/tuner-core
    snapshot: 0d408514bf52a2a68b18eb46c64077ab7abb2fae8f91145aad1c7f48cabb2c4e
knowledge_reviews:
  - id: cc82af2668b912588fe488a76211adba6326c92e4b734d09c48e23bbd87f59a4
    contract: df2aa81d7251366cf3bc5d474573bf1f76d3aba34f5c9753feed4f65a9f0c351
    kind: updated
    entry: /components/pitch-core.md
    entry_revision: 3431f0e1f8151e240172a45fd3ff83729735049b6e1de8efebccb8da8736a8b2
    observation: New entry describing PitchDetector API, algorithm, accuracy and cost as implemented
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-15T17:43:41Z
    scopes:
      - crates/tuner-core
    snapshot: 0d408514bf52a2a68b18eb46c64077ab7abb2fae8f91145aad1c7f48cabb2c4e
  - id: e9a245d346d71ed4ea33778675c2d17ed18fbf4958682071e4ff19fda4ce279e
    previous: cc82af2668b912588fe488a76211adba6326c92e4b734d09c48e23bbd87f59a4
    contract: df2aa81d7251366cf3bc5d474573bf1f76d3aba34f5c9753feed4f65a9f0c351
    kind: updated
    entry: /conventions/pitch-and-tunings.md
    entry_revision: 1bddd9014e3976547df22b31c8d1059199bb72a913bf65cd274814d6eb95426f
    observation: New entry matching tuning.rs constants, cents sign, naming and nearest_string
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-15T17:43:41Z
    scopes:
      - crates/tuner-core
    snapshot: 0d408514bf52a2a68b18eb46c64077ab7abb2fae8f91145aad1c7f48cabb2c4e
  - id: 7cce456dcd6c59fc9c7fe3732b16f05f056dd170e2308a926242bd5ac43371f7
    previous: e9a245d346d71ed4ea33778675c2d17ed18fbf4958682071e4ff19fda4ce279e
    contract: df2aa81d7251366cf3bc5d474573bf1f76d3aba34f5c9753feed4f65a9f0c351
    kind: updated
    entry: /overview.md
    entry_revision: ed1c1b4bcca9a94b28925b6181e93fb99da71cb8d57b6828242e6203bc455e71
    observation: Linked the two new entries from Architecture
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-15T17:43:42Z
    scopes:
      - crates/tuner-core
    snapshot: 0d408514bf52a2a68b18eb46c64077ab7abb2fae8f91145aad1c7f48cabb2c4e
generation: 1
outcome: done
by: claude-code/opus-5.5
at: 2026-10-15T17:44:35Z
documentation:
  entries:
    - /components/pitch-core.md
    - /conventions/pitch-and-tunings.md
    - /overview.md
dod:
  - prep check passes
---
