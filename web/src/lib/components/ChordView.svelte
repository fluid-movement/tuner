<script lang="ts" module>
	import type { ChordAnalysis } from '#lib/theory/chord.ts';

	/** A chord in the history; consecutive repeats are counted. */
	export interface HistoryEntry {
		analysis: ChordAnalysis;
		count: number;
	}
</script>

<script lang="ts">
	import { detectKey, numeralIn, suggestNext } from '#lib/theory/key.ts';
	import type { StrumReport, TunerString } from '#lib/wasm/tuner.ts';
	import { describeOff, offStrings } from '#lib/tuner/intonation.ts';

	interface Props {
		/** Ukulele strings in string order (G C E A). */
		strings: TunerString[];
		/** Latest chord report (early or settled), or null before a strum. */
		report: StrumReport | null;
		/** Settled chords, oldest first. */
		history: HistoryEntry[];
		/** Whether audio is flowing. */
		listening: boolean;
		/** Switches to the strum tuning check. */
		onTune?: () => void;
	}
	let { strings, report, history, listening, onTune }: Props = $props();

	const FRETS_SHOWN = 4;
	let chord = $derived(report?.chord ?? null);
	let analysis = $derived(chord?.analysis ?? null);
	/** The key of the recent chords and what could come next. */
	let key = $derived(detectKey(history.map((h) => h.analysis)));
	let next = $derived(key && history.length ? suggestNext(key, history.at(-1)!.analysis) : []);
	/** Strings clearly out of tune, from a settled chord. */
	let off = $derived(
		report?.settled && chord
			? offStrings(
					strings.map((s) => s.name),
					chord.cents
				)
			: []
	);
	/** Sounding note and role per string (null = muted / no chord). */
	let perString = $derived.by(() => {
		const tones = analysis?.tones ?? [];
		let k = 0;
		return (chord?.frets ?? strings.map(() => null)).map((f) => (f === null ? null : tones[k++]));
	});
	let frets = $derived(chord?.frets ?? strings.map(() => null));
	/** First fret shown in the diagram (1 unless the shape sits higher). */
	let base = $derived.by(() => {
		const fretted = frets.filter((f): f is number => f !== null && f > 0);
		const top = fretted.length ? Math.max(...fretted) : 0;
		return top <= FRETS_SHOWN ? 1 : Math.min(...fretted);
	});
	/** Tab lines, highest string on top (A E C G). */
	let tab = $derived(
		strings
			.map((s, i) => {
				const f = frets[i];
				const cell = !chord ? '-' : f === null ? 'x' : String(f);
				return `${s.name.replace(/-?\d+$/, '')}|-${cell.padEnd(2, '-')}-|`;
			})
			.reverse()
	);
	const noteNames = $derived(strings.map((s) => s.name.replace(/-?\d+$/, '')));

	// Diagram geometry (SVG user units).
	const W = 160;
	const H = 190;
	const left = 24;
	const top = 34;
	const fretH = 30;
	let gap = $derived((W - 2 * left) / Math.max(1, strings.length - 1));
</script>

<section class="chord" aria-live="polite">
	<p class="prompt" data-testid="chord-status">
		{#if report && !report.settled}
			Listening…
		{:else if report && !chord}
			Couldn't make out a chord · strum again
		{:else if report}
			Strum the next chord
		{:else if listening}
			Strum a chord
		{:else}
			Starting microphone…
		{/if}
	</p>

	<div class="name" data-testid="chord-name" class:pending={report && !report.settled}>
		{#if analysis}
			{analysis.symbol}
		{:else}
			<span class="placeholder">–</span>
		{/if}
	</div>
	{#if analysis}
		<p class="about">
			<span data-testid="chord-description">{analysis.description}</span>
			{#if analysis.alternatives.length}
				<span class="also" data-testid="chord-alternatives"
					>also {analysis.alternatives.join(' · ')}</span
				>
			{/if}
		</p>
	{/if}

	<div class="shapes">
		<svg
			viewBox="0 0 {W} {H}"
			class="diagram"
			role="img"
			aria-label="Chord diagram"
			data-testid="chord-diagram"
		>
			{#if base > 1}
				<text x="4" y={top + fretH / 2 + 4} class="base">{base}</text>
			{:else}
				<rect x={left - 1} y={top - 4} width={W - 2 * left + 2} height="5" class="nut" />
			{/if}
			{#each Array.from({ length: FRETS_SHOWN + 1 }, (_, i) => i) as i (i)}
				<line x1={left} x2={W - left} y1={top + i * fretH} y2={top + i * fretH} class="fret" />
			{/each}
			{#each strings as s, i (s.index)}
				{@const x = left + i * gap}
				{@const f = frets[i]}
				<line x1={x} x2={x} y1={top} y2={top + FRETS_SHOWN * fretH} class="string" />
				{#if chord && f === null}
					<text {x} y={top - 12} class="mark">×</text>
				{:else if chord && f === 0}
					<circle cx={x} cy={top - 16} r="5" class="open" />
				{:else if chord && f !== null}
					<circle cx={x} cy={top + (f - base + 0.5) * fretH} r="9" class="dot" />
				{/if}
				{#if perString[i]}
					<text {x} y={H - 18} class="note">{perString[i]!.name}</text>
					<text {x} y={H - 2} class="role">{perString[i]!.role}</text>
				{:else}
					<text {x} y={H - 18} class="label">{noteNames[i]}</text>
				{/if}
			{/each}
		</svg>

		<pre class="tab" data-testid="chord-tab">{tab.join('\n')}</pre>
	</div>

	{#if key}
		<p class="keys" data-testid="chord-key">
			Key: <strong>{key.name}</strong>
			<span class="sure">{key.confidence >= 0.3 ? 'likely' : 'maybe'}</span>
		</p>
		{#if next.length}
			<ol class="next" aria-label="Next chord ideas" data-testid="chord-next">
				{#each next as s (s.symbol)}
					<li>
						<strong>{s.symbol}</strong> <span class="numeral">{s.numeral}</span>
						{#if s.frets}<span class="frets">{s.frets.join('')}</span>{/if}
					</li>
				{/each}
			</ol>
		{/if}
	{:else if analysis?.keys.length}
		<p class="keys" data-testid="chord-keys">
			In {analysis.keys
				.slice(0, 3)
				.map((k) => `${k.key} (${k.numeral})`)
				.join(' · ')}
		</p>
	{:else if analysis}
		<p class="keys" data-testid="chord-keys">Outside any single major key</p>
	{/if}

	{#if off.length}
		<p class="tune" data-testid="chord-intonation">
			Out of tune: {describeOff(off)}
			{#if onTune}<button type="button" onclick={onTune}>Tune</button>{/if}
		</p>
	{/if}

	{#if history.length}
		<ol class="history" aria-label="Recent chords" data-testid="chord-history">
			{#each history as h, i (i)}
				<li>
					{h.analysis.symbol}{#if h.count > 1}<span class="count">×{h.count}</span>{/if}
					{#if key}<span class="numeral">{numeralIn(key, h.analysis)}</span>{/if}
				</li>
			{/each}
		</ol>
	{/if}
</section>

<style>
	.chord {
		background: var(--surface);
		border: 1px solid var(--line);
		border-radius: var(--radius);
		padding: 16px;
		display: grid;
		gap: 12px;
		text-align: center;
	}
	.prompt {
		margin: 0;
		color: var(--muted);
		font-weight: 500;
	}
	.name {
		font-size: clamp(3.5rem, 18vw, 5.5rem);
		font-weight: 700;
		line-height: 1;
		transition: opacity 120ms;
	}
	.name.pending {
		opacity: 0.6;
	}
	.placeholder {
		color: var(--line);
	}
	.shapes {
		display: flex;
		flex-wrap: wrap;
		justify-content: center;
		align-items: center;
		gap: 16px 28px;
	}
	.diagram {
		width: 150px;
		height: auto;
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
	.about {
		margin: -4px 0 0;
		color: var(--muted);
		display: grid;
		gap: 2px;
	}
	.also {
		font-size: 0.9rem;
	}
	.keys {
		margin: 0;
		color: var(--muted);
		font-size: 0.9rem;
	}
	.mark,
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
	}
	.tab {
		margin: 0;
		font:
			1.05rem/1.35 ui-monospace,
			'SFMono-Regular',
			Menlo,
			monospace;
		text-align: left;
	}
	.tune {
		margin: 0;
		padding: 8px 12px;
		border-radius: 10px;
		background: color-mix(in srgb, var(--off) 14%, transparent);
		color: var(--text);
		font-size: 0.9rem;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-wrap: wrap;
		gap: 8px;
	}
	.tune button {
		border: 0;
		border-radius: 999px;
		padding: 4px 12px;
		background: var(--off);
		color: #fff;
		font: inherit;
		font-weight: 600;
		cursor: pointer;
	}
	.sure {
		font-size: 0.8rem;
		opacity: 0.8;
	}
	.next {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-wrap: wrap;
		justify-content: center;
		gap: 8px;
	}
	.next li {
		padding: 4px 10px;
		border: 1px solid var(--line);
		border-radius: 10px;
		font-size: 0.9rem;
	}
	.numeral {
		margin-left: 4px;
		font-size: 0.8em;
		opacity: 0.75;
	}
	.frets {
		margin-left: 6px;
		font:
			0.85em ui-monospace,
			monospace;
		color: var(--muted);
	}
	.count {
		margin-left: 2px;
		font-size: 0.8em;
	}
	.history {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-wrap: wrap;
		justify-content: center;
		gap: 6px;
	}
	.history li {
		padding: 3px 10px;
		border-radius: 999px;
		background: var(--line);
		font-weight: 600;
		font-size: 0.9rem;
	}
	.history li:last-child {
		background: var(--accent);
		color: #fff;
	}
</style>
