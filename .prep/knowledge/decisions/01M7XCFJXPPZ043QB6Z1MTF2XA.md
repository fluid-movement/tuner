---
type: decision
title: 'No float clamp in tuner-core: use max/min'
description: f32/f64::clamp's panic message pulls float formatting (~14 KB of dragon/grisu code) into tuner.wasm; replacing the two calls with max/min took the module from 161.2 to 135.5 KB. Keep panicking float formatting (clamp, {:?} of floats in asserts) out of non-test core code; check with twiggy when the wasm grows unexpectedly.
date: "2026-11-20"
issue: 01M7XAFZWM7YT3JAEFAWNGPXQV
scope:
  - crates/tuner-core/src/**
---


