<script lang="ts">
	import { INSTRUMENTS, type Instrument } from '#lib/wasm/tuner.ts';

	interface Props {
		value: Instrument;
	}
	let { value = $bindable() }: Props = $props();

	const LABELS: Record<Instrument, string> = { guitar: 'Guitar', ukulele: 'Ukulele' };
</script>

<div class="toggle" role="radiogroup" aria-label="Instrument">
	{#each INSTRUMENTS as instrument (instrument)}
		<label class:selected={value === instrument}>
			<input type="radio" name="instrument" value={instrument} bind:group={value} />
			{LABELS[instrument]}
		</label>
	{/each}
</div>

<style>
	.toggle {
		display: inline-flex;
		padding: 3px;
		border-radius: 999px;
		background: var(--line);
	}
	label {
		padding: 6px 14px;
		border-radius: 999px;
		cursor: pointer;
		color: var(--muted);
		font-weight: 500;
		font-size: 0.95rem;
	}
	label.selected {
		background: var(--surface);
		color: var(--text);
		box-shadow: 0 1px 2px rgb(0 0 0 / 0.15);
	}
	label:has(input:focus-visible) {
		outline: 2px solid var(--accent);
		outline-offset: 2px;
	}
	input {
		position: absolute;
		opacity: 0;
		pointer-events: none;
	}
</style>
