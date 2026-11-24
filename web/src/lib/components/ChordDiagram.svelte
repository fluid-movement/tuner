<script lang="ts" module>
	/** Tab lines for `frets`, highest string on top ("A|-0--|"). */
	export function tabLines(names: string[], frets: (number | null)[] | null): string[] {
		return names
			.map((name, i) => {
				const f = frets?.[i];
				const cell = !frets ? '-' : f === null || f === undefined ? 'x' : String(f);
				return `${name}|-${cell.padEnd(2, '-')}-|`;
			})
			.reverse();
	}
</script>

<script lang="ts">
	import type { ChordTone } from '#lib/theory/chord.ts';

	interface Props {
		/** String names in string order, without octave (G C E A). */
		names: string[];
		/** Fret per string (null = muted), or null for no chord. */
		frets: (number | null)[] | null;
		/** Note and role per string, shown under it (null: the string name). */
		tones?: (ChordTone | null)[];
		/** A smaller diagram, e.g. for alternative shapes. */
		small?: boolean;
		/** A thumbnail: no string names, for chord ideas. */
		tiny?: boolean;
		/** A preview: drawn faintly. */
		faint?: boolean;
		label?: string;
		testid?: string;
	}
	let {
		names,
		frets,
		tones = [],
		small = false,
		tiny = false,
		faint = false,
		label = 'Chord diagram',
		testid
	}: Props = $props();

	const FRETS_SHOWN = 4;
	/** First fret shown (1 unless the shape sits higher). */
	let base = $derived.by(() => {
		const fretted = (frets ?? []).filter((f): f is number => f !== null && f > 0);
		const top = fretted.length ? Math.max(...fretted) : 0;
		return top <= FRETS_SHOWN ? 1 : Math.min(...fretted);
	});
	/** Rows: one more when a stretched shape needs it. */
	let rows = $derived.by(() => {
		const fretted = (frets ?? []).filter((f): f is number => f !== null && f > 0);
		return fretted.length ? Math.max(FRETS_SHOWN, Math.max(...fretted) - base + 1) : FRETS_SHOWN;
	});

	const W = 160;
	const left = 24;
	const top = 34;
	const fretH = 30;
	let H = $derived(top + rows * fretH + (tiny ? 8 : 36));
	let gap = $derived((W - 2 * left) / Math.max(1, names.length - 1));
</script>

<svg
	viewBox="0 0 {W} {H}"
	class="diagram"
	class:small
	class:tiny
	class:faint
	role="img"
	aria-label={label}
	data-testid={testid}
>
	{#if base > 1}
		<text x="4" y={top + fretH / 2 + 4} class="base">{base}</text>
	{:else}
		<rect x={left - 1} y={top - 4} width={W - 2 * left + 2} height="5" class="nut" />
	{/if}
	{#each Array.from({ length: rows + 1 }, (_, i) => i) as i (i)}
		<line x1={left} x2={W - left} y1={top + i * fretH} y2={top + i * fretH} class="fret" />
	{/each}
	{#each names as name, i (i)}
		{@const x = left + i * gap}
		{@const f = frets?.[i]}
		<line x1={x} x2={x} y1={top} y2={top + rows * fretH} class="string" />
		{#if frets && f === null}
			<text {x} y={top - 12} class="mark">×</text>
		{:else if frets && f === 0}
			<circle cx={x} cy={top - 16} r="5" class="open" />
		{:else if frets && f !== undefined && f !== null}
			<circle cx={x} cy={top + (f - base + 0.5) * fretH} r="9" class="dot" />
		{/if}
		{#if tiny}
			<!-- No names: too small to read. -->
		{:else if tones[i]}
			<text {x} y={H - 18} class="note">{tones[i]!.name}</text>
			<text {x} y={H - 2} class="role">{tones[i]!.role}</text>
		{:else}
			<text {x} y={H - 18} class="label">{name}</text>
		{/if}
	{/each}
</svg>

<style>
	.diagram {
		width: 150px;
		height: auto;
	}
	.diagram.small {
		width: 96px;
	}
	.diagram.faint {
		opacity: 0.3;
	}
	.diagram.tiny {
		width: 52px;
	}
	.tiny .fret,
	.tiny .string {
		stroke-width: 4;
	}
	.fret,
	.string {
		stroke: var(--muted);
		stroke-width: 1.5;
	}
	.nut {
		fill: var(--text);
	}
	.dot {
		fill: var(--accent);
	}
	.open {
		fill: none;
		stroke: var(--text);
		stroke-width: 1.5;
	}
	.note {
		fill: var(--text);
		font-size: 14px;
		font-weight: 600;
		text-anchor: middle;
	}
	.role {
		fill: var(--accent);
		font-size: 12px;
		text-anchor: middle;
	}
	.label,
	.base {
		fill: var(--muted);
		font-size: 13px;
		text-anchor: middle;
	}
	.base {
		text-anchor: start;
	}
	.mark {
		font-size: 16px;
		fill: var(--text);
		text-anchor: middle;
	}
	/* Short phone screens. */
	@media (max-height: 700px) {
		.diagram:not(.small, .tiny) {
			width: 124px;
		}
	}
	/* Tall phones: room for a bigger diagram. */
	@media (min-height: 800px) {
		.diagram:not(.small, .tiny) {
			width: 176px;
		}
	}
</style>
