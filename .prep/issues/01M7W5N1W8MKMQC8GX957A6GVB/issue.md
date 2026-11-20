---
title: Faster note lock in single-string mode
kind: code
parent: 01M509SB0RMTVQ2PY4M21KKQAA
tags:
  - latency
priority: high
---

## Requirement

A plucked string shows a reading as close to instantly as possible; today it takes about a second. On paper the path is fast (2048-sample window, 21 ms hop, lock after 2 confident frames, median of 5, 120 ms EMA: roughly 100–200 ms), so the delay comes from somewhere the numbers don't show: the attack frames failing the onset clarity/margin gates, the noise floor, the UI median/EMA, or CSS transitions. Measure first (tuner-lab "time to first reading" on synthetic and real plucks, plus an end-to-end browser timing), then cut the biggest contributors.

Target: first reading within 150 ms of the pluck on the 95th percentile, with no more wrong first readings (octave errors) than today.

## Open questions

- Is a briefly rough first reading (that settles over the next ~100 ms) acceptable in exchange for speed?
- Can the user record a few plucks per string (tuner-lab protocol) so we tune against real attacks?
- Does "about a second" mean until the needle first appears, or until it settles?
