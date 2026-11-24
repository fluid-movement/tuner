<script lang="ts">
	interface Props {
		/** Whether explanations are shown. */
		on: boolean;
	}
	let { on = $bindable() }: Props = $props();
</script>

<button
	type="button"
	class="learn"
	class:on
	aria-pressed={on}
	data-testid="learn-toggle"
	title={on ? 'Hide the theory' : 'Explain the chords'}
	onclick={() => (on = !on)}>Learn</button
>

<style>
	.learn {
		position: absolute;
		top: 10px;
		right: 10px;
		min-height: 32px;
		padding: 0 12px;
		border: 1px solid var(--line);
		border-radius: 999px;
		background: transparent;
		color: var(--muted);
		font-size: 0.85rem;
		font-weight: 600;
		cursor: pointer;
	}
	.learn.on {
		background: var(--accent);
		border-color: var(--accent);
		color: #fff;
	}
</style>
