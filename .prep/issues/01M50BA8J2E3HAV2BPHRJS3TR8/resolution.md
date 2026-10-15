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
verification:
  - id: 54903781c49efa31d1187c3994c180183e8ad9b1e0ce277f7108dcea857ddace
    contract: e304c2306f19bf580c27dec1c765820111d406be8af5f1b7e9be7294ae310623
    requirement: 0ed80fd753f73dbec8cfa49b721c5a6108f8f66de4875b158daaf9b0bc8e3aea
    result: pass
    method: automated
    command: just test
    observation: 'just check: cargo test all ok, vitest 13 passed; no detection change'
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-15T17:58:05.917868Z
    scopes:
      - justfile
      - README.md
    snapshot: 0354a32d8a822a54581fce3e17a2185d0172b3555322caa7ea37af624bba1f2b
  - id: 00cbb823e71d4ba311aec8087bc4f31ae241c8f505822796642d608ba747dfa9
    previous: 54903781c49efa31d1187c3994c180183e8ad9b1e0ce277f7108dcea857ddace
    contract: e304c2306f19bf580c27dec1c765820111d406be8af5f1b7e9be7294ae310623
    requirement: 5e5cff922ec43ceb8a22e0bea73e715418c9c2439bbb4c6322f3bb900c0dd43f
    result: pass
    method: manual
    observation: Knowledge review recorded for /overview.md
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-15T17:58:06.073265Z
    scopes:
      - justfile
      - README.md
    snapshot: 0354a32d8a822a54581fce3e17a2185d0172b3555322caa7ea37af624bba1f2b
  - id: 03590c50b06dd4509a287da0374533a277b24757ec79879b6b8bd9ee426bd8a7
    previous: 00cbb823e71d4ba311aec8087bc4f31ae241c8f505822796642d608ba747dfa9
    contract: e304c2306f19bf580c27dec1c765820111d406be8af5f1b7e9be7294ae310623
    requirement: f2d75eba11353969ea0ef3437e450159c5a6bedd6753d851f723906d42aea682
    result: pass
    method: automated
    command: prep check
    observation: 'prep check: 0 errors, 0 warnings'
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-15T17:58:06.207237Z
    scopes:
      - justfile
      - README.md
    snapshot: 0354a32d8a822a54581fce3e17a2185d0172b3555322caa7ea37af624bba1f2b
  - id: 82b7bdfde7e01cb574e1db2a6e38edc4507ea4b2b656d2c6e06e5fc6bc1496c9
    previous: 03590c50b06dd4509a287da0374533a277b24757ec79879b6b8bd9ee426bd8a7
    contract: e304c2306f19bf580c27dec1c765820111d406be8af5f1b7e9be7294ae310623
    requirement: 05a0f606567262d11ee0ce36cf1b66af16fb615ae20b91ff129806fdaa58e45e
    result: pass
    method: automated
    command: just clean && just dev --port 5197
    observation: After just clean, just dev ran setup, built wasm, served / (title Tuner) and /wasm/tuner.wasm 200
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-15T17:58:06.346807Z
    scopes:
      - justfile
      - README.md
    snapshot: 0354a32d8a822a54581fce3e17a2185d0172b3555322caa7ea37af624bba1f2b
  - id: a2f6e0e7634ce60bd1ad1df82401e1b4ad2e8eae400facfcbbe2b63dec2178e4
    previous: 82b7bdfde7e01cb574e1db2a6e38edc4507ea4b2b656d2c6e06e5fc6bc1496c9
    contract: e304c2306f19bf580c27dec1c765820111d406be8af5f1b7e9be7294ae310623
    requirement: 2a8974af01fb2d5e41ad097086a27b70700312dccee07a7987f7f64eb245d6d8
    result: pass
    method: automated
    command: just check
    observation: 'just check passed: fmt, clippy, svelte-check, lint, cargo test, vitest, build, playwright 7 passed'
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-15T17:58:06.488879Z
    scopes:
      - justfile
      - README.md
    snapshot: 0354a32d8a822a54581fce3e17a2185d0172b3555322caa7ea37af624bba1f2b
  - id: af31a05f9aaceba1c3b382f05d3e0ead631cf1cc08b55c674b54d025a4503e9f
    previous: a2f6e0e7634ce60bd1ad1df82401e1b4ad2e8eae400facfcbbe2b63dec2178e4
    contract: e304c2306f19bf580c27dec1c765820111d406be8af5f1b7e9be7294ae310623
    requirement: edbea6276406c5033d5f03f267cb854881231c6e8c3b71755f8dd546cfa0e66f
    result: pass
    method: automated
    command: just --list
    observation: just --list shows all 11 recipes with comments; README Quickstart has just dev / just check
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-15T17:58:06.648146Z
    scopes:
      - justfile
      - README.md
    snapshot: 0354a32d8a822a54581fce3e17a2185d0172b3555322caa7ea37af624bba1f2b
knowledge_reviews:
  - id: 35c38592a8748e5ec585b833725076c15d9f6a17d2a2e13851817d72271676a9
    contract: e304c2306f19bf580c27dec1c765820111d406be8af5f1b7e9be7294ae310623
    kind: updated
    entry: /overview.md
    entry_revision: 05b162e41e9b84536f6e5e576a62d6541bbffaec7ed4421e993aa9c2ea974308
    observation: Added Running section describing just dev / just check
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-15T17:57:58Z
    scopes:
      - README.md
      - justfile
    snapshot: 0354a32d8a822a54581fce3e17a2185d0172b3555322caa7ea37af624bba1f2b
generation: 1
outcome: done
by: claude-code/opus-5.5
at: 2026-10-15T17:58:06Z
documentation:
  entries:
    - /overview.md
dod:
  - prep check passes
---
