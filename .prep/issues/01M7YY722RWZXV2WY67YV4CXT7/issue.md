---
title: Release the microphone when the app isn't focused; chord mode keeps the last chord shown
kind: code
tags:
  - ux
  - chord
priority: high
---

User request (2026-11-21): (1) In PWA/browser, use the microphone only while the app is focused: when the page is hidden or the window loses focus, stop the engine (mic tracks stopped, OS mic indicator off); restart it when focus returns. Chord history and the shown chord survive the pause. (2) Chord mode: keep the current chord displayed until the next chord is recognised; a new strum that is pending or yields no chord does not clear it (the status line still says Listening… / Couldn't make out a chord).

## Open questions
