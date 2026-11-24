<script lang="ts">
	import { CHORD_TYPES, ROOTS, lookUpChord } from '#lib/theory/lookup.ts';
	import ChordDiagram, { tabLines } from './ChordDiagram.svelte';

	import LearnToggle from './LearnToggle.svelte';
	import { chordKeyOf } from '#lib/practice/daily.ts';

	interface Props {
		/** Root pitch class (0 = C). */
		root: number;
		/** Chord-type suffix ('' for major). */
		suffix: string;
		/** Show the theory: description, notes and roles, keys. */
		learn: boolean;
		/** Collection keys of chords already played (see chordKeyOf). */
		played?: Set<string>;
		/** Aims at the chord on the listening screen. */
		onplay?: (root: number, suffix: string) => void;
	}
	let {
		root = $bindable(),
		suffix = $bindable(),
		learn = $bindable(),
		played = new Set(),
		onplay
	}: Props = $props();

	const NAMES = ['G', 'C', 'E', 'A'];
	/** Which shape is shown large (index into the shapes). */
	let picked = $state(0);
	/** "3203", or "7-10-8-7" once a fret has two digits. */
	const fretText = (frets: number[]) => frets.join(frets.some((f) => f > 9) ? '-' : '');

	let type = $derived(CHORD_TYPES.find((t) => t.suffix === suffix) ?? CHORD_TYPES[0]);
	let found = $derived(lookUpChord(root, type));
	// A new chord starts at its main shape.
	$effect(() => {
		void found;
		picked = 0;
	});
	let shape = $derived(found.shapes[picked] ?? found.shapes[0] ?? null);
	let analysis = $derived(shape?.analysis ?? null);
	/** The chord's notes, root first, each once. */
	let notes = $derived.by(() => {
		if (!analysis) return [];
		const tones = analysis.tones.map((t) => ({ ...t, interval: (t.pc - root + 12) % 12 }));
		return tones
			.filter((t, i) => tones.findIndex((u) => u.pc === t.pc) === i)
			.sort((a, b) => a.interval - b.interval);
	});
	/** Chord notes the shape leaves out (the 5th of a 9th chord). */
	let omitted = $derived(
		(type.optional ?? []).filter((i) => !notes.some((n) => n.interval === i)).length > 0
	);
</script>

<section class="lookup">
	<div class="pickers">
		<div class="roots" role="radiogroup" aria-label="Root" data-testid="lookup-roots">
			{#each ROOTS as name, pc (pc)}
				<button
					type="button"
					role="radio"
					aria-checked={root === pc}
					class:selected={root === pc}
					onclick={() => (root = pc)}>{name}</button
				>
			{/each}
		</div>
		<div class="types" role="radiogroup" aria-label="Chord type" data-testid="lookup-types">
			{#each CHORD_TYPES as t (t.suffix)}
				<button
					type="button"
					role="radio"
					aria-checked={suffix === t.suffix}
					class:selected={suffix === t.suffix}
					onclick={() => (suffix = t.suffix)}>{t.label}</button
				>
			{/each}
		</div>
	</div>

	<div class="card" aria-live="polite">
		<div class="top"><LearnToggle bind:on={learn} /></div>
		<div class="name" data-testid="lookup-name">{found.symbol}</div>
		{#if played.has(chordKeyOf(root, suffix))}
			<p class="played" data-testid="lookup-played">✓ You've played this</p>
		{/if}
		{#if analysis && learn}
			<p class="about">
				<span data-testid="lookup-description">{analysis.description}</span>
				{#if analysis.alternatives.length}
					<span class="also" data-testid="lookup-alternatives"
						>also {analysis.alternatives.join(' · ')}</span
					>
				{/if}
			</p>
		{/if}

		{#if shape}
			<div class="shapes">
				<ChordDiagram
					names={NAMES}
					frets={shape.frets}
					tones={learn ? analysis?.tones : []}
					testid="lookup-diagram"
				/>
				<pre class="tab" data-testid="lookup-tab">{tabLines(NAMES, shape.frets).join('\n')}</pre>
			</div>

			{#if onplay}
				<button type="button" class="play" onclick={() => onplay(root, suffix)}
					>Play it · I'll listen</button
				>
			{/if}
			{#if learn}
				<p class="notes" data-testid="lookup-notes">
					{#each notes as n (n.name)}
						<span><strong>{n.name}</strong> <span class="role">{n.role}</span></span>
					{/each}
					{#if omitted}<span class="omit">5th left out</span>{/if}
				</p>

				{#if analysis?.keys.length}
					<p class="keys" data-testid="lookup-keys">
						In {analysis.keys
							.slice(0, 3)
							.map((k) => `${k.key} (${k.numeral})`)
							.join(' · ')}
					</p>
				{:else}
					<p class="keys" data-testid="lookup-keys">Outside any single major key</p>
				{/if}
			{/if}
		{:else}
			<p class="keys">No playable ukulele shape found</p>
		{/if}
	</div>

	{#if found.shapes.length > 1}
		<div class="others">
			<h2>Shapes on the neck</h2>
			<ul data-testid="lookup-shapes">
				{#each found.shapes as s, i (s.frets.join('.'))}
					<li>
						<button
							type="button"
							class:selected={i === picked}
							aria-pressed={i === picked}
							aria-label="{found.symbol} {s.frets.join(' ')}"
							onclick={() => (picked = i)}
						>
							<ChordDiagram names={NAMES} frets={s.frets} small label="" />
							<span class="frets">{fretText(s.frets)}</span>
						</button>
					</li>
				{/each}
			</ul>
		</div>
	{/if}
</section>

<style>
	.lookup {
		display: grid;
		gap: 12px;
	}
	.pickers {
		display: grid;
		gap: 8px;
	}
	.roots {
		display: grid;
		grid-template-columns: repeat(6, 1fr);
		gap: 6px;
	}
	.types {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}
	.pickers button {
		min-height: 40px;
		border: 1px solid var(--line);
		border-radius: 10px;
		background: var(--surface);
		color: var(--text);
		font: inherit;
		font-weight: 600;
		cursor: pointer;
	}
	.types button {
		padding: 0 12px;
		font-weight: 500;
		font-size: 0.95rem;
	}
	.pickers button.selected {
		background: var(--accent);
		border-color: var(--accent);
		color: #fff;
	}
	.card {
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
		justify-content: flex-end;
		margin-bottom: -12px;
	}
	.name {
		font-size: clamp(3rem, 15vw, 4.5rem);
		font-weight: 700;
		line-height: 1;
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
	.shapes {
		display: flex;
		flex-wrap: wrap;
		justify-content: center;
		align-items: center;
		gap: 16px 28px;
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
	.notes {
		margin: 0;
		display: flex;
		flex-wrap: wrap;
		justify-content: center;
		gap: 6px 14px;
	}
	.notes .role {
		color: var(--accent);
		font-size: 0.85em;
	}
	.omit {
		color: var(--muted);
		font-size: 0.85rem;
	}
	.keys {
		margin: 0;
		color: var(--muted);
		font-size: 0.9rem;
	}
	.others h2 {
		margin: 0 0 8px;
		font-size: 0.95rem;
		font-weight: 600;
		color: var(--muted);
	}
	.others ul {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(96px, 1fr));
		gap: 8px;
	}
	.others button {
		width: 100%;
		display: grid;
		justify-items: center;
		gap: 2px;
		padding: 6px 4px;
		border: 1px solid var(--line);
		border-radius: 10px;
		background: var(--surface);
		color: var(--text);
		cursor: pointer;
	}
	.others button.selected {
		border-color: var(--accent);
		box-shadow: inset 0 0 0 1px var(--accent);
	}
	.frets {
		font:
			0.85rem ui-monospace,
			monospace;
		color: var(--muted);
	}
	.played {
		margin: -6px 0 0;
		color: var(--in-tune);
		font-size: 0.9rem;
	}
	.play {
		justify-self: center;
		min-height: 44px;
		padding: 0 22px;
		border: 0;
		border-radius: 999px;
		background: var(--accent);
		color: #fff;
		font: inherit;
		font-weight: 600;
		cursor: pointer;
	}
</style>
