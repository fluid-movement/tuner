---
type: decision
title: Microphone only while the app is focused
description: 'User request: the app stops the audio engine (releasing the microphone) when the page is hidden or the window loses focus, and restarts it on focus. Stopping tracks rather than disabling them, because only stopping turns off the OS microphone indicator. On browsers that need a gesture for audio, returning shows the tap-to-start overlay.'
date: "2026-11-21"
issue: 01M7YY722RWZXV2WY67YV4CXT7
scope:
  - web/src/routes/+page.svelte
---


