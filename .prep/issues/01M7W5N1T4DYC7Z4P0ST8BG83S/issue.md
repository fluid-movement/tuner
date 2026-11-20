---
title: Start listening on load, remove the Start button
kind: code
parent: 01M509SB0RMTVQ2PY4M21KKQAA
tags:
  - ux
priority: high
---

## Requirement

The app starts listening as soon as it loads; the Start button goes away. First visit shows the browser's microphone prompt straight away. Where the browser refuses to start audio without a gesture (AudioContext stays suspended, mainly iOS Safari), the app shows a full-screen "tap to start" hint that starts on the first tap anywhere, instead of a button. Denied permission and missing microphone keep their current error messages.

## Open questions

- Is the "tap anywhere" fallback acceptable where the browser demands a gesture, or should those browsers keep an explicit button?
