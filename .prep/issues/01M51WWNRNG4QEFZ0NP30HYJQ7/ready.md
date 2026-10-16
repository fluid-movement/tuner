---
contract:
  id: 06752ff449011496be17b5cb601e62d7df06fc7ba89de69a62421565f92284f8
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
    - id: b433038eef9b1bd8e134c5213f01c5ee0bea050f573b1bb2e1fb2014c1728b24
      kind: acceptance
      text: On pull requests and pushes the check job runs just check
    - id: c921a18b26ac6c62a592fa69eef58578d13db3a2bd3f62ae5a77eaa67d395c30
      kind: acceptance
      text: On main, a passing check builds and pushes ghcr.io/fluid-movement/tuner with :latest and :<sha> tags
    - id: 7367061fd3e3fb40452e3e605c2164be32d3a71939fb62614e369d1e76d0a05e
      kind: acceptance
      text: The deploy job triggers Coolify, waits for version.txt to match the commit, then runs the browser tests against the live URL; it skips with a notice when secrets are missing
    - id: a7159eaf426cbc7f0449628a03bb27d05b1d01624f0cc07f98d0aae28395b0f3
      kind: acceptance
      text: The workflow passes actionlint; the image serves version.txt with the build's SHA
    - id: 3a34b310579cdd4956328e12ba558d25c4547c0e28b6da2e50196346b47ce39f
      kind: acceptance
      text: README documents the GitHub secrets/variables and the Coolify Docker Image resource setup
by: claude-code/opus-5.5
at: 2026-10-16T08:22:40Z
baseline: 20261016-082238
---
