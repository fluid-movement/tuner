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
	import { lookupTarget } from '#lib/theory/lookup.ts';
	import type { Target, TargetSource } from '#lib/practice/target.ts';
	import type { Change } from '#lib/practice/progress.ts';
	import ChordDiagram, { tabLines } from './ChordDiagram.svelte';
	import LearnToggle from './LearnToggle.svelte';

	interface Props {
		/** Ukulele strings in string order (G C E A). */
		strings: TunerString[];
		/** Latest chord report (early or settled), or null before a strum. */
		report: StrumReport | null;
		/** The report whose chord is on display (kept until the next chord). */
		shown: StrumReport | null;
		/** Settled chords, oldest first. */
		history: HistoryEntry[];
		/** Whether audio is flowing. */
		listening: boolean;
		/** Switches to the strum tuning check. */
		onTune?: () => void;
		/** Show the theory: what chords are, the key, why ideas fit. */
		learn: boolean;
		/** Aims at a chord (an idea or a history chord). */
		onTarget?: (root: number, suffix: string, source: TargetSource) => void;
		/** The chord being aimed at, previewed faintly before anything is played. */
		aimed?: Target | null;
		/** Changes each time the aimed chord lands (replays the landed moment). */
		landedKey?: number | null;
		/** The chord on display is played for the first time. */
		isNew?: boolean;
		/** The change into the chord on display, if quick. */
		change?: Change | null;
		/** Just tuned: invite the first strum. */
		ready?: boolean;
	}
	let {
		strings,
		report,
		shown,
		history,
		listening,
		onTune,
		learn = $bindable(),
		onTarget,
		aimed = null,
		landedKey = null,
		isNew = false,
		change = null,
		ready = false
	}: Props = $props();

	let chord = $derived(shown?.chord ?? null);
	let analysis = $derived(chord?.analysis ?? null);
	/** Nothing played yet: preview the aimed chord. */
	let ghost = $derived(!analysis && !!aimed);
	/** The key of the recent chords and what could come next. */
	let key = $derived(detectKey(history.map((h) => h.analysis)));
	let next = $derived(key && history.length ? suggestNext(key, history.at(-1)!.analysis) : []);
	/** Strings clearly out of tune, from a settled chord. */
	let off = $derived(
		shown?.settled && chord
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
	const noteNames = $derived(strings.map((s) => s.name.replace(/-?\d+$/, '')));
	let tab = $derived(tabLines(noteNames, chord?.frets ?? null));
	/** The last chord's numeral in the key, for "V → I". */
	let fromNumeral = $derived(key && history.length ? numeralIn(key, history.at(-1)!.analysis) : '');
</script>

<section class="chord" aria-live="polite">
	<div class="top">
		<p class="prompt" data-testid="chord-status">
			{#if report && !report.settled}
				Listening…
			{:else if report && !report.chord}
				Couldn't make out a chord · strum again
			{:else if report}
				Strum the next chord
			{:else if ready && listening}
				Ready · strum a chord
			{:else if aimed && listening}
				Strum {aimed.symbol}
			{:else if listening}
				Strum a chord
			{:else}
				Starting microphone…
			{/if}
		</p>
		<LearnToggle bind:on={learn} />
	</div>

	{#key landedKey}
		<div class="name" class:pending={shown && !shown.settled} class:landed={landedKey !== null}>
			<span data-testid="chord-name"
				>{#if analysis}{analysis.symbol}{:else if ghost}<span class="ghost-name"
						>{aimed!.symbol}</span
					>{:else}<span class="placeholder">–</span>{/if}</span
			>{#if analysis && isNew}<span class="new" data-testid="chord-new">new</span>{/if}
		</div>
	{/key}
	{#if change && analysis}
		<p class="change" data-testid="chord-change">
			{change.pair} · {change.seconds.toFixed(1)} s
			{#if change.best}<span class="best">· best</span>{/if}
		</p>
	{/if}
	{#if analysis && learn}
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
		<ChordDiagram
			names={noteNames}
			frets={chord?.frets ?? (ghost ? aimed!.frets : null)}
			tones={learn ? perString : []}
			faint={ghost}
			testid="chord-diagram"
		/>

		<pre class="tab" data-testid="chord-tab">{tab.join('\n')}</pre>
	</div>

	{#if key && learn}
		<p class="keys" data-testid="chord-key">
			Key: <strong>{key.name}</strong>
			<span class="sure">{key.confidence >= 0.3 ? 'likely' : 'maybe'}</span>
		</p>
	{:else if analysis && learn}
		<p class="keys" data-testid="chord-keys">
			{#if analysis.keys.length}
				In {analysis.keys
					.slice(0, 3)
					.map((k) => `${k.key} (${k.numeral})`)
					.join(' · ')}
			{:else}
				Outside any single major key
			{/if}
		</p>
	{/if}
	{#if next.length}
		<div class="ideas">
			<p class="ideas-label">Try next</p>
			<ol class="next" class:learn aria-label="Next chord ideas" data-testid="chord-next">
				{#each next as s (s.symbol)}
					<li>
						<button
							type="button"
							onclick={() => onTarget?.(s.root, s.suffix, 'idea')}
							aria-label="Play {s.symbol} next"
						>
							{#if s.frets}<ChordDiagram names={noteNames} frets={s.frets} tiny label="" />{/if}
							<span class="idea">
								<strong>{s.symbol}</strong>
								{#if learn}
									<span class="numeral">{s.numeral}</span>
									<span class="why" data-testid="chord-next-reason"
										>{fromNumeral} → {s.numeral} · {s.reason}</span
									>
								{/if}
							</span>
						</button>
					</li>
				{/each}
			</ol>
		</div>
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
				{@const target = lookupTarget(h.analysis)}
				<li>
					<button
						type="button"
						disabled={!target || !onTarget}
						onclick={() => target && onTarget?.(target.root, target.suffix, 'history')}
					>
						{h.analysis.symbol}{#if h.count > 1}<span class="count">×{h.count}</span>{/if}
						{#if key && learn}<span class="numeral">{numeralIn(key, h.analysis)}</span>{/if}
					</button>
				</li>
			{/each}
		</ol>
	{/if}
</section>

<style>
	.chord {
		position: relative;
		background: var(--surface);
		border: 1px solid var(--line);
		border-radius: var(--radius);
		padding: 16px;
		display: grid;
		gap: 12px;
		text-align: center;
	}
	.top {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		text-align: left;
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
	.name.landed {
		color: var(--in-tune);
		animation: ring 600ms ease-out;
	}
	@keyframes ring {
		from {
			text-shadow: 0 0 0 color-mix(in srgb, var(--in-tune) 60%, transparent);
			transform: scale(1.06);
		}
		to {
			text-shadow: 0 0 28px transparent;
			transform: scale(1);
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.name.landed {
			animation: none;
		}
	}
	.new {
		margin-left: 8px;
		vertical-align: super;
		font-size: 0.8rem;
		font-weight: 600;
		letter-spacing: 0.04em;
		text-transform: uppercase;
		color: var(--accent);
	}
	.ghost-name {
		opacity: 0.25;
	}
	.change {
		margin: -6px 0 0;
		color: var(--muted);
		font-size: 0.9rem;
		font-variant-numeric: tabular-nums;
	}
	.best {
		color: var(--in-tune);
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
	.next li button {
		height: 100%;
		width: 100%;
		display: grid;
		justify-items: center;
		gap: 4px;
		padding: 6px 10px;
		border: 1px solid var(--line);
		border-radius: 10px;
		background: transparent;
		color: var(--text);
		cursor: pointer;
	}
	.next.learn {
		display: grid;
		justify-content: stretch;
	}
	.next.learn li button {
		display: flex;
		align-items: center;
		gap: 12px;
		text-align: left;
	}
	.next.learn li button :global(svg) {
		flex: none;
	}
	.idea {
		display: grid;
		grid-template-columns: auto 1fr;
		align-items: baseline;
		column-gap: 6px;
	}
	.why {
		grid-column: 1 / -1;
		color: var(--muted);
		font-size: 0.85rem;
	}
	.ideas {
		display: grid;
		gap: 6px;
	}
	.ideas-label {
		margin: 0;
		color: var(--muted);
		font-size: 0.85rem;
	}
	.numeral {
		margin-left: 4px;
		font-size: 0.8em;
		opacity: 0.75;
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
	.history li button {
		padding: 3px 10px;
		border-radius: 999px;
		background: var(--line);
		font-weight: 600;
		font-size: 0.9rem;
		border: 0;
		color: inherit;
		cursor: pointer;
	}
	.history li button:disabled {
		color: inherit;
		cursor: default;
	}
	.history li:last-child button {
		background: var(--accent);
		color: #fff;
	}
	/* Short phone screens: everything fits without scrolling. */
	@media (max-height: 700px) {
		.chord {
			padding: 12px;
			gap: 8px;
		}
		.name {
			font-size: 3.4rem;
		}
		.tab {
			font-size: 0.95rem;
		}
	}
	/* Tall phones: use the room for a bigger name and diagram. */
	@media (min-height: 800px) {
		.chord {
			gap: 16px;
			padding: 20px 16px;
		}
		.tab {
			font-size: 1.2rem;
		}
	}
</style>
