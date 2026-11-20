---
outcome: done
by: claude-code
at: 2026-11-20T12:08:15Z
evidence: c1b43e4
---

Checked: svelte-check + tsc (service worker), eslint/prettier, vitest 12, Playwright 18 (installability via CDP apart from incognito, offline chord detection 12/12 repeats, wake lock). Criterion 3 (update on next launch) verified by design: per-build cache name, no skipWaiting, activate drops old caches; not covered by a browser test. nginx -t not run (no docker daemon here); the deploy e2e covers production.
