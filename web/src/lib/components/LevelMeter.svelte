<script lang="ts">
	interface Props {
		/** Input level in dBFS, or null before audio starts. */
		levelDb: number | null;
		/** Estimated noise floor in dBFS. */
		floorDb: number | null;
	}
	let { levelDb, floorDb }: Props = $props();

	/** dB above the floor shown at full scale. */
	const RANGE = 40;
	/** Where a pluck becomes loud enough to lock (tuner-core onset margin). */
	const ONSET = 12;

	let above = $derived(
		levelDb === null || floorDb === null ? 0 : Math.max(0, Math.min(RANGE, levelDb - floorDb))
	);
</script>

<div
	class="meter"
	role="meter"
	aria-label="Input level above background noise"
	aria-valuemin={0}
	aria-valuemax={RANGE}
	aria-valuenow={Math.round(above)}
	data-testid="level"
>
	<div class="fill" class:strong={above >= ONSET} style:width="{(above / RANGE) * 100}%"></div>
	<div class="mark" style:left="{(ONSET / RANGE) * 100}%"></div>
</div>

<style>
	.meter {
		position: relative;
		height: 6px;
		border-radius: 3px;
		background: var(--line);
		overflow: hidden;
	}
	.fill {
		height: 100%;
		background: var(--muted);
		transition: width 60ms linear;
	}
	.fill.strong {
		background: var(--accent);
	}
	.mark {
		position: absolute;
		top: 0;
		bottom: 0;
		width: 2px;
		background: var(--surface);
	}
	@media (prefers-reduced-motion: reduce) {
		.fill {
			transition: none;
		}
	}
</style>
