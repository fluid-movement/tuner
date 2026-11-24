<script lang="ts" generics="T extends string">
	interface Props {
		/** Accessible name of the group, e.g. "Instrument". */
		label: string;
		options: readonly { value: T; label: string }[];
		value: T;
	}
	let { label, options, value = $bindable() }: Props = $props();
	let name = $derived(`segmented-${label.toLowerCase().replace(/\s+/g, '-')}`);
</script>

<div class="toggle" role="radiogroup" aria-label={label}>
	{#each options as option (option.value)}
		<label class:selected={value === option.value}>
			<input type="radio" {name} value={option.value} bind:group={value} />
			{option.label}
		</label>
	{/each}
</div>

<style>
	.toggle {
		display: inline-flex;
		padding: 3px;
		min-height: 44px;
		border-radius: 999px;
		background: var(--line);
	}
	label {
		/* At least 44 px tall: a comfortable touch target. */
		min-height: 38px;
		display: inline-flex;
		align-items: center;
		padding: 0 16px;
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
	/* Narrow phones: four modes and the title fit on one row. */
	@media (max-width: 400px) {
		label {
			padding: 0 9px;
			font-size: 0.9rem;
		}
	}
	@media (max-width: 340px) {
		label {
			padding: 0 6px;
			font-size: 0.85rem;
		}
	}
</style>
