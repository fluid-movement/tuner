---
title: Start listening on load, remove the Start button
kind: code
parent: 01M509SB0RMTVQ2PY4M21KKQAA
tags:
  - ux
priority: high
---

## Requirement

The app starts listening as soon as it loads; the Start (and Stop) button goes away. First visit shows the browser's microphone prompt straight away. Where the browser refuses to start audio without a gesture (AudioContext stays suspended, mainly iOS Safari), the app shows a "Tap anywhere to start listening" overlay; the first tap anywhere starts audio and still acts on what was tapped. Denied permission and missing microphone keep their error messages, with a "Try again" button.

## Open questions
