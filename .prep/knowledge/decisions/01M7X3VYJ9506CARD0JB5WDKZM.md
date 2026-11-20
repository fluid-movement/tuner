---
type: decision
title: Chord naming and theory live in TypeScript; WASM reports frets only
description: The chord namer and theory (names, alternatives, tone roles, spelling, key fit) move from tuner-core to web/src/lib/theory. tuner.wasm is at 158.7 of its 160 KB budget and name tables plus spelling would exceed it; theory is presentation-side and will grow (key detection, suggestions), and vitest makes it cheap to test. tuner-core keeps a pitch-set check only for chord_eval. Supersedes the in-core name_chord/Quality used by the wasm exports.
date: "2026-11-20"
issue: 01M7X3VRM4DW3WXC572XN22PM6
scope:
  - web/src/lib/theory/**
---


