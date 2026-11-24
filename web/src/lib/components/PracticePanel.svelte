<script lang="ts">
	import { untrack } from 'svelte';
	import type { Key } from '#lib/theory/key.ts';
	import { ROOTS } from '#lib/theory/lookup.ts';
	import { chordsIn, progressionsFor, type Progression } from '#lib/practice/progressions.ts';
	import { minutesText, type ChordRecord, type WeekDay } from '#lib/practice/progress.ts';
	import type { Target } from '#lib/practice/target.ts';
	import ChordDiagram from './ChordDiagram.svelte';

	interface Props {
		days: WeekDay[];
		daily: Target;
		/** The chord of the day was played already. */
		dailyPlayed: boolean;
		/** Pairs to drill, slow ones first, with their best counts. */
		pairs: { a: Target; b: Target; best: number | null }[];
		/** The key heard while playing, if any. */
		heardKey: Key | null;
		myChords: (ChordRecord & { key: string })[];
		onplay: (target: Target) => void;
		ondrill: (a: Target, b: Target) => void;
		onprogression: (p: Progression, tonic: number) => void;
		onchord: (key: string) => void;
	}
	let {
		days,
		daily,
		dailyPlayed,
		pairs,
		heardKey,
		myChords,
		onplay,
		ondrill,
		onprogression,
		onchord
	}: Props = $props();

	const NAMES = ['G', 'C', 'E', 'A'];
	// Start in the key heard while playing (else C).
	let tonic = $state(untrack(() => heardKey?.tonic ?? 0));
	let progressions = $derived(progressionsFor(heardKey));
	let minutes = $derived(minutesText(days.reduce((s, d) => s + d.seconds, 0)));
	let played = $derived(days.filter((d) => d.seconds > 0).length);
	const MINOR_NAMES = ['C', 'C#', 'D', 'Eb', 'E', 'F', 'F#', 'G', 'G#', 'A', 'Bb', 'B'];
	const keyName = (p: Progression) =>
		p.mode === 'major' ? `${ROOTS[tonic]} major` : `${MINOR_NAMES[tonic]} minor`;
</script>

<section class="block">
	<h3>This week</h3>
	<p class="muted" data-testid="practice-week">
		{#if played}
			{minutes} over {played} {played === 1 ? 'day' : 'days'}. Nice.
		{:else}
			Nothing yet this week. A few minutes a day adds up.
		{/if}
	</p>
</section>

<section class="block daily">
	<ChordDiagram names={NAMES} frets={daily.frets} small label="{daily.symbol} shape" />
	<div>
		<h3>Chord of the day</h3>
		<p class="big" data-testid="daily-chord">{daily.symbol}</p>
		<p class="muted">{dailyPlayed ? 'You know this one; play it again.' : 'One to learn today.'}</p>
		<button type="button" class="primary" onclick={() => onplay(daily)}>Play it</button>
	</div>
</section>

<section class="block">
	<h3>One-minute changes</h3>
	<p class="muted">Switch between two chords as often as you can in a minute.</p>
	<ul class="pairs" data-testid="drill-pairs">
		{#each pairs as p (p.a.symbol + p.b.symbol)}
			<li>
				<button type="button" onclick={() => ondrill(p.a, p.b)}>
					<strong>{p.a.symbol} ⇄ {p.b.symbol}</strong>
					<span class="muted">{p.best ? `best ${p.best}` : 'not tried'}</span>
				</button>
			</li>
		{/each}
	</ul>
</section>

<section class="block">
	<div class="row">
		<h3>Play along</h3>
		<label class="key">
			Key
			<select bind:value={tonic} aria-label="Key">
				{#each ROOTS as r, pc (pc)}<option value={pc}>{r}</option>{/each}
			</select>
		</label>
	</div>
	<p class="muted">
		{heardKey ? `You were playing in ${heardKey.name}.` : 'Pick a key.'} The next chord lights up as you
		play.
	</p>
	<ul class="progressions" data-testid="progressions">
		{#each progressions as p (p.name)}
			<li>
				<button type="button" onclick={() => onprogression(p, tonic)}>
					<span class="pname">{p.name} <span class="muted">· {keyName(p)}</span></span>
					<span class="chords"
						>{chordsIn(p, tonic)
							.map((t) => t.symbol)
							.join(' · ')}</span
					>
				</button>
			</li>
		{/each}
	</ul>
</section>

<section class="block">
	<h3>My chords <span class="muted">({myChords.length})</span></h3>
	{#if myChords.length}
		<ul class="mine" data-testid="my-chords">
			{#each myChords as c (c.key)}
				<li>
					<button
						type="button"
						disabled={c.key.startsWith('symbol:')}
						title="First played {c.first}"
						onclick={() => onchord(c.key)}>{c.symbol}</button
					>
				</li>
			{/each}
		</ul>
	{:else}
		<p class="muted">Chords you play show up here.</p>
	{/if}
</section>

<style>
	.block {
		background: var(--surface);
		border: 1px solid var(--line);
		border-radius: var(--radius);
		padding: 14px 16px;
		display: grid;
		gap: 8px;
	}
	h3 {
		margin: 0;
		font-size: 1rem;
		font-weight: 650;
	}
	.muted {
		margin: 0;
		color: var(--muted);
		font-size: 0.9rem;
		font-weight: 400;
	}
	.daily {
		grid-template-columns: auto 1fr;
		align-items: center;
		gap: 16px;
	}
	.daily > div {
		display: grid;
		gap: 4px;
		justify-items: start;
	}
	.big {
		margin: 0;
		font-size: 2rem;
		font-weight: 700;
		line-height: 1.1;
	}
	.primary {
		margin-top: 4px;
		min-height: 40px;
		padding: 0 18px;
		border: 0;
		border-radius: 999px;
		background: var(--accent);
		color: #fff;
		font-weight: 600;
		cursor: pointer;
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
	}
	.pairs {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
		gap: 8px;
	}
	.pairs button,
	.progressions button {
		width: 100%;
		display: grid;
		gap: 2px;
		padding: 10px 12px;
		border: 1px solid var(--line);
		border-radius: 10px;
		background: transparent;
		color: var(--text);
		text-align: left;
		cursor: pointer;
	}
	.pairs .muted {
		font-size: 0.8rem;
	}
	.progressions {
		display: grid;
		gap: 8px;
	}
	.pname {
		font-weight: 600;
	}
	.chords {
		color: var(--muted);
		font-size: 0.9rem;
	}
	.row {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}
	.key {
		display: flex;
		align-items: center;
		gap: 6px;
		color: var(--muted);
		font-size: 0.9rem;
	}
	select {
		min-height: 36px;
		padding: 0 8px;
		border: 1px solid var(--line);
		border-radius: 8px;
		background: var(--surface);
		color: var(--text);
		font: inherit;
	}
	.mine {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}
	.mine button {
		padding: 4px 12px;
		border: 0;
		border-radius: 999px;
		background: var(--line);
		color: var(--text);
		font-weight: 600;
		cursor: pointer;
	}
	.mine button:disabled {
		cursor: default;
	}
</style>
