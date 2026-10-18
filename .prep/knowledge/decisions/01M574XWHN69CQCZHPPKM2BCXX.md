---
type: decision
title: Delegate implementation to Codex agents in git worktrees; Claude orchestrates
description: 'User (2026-10-18): Codex implements and a second Codex reviews each issue in its own git worktree, at most 2-3 agents at a time; finished worktrees merge into main locally and are deleted'
date: "2026-10-18"
---

User instruction (2026-10-18), reason: more usage available on Codex. Claude only orchestrates: makes issues ready, creates a worktree and branch per issue (`../tuner-wt-<slug>`, `issue/<slug>`), pre-installs dependencies, and starts `codex exec -C <worktree> -s danger-full-access` with the prompt "run prep guide <id>, implement it, log ready for check and stop" plus the repository rules. When the implementer is done, a second Codex agent reviews in the same worktree ("run prep guide <id> and follow its checker section"). Then the branch is merged into main locally (no push) and the worktree deleted. At most 2-3 agents run at once so the machine isn't overloaded; each gets its own Playwright port via E2E_PORT (added to web/playwright.config.ts for this).
