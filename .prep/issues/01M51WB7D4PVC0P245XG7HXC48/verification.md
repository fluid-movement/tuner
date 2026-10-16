---
evidence:
  - id: 3a6626e09e8e7bba24eab92239623701c274744c291404a0d9cdd9cb03049603
    contract: c3356c859166a9d7067b5d5fe694ff7e4fd703cc5b66019006a02e122d9e1309
    requirement: 91fe8280ebdd6d0fe8c2a732973df35e75eb82bdc20590822112c9f489f998c9
    result: pass
    method: automated
    command: cargo test -p tuner-lab; ffmpeg -c:a flac + just analyze
    observation: WAV mono 16-bit and stereo 24-bit 44.1 kHz tests pass; a demo take converted to FLAC analyses identically (7 events, 4 plucks, 3 strums); events table shows kind, sounding strings, clarity
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-16T10:30:21.390595Z
    scopes:
      - crates/tuner-lab
      - crates/tuner-core/src/tracker.rs
      - justfile
      - README.md
    snapshot: 19274b858554fa6725356cd2fea2dd884177d1824aa264fa3805f29490d12870
  - id: 15818f8a8bb74f82203c056857c85e50dd0c1f8c48c2ad3eac957e707d967b31
    previous: 3a6626e09e8e7bba24eab92239623701c274744c291404a0d9cdd9cb03049603
    contract: c3356c859166a9d7067b5d5fe694ff7e4fd703cc5b66019006a02e122d9e1309
    requirement: 86dd641d1dae95b57795c41ab202b1cfb7285f8c59fcb6ebe6fc8c52cd3bfac4
    result: pass
    method: automated
    command: cargo test -p tuner-lab
    observation: 'guitar (E4 -12, B3 +9), ukulele and stereo takes: exact event kinds/onsets, pluck truth within ±2 c, >=90% shown, <=100 ms latency, strum error <=3 c, no differing verdicts'
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-16T10:30:21.63235Z
    scopes:
      - crates/tuner-lab
      - crates/tuner-core/src/tracker.rs
      - justfile
      - README.md
    snapshot: 19274b858554fa6725356cd2fea2dd884177d1824aa264fa3805f29490d12870
  - id: 50f6b55834c6d05f1ce686deb73f2bcbd04a8fe92f9df79ad9c15c778bddbf59
    previous: 15818f8a8bb74f82203c056857c85e50dd0c1f8c48c2ad3eac957e707d967b31
    contract: c3356c859166a9d7067b5d5fe694ff7e4fd703cc5b66019006a02e122d9e1309
    requirement: b191a3472d2ef6f0acb9fe73828fec37296709a6086585eb4c6bdb390ce10331
    result: pass
    method: automated
    command: just analyze <demo dir>; just analyze <file> --json
    observation: Markdown per take (events, single-string latency/shown/spread/drift, strum tables) and summary per instrument; --json emits the same data
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-16T10:30:21.841506Z
    scopes:
      - crates/tuner-lab
      - crates/tuner-core/src/tracker.rs
      - justfile
      - README.md
    snapshot: 19274b858554fa6725356cd2fea2dd884177d1824aa264fa3805f29490d12870
  - id: f0330476ad78325bbe52cd52a6eb5c89154b94af81aeb0cfd46d85fbbcd6a4e9
    previous: 50f6b55834c6d05f1ce686deb73f2bcbd04a8fe92f9df79ad9c15c778bddbf59
    contract: c3356c859166a9d7067b5d5fe694ff7e4fd703cc5b66019006a02e122d9e1309
    requirement: 54626f249505a46fc2c66cb497d987f74200fba542e0668d46f92fa5252e3c69
    result: pass
    method: automated
    command: just analyze; just analyze-demo
    observation: with empty recordings/ prints 'No WAV or FLAC files in recordings.' plus the protocol; analyze-demo writes three takes
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-16T10:30:22.040133Z
    scopes:
      - crates/tuner-lab
      - crates/tuner-core/src/tracker.rs
      - justfile
      - README.md
    snapshot: 19274b858554fa6725356cd2fea2dd884177d1824aa264fa3805f29490d12870
  - id: 3f4c5eb8c004c8e3db62460fbd84f5645b7042119cf9da4a64a1ef95be685836
    previous: f0330476ad78325bbe52cd52a6eb5c89154b94af81aeb0cfd46d85fbbcd6a4e9
    contract: c3356c859166a9d7067b5d5fe694ff7e4fd703cc5b66019006a02e122d9e1309
    requirement: bd159bb3007442da21e933008db435fc5f6562288ab2b8046eaa001e86912f99
    result: pass
    method: automated
    command: just check; cargo tree -p tuner-core --depth 1
    observation: just check exit 0 (core 28, alloc 2, lab 4, wasm 4, vitest, build, e2e); tuner-core still depends only on rustfft
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-16T10:30:22.247726Z
    scopes:
      - crates/tuner-lab
      - crates/tuner-core/src/tracker.rs
      - justfile
      - README.md
    snapshot: 19274b858554fa6725356cd2fea2dd884177d1824aa264fa3805f29490d12870
  - id: 496286926f04ecf24de5e850ac2f5eef7a438dbfad0841079bae48fe48a617cf
    previous: 3f4c5eb8c004c8e3db62460fbd84f5645b7042119cf9da4a64a1ef95be685836
    contract: c3356c859166a9d7067b5d5fe694ff7e4fd703cc5b66019006a02e122d9e1309
    requirement: f2d75eba11353969ea0ef3437e450159c5a6bedd6753d851f723906d42aea682
    result: pass
    method: automated
    command: prep check
    observation: 0 errors, 0 warnings
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-16T10:30:22.458178Z
    scopes:
      - crates/tuner-lab
      - crates/tuner-core/src/tracker.rs
      - justfile
      - README.md
    snapshot: 19274b858554fa6725356cd2fea2dd884177d1824aa264fa3805f29490d12870
  - id: 307a583823cb307ae5acbe49810fb863a52394cead21601c6cead218714a2392
    previous: 496286926f04ecf24de5e850ac2f5eef7a438dbfad0841079bae48fe48a617cf
    contract: c3356c859166a9d7067b5d5fe694ff7e4fd703cc5b66019006a02e122d9e1309
    requirement: 5e5cff922ec43ceb8a22e0bea73e715418c9c2439bbb4c6322f3bb900c0dd43f
    result: pass
    method: manual
    observation: 'Knowledge reviews: tuner-lab (new), note-tracker, overview updated; single-string-tuner confirmed'
    provenance: reported
    by: claude-code/opus-5.5
    at: 2026-10-16T10:30:22.667408Z
    scopes:
      - crates/tuner-lab
      - crates/tuner-core/src/tracker.rs
      - justfile
      - README.md
    snapshot: 19274b858554fa6725356cd2fea2dd884177d1824aa264fa3805f29490d12870
---
