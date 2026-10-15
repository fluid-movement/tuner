---
type: component
title: UI
description: Svelte 5 components (CentsMeter, StringRow, InstrumentToggle), page state flow, CSS tokens and dark mode, layout rules
status: draft
generated:
  by: claude-code/opus-5.5
  at: 2026-10-15T17:58:39Z
scope:
  - web/src/lib/components
  - web/src/routes
  - web/src/app.css
---

Plain Svelte 5 (runes) and CSS; no UI library. Imports use the `#lib/...` alias with explicit `.ts` extensions (SvelteKit 3 template).

## Components (`web/src/lib/components/`)

- `CentsMeter.svelte`: SVG arc for −50..+50 cents over ±60°, ticks every 10 cents, a ±5-cent in-tune zone, and a needle rotated with a short CSS transition (disabled under `prefers-reduced-motion`). Props: `cents` (null = idle), `inTune`.
- `StringRow.svelte`: the instrument's strings as pills in string order; the active one has `aria-current="true"` and turns solid green when in tune. Each pill has `data-testid="string-<name>"`.
- `InstrumentToggle.svelte`: a segmented radio group (`bind:value`), visually hidden inputs with a focus ring on the label.

## Page state (`+page.svelte`)

`$state`: instrument, `Tunings` (instantiated on mount from the shared wasm bytes), engine, error, smoothed frequency. `$derived`: strings, nearest target and cents, note, in-tune (via `InTuneLatch`), status text. Readings from the engine go through `PitchSmoother` before touching state. Use `$state<T | null>(null)` with an explicit type, or TypeScript narrows the variable to `never`.

## Styling

`web/src/app.css` (imported in `+layout.svelte`) defines colour tokens on `:root` (`--bg`, `--surface`, `--text`, `--muted`, `--line`, `--accent`, `--in-tune`, `--off`) with a dark set under `prefers-color-scheme: dark`. Layout is a single column capped at 560 px with 16 px padding, which must fit 375 px wide without horizontal scrolling (checked in Playwright). Touch targets are at least 44 px.
