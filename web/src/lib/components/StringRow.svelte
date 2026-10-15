<script lang="ts">
	import type { TunerString } from '#lib/wasm/tuner.ts';

	interface Props {
		strings: TunerString[];
		/** Index of the string being tuned, or null. */
		active: number | null;
		inTune: boolean;
	}
	let { strings, active, inTune }: Props = $props();
</script>

<ol class="strings" aria-label="Strings">
	{#each strings as s (s.index)}
		<li
			data-testid="string-{s.name}"
			class:active={s.index === active}
			class:in-tune={s.index === active && inTune}
			aria-current={s.index === active ? 'true' : undefined}
		>
			{s.name.replace(/-?\d+$/, '')}<sub>{s.name.match(/-?\d+$/)?.[0]}</sub>
		</li>
	{/each}
</ol>

<style>
	.strings {
		list-style: none;
		display: flex;
		justify-content: center;
		gap: 8px;
		margin: 0;
		padding: 0;
		flex-wrap: wrap;
	}
	li {
		min-width: 44px;
		height: 44px;
		padding: 0 8px;
		display: grid;
		place-items: center;
		grid-auto-flow: column;
		border-radius: 999px;
		border: 1.5px solid var(--line);
		color: var(--muted);
		font-weight: 600;
		transition:
			background 120ms,
			color 120ms,
			border-color 120ms;
	}
	sub {
		font-size: 0.65em;
		font-weight: 500;
	}
	li.active {
		border-color: var(--off);
		color: var(--text);
	}
	li.in-tune {
		border-color: var(--in-tune);
		background: var(--in-tune);
		color: #fff;
	}
</style>
