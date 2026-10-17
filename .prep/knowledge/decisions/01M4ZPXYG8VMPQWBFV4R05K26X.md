---
type: decision
title: "Smoothing in the UI, tuning lookups from Rust on the main thread"
description: "Smoothing in the UI, tuning lookups from Rust on the main thread"
date: "2026-10-15"
issue: 01M509SBJQQPMA0QNE3B4J1X41
---

Readings arrive ~47 times a second and single frames jitter by up to a few cents, so the UI takes the median of the last 5 readings in cents (rejects outliers such as octave slips in the decay), then applies an exponential moving average (time constant ~120 ms). A jump of more than 40 cents resets the filter so a new string shows immediately; no reading for 250 ms clears the display. In tune means |cents| ≤ 5 to enter and > 7 to leave. Smoothing stays in TS because it is presentation policy and the polyphonic mode will need different smoothing.

Tuning tables stay in Rust (`tuner-core::tuning`) and are reached through new WASM exports on a main-thread instance, rather than duplicated in TS. The bytes are already fetched for the worklet, so it costs one extra instantiation.
