---
type: decision
title: Rust toolchain pinned in rust-toolchain.toml
description: CI installs stable (dtolnay/rust-toolchain@stable) while this environment had 1.97; Rust 1.100's new clippy lint failed CI on code that passed locally. rust-toolchain.toml pins 1.100.0 (clippy, rustfmt, wasm32 target), which rustup honours both locally and in CI. Bump it deliberately and fix new lints in the same commit.
date: "2026-11-20"
scope:
  - '**'
---


