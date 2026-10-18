---
title: Polyphonic strum check
kind: code
parent: 01M509SB0RMTVQ2PY4M21KKQAA
depends_on:
  - 01M509SBVQFQ26T79APJR7NNPK
tags:
  - strum
priority: high
---

Strum all strings (standard tuning, guitar or ukulele as selected) and see per string whether it is in tune, flat or sharp, plus its offset in cents, based on the approach chosen in the polyphonic research (01M509SBVQFQ26T79APJR7NNPK). The app gets a mode switch between single-string tuning (unchanged) and strum check. A strum is detected automatically; its results stay on screen until the next strum. Strings that were not played (muted or missed) are shown as not heard rather than with a made-up reading. Analysis runs off the audio thread so the worklet never glitches, and the WASM module stays small.

Built and validated on synthetic strums first; thresholds are re-checked against the user's real recordings (tuner-lab) before the feature is called done.

## Open questions
