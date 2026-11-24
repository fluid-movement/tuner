<script lang="ts">
	import { DRILL_SECONDS, pairName, remaining } from '#lib/practice/drill.ts';
	import { aim, type Practice } from '#lib/practice/session.ts';
	import type { Target } from '#lib/practice/target.ts';
	import ChordDiagram from './ChordDiagram.svelte';
	import Icon from './Icon.svelte';

	interface Props {
		practice: Practice;
		/** ms since the epoch, ticking while a drill runs. */
		now: number;
		/** Learn mode: why the landed move works, e.g. "V → I · …". */
		why: string | null;
		onclear: () => void;
		/** Shows the target's other shapes in the lookup. */
		onshapes: (target: Target) => void;
		/** Runs the same drill again. */
		onagain: () => void;
	}
	let { practice, now, why, onclear, onshapes, onagain }: Props = $props();

	const NAMES = ['G', 'C', 'E', 'A'];
	const SOURCES: Record<Target['source'], string> = {
		idea: 'Next idea',
		history: 'Again',
		lookup: 'From the lookup',
		daily: 'Chord of the day',
		drill: 'Changes',
		progression: 'Play along'
	};
	let target = $derived(aim(practice));
	let landed = $derived(practice.kind === 'target' && practice.landedAt !== null);
	const clock = (s: number) => {
		const t = Math.ceil(s);
		return `${Math.floor(t / 60)}:${String(t % 60).padStart(2, '0')}`;
	};
</script>

<section
	class="strip"
	class:landed
	aria-live="polite"
	data-testid="target"
	data-kind={practice.kind}
	data-landed={landed}
>
	{#if target}
		<div class="shape">
			<ChordDiagram names={NAMES} frets={target.frets} tiny label="{target.symbol} shape" />
		</div>
	{/if}

	<div class="text">
		{#if practice.kind === 'target'}
			<p class="label">{SOURCES[practice.target.source]}</p>
			<p class="main" data-testid="target-text">
				{#if landed}
					{practice.target.symbol} <span class="tick">✓</span>
				{:else}
					Play <strong>{practice.target.symbol}</strong>
				{/if}
			</p>
			{#if landed && why}<p class="why" data-testid="target-why">{why}</p>{/if}
		{:else if practice.kind === 'drill'}
			<p class="label">Changes · {pairName(practice.drill.a, practice.drill.b)}</p>
			{#if practice.result}
				<p class="main" data-testid="target-text">
					{practice.result.count} changes{#if practice.result.best}<span class="tick">
							· best</span
						>{/if}
				</p>
				{#if !practice.result.best && practice.result.previousBest}
					<p class="why">Best {practice.result.previousBest}</p>
				{/if}
			{:else}
				<p class="main" data-testid="target-text">
					{#if practice.drill.startedAt === null}
						Play <strong>{practice.drill.a.symbol}</strong> to start
					{:else}
						<strong>{target?.symbol}</strong>
						<span class="count" data-testid="drill-count">{practice.drill.count}</span>
					{/if}
				</p>
				<p class="why">
					{practice.drill.startedAt === null
						? `${DRILL_SECONDS} s · switch as often as you can`
						: `${clock(remaining(practice.drill, now))} left`}
				</p>
			{/if}
		{:else}
			<p class="label">
				{practice.name} in {practice.keyName}{#if practice.rounds}&nbsp;· round {practice.rounds +
						1}{/if}
			</p>
			<ol class="steps" data-testid="progression">
				{#each practice.chords as c, i (i)}
					<li class:current={i === practice.index}>{c.symbol}</li>
				{/each}
			</ol>
		{/if}
	</div>

	<div class="actions">
		{#if practice.kind === 'drill' && practice.result}
			<button type="button" class="text-button" onclick={onagain}>Again</button>
		{:else if practice.kind === 'target'}
			<button type="button" class="text-button" onclick={() => onshapes(practice.target)}
				>Shapes</button
			>
		{/if}
		<button type="button" class="clear" aria-label="Stop" onclick={onclear}>
			<Icon name="close" size={18} />
		</button>
	</div>
</section>

<style>
	.strip {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 10px 10px 10px 12px;
		border: 1px solid var(--line);
		border-radius: var(--radius);
		background: var(--surface);
		transition:
			border-color 200ms,
			background-color 200ms;
	}
	.strip.landed {
		border-color: color-mix(in srgb, var(--in-tune) 60%, var(--line));
		background: color-mix(in srgb, var(--in-tune) 8%, var(--surface));
	}
	.shape {
		flex: none;
	}
	.text {
		flex: 1;
		min-width: 0;
		display: grid;
		gap: 2px;
	}
	.label {
		margin: 0;
		color: var(--muted);
		font-size: 0.8rem;
	}
	.main {
		margin: 0;
		font-size: 1.1rem;
	}
	.tick {
		color: var(--in-tune);
		font-weight: 650;
	}
	.count {
		margin-left: 8px;
		font-variant-numeric: tabular-nums;
		color: var(--accent);
		font-weight: 650;
	}
	.why {
		margin: 0;
		color: var(--muted);
		font-size: 0.85rem;
	}
	.steps {
		list-style: none;
		margin: 2px 0 0;
		padding: 0;
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}
	.steps li {
		padding: 2px 10px;
		border-radius: 999px;
		background: var(--line);
		font-weight: 600;
	}
	.steps li.current {
		background: var(--accent);
		color: #fff;
	}
	.actions {
		display: flex;
		align-items: center;
		gap: 4px;
	}
	.text-button {
		min-height: 36px;
		padding: 0 10px;
		border: 0;
		border-radius: 999px;
		background: transparent;
		color: var(--accent);
		font-weight: 600;
		cursor: pointer;
	}
	.clear {
		display: grid;
		place-items: center;
		width: 36px;
		height: 36px;
		border: 0;
		border-radius: 999px;
		background: var(--line);
		color: var(--text);
		cursor: pointer;
	}
</style>
