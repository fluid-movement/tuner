<script lang="ts">
	interface Props {
		/** Offset from the target in cents, or null when nothing is heard. */
		cents: number | null;
		inTune: boolean;
	}
	let { cents, inTune }: Props = $props();

	const RANGE = 50;
	const SWEEP = 60; // degrees either side of centre
	const CX = 100;
	const CY = 110;
	const R = 90;

	const angle = (c: number) => (Math.max(-RANGE, Math.min(RANGE, c)) / RANGE) * SWEEP;
	function point(deg: number, r: number) {
		const rad = ((deg - 90) * Math.PI) / 180;
		return [CX + r * Math.cos(rad), CY + r * Math.sin(rad)];
	}
	function arc(fromDeg: number, toDeg: number, r: number) {
		const [x1, y1] = point(fromDeg, r);
		const [x2, y2] = point(toDeg, r);
		return `M ${x1} ${y1} A ${r} ${r} 0 0 1 ${x2} ${y2}`;
	}

	const ticks = Array.from({ length: 11 }, (_, i) => -RANGE + i * 10);
	let needle = $derived(cents === null ? 0 : angle(cents));
	let pegged = $derived(cents !== null && Math.abs(cents) > RANGE);
</script>

<svg viewBox="0 0 200 120" role="img" aria-label="Tuning meter" class:idle={cents === null}>
	<path d={arc(-SWEEP, SWEEP, R)} class="scale" />
	<path d={arc(angle(-5), angle(5), R)} class="zone" class:active={inTune} />
	{#each ticks as t (t)}
		{@const [x1, y1] = point(angle(t), R - (t === 0 ? 14 : 8))}
		{@const [x2, y2] = point(angle(t), R + 4)}
		<line {x1} {y1} {x2} {y2} class="tick" class:major={t === 0} />
	{/each}
	<text x={point(-SWEEP, R)[0] - 6} y={CY - 32} class="label">♭</text>
	<text x={point(SWEEP, R)[0] + 6} y={CY - 32} class="label">♯</text>
	<g class="needle" class:in-tune={inTune} class:pegged style:transform="rotate({needle}deg)">
		<line x1={CX} y1={CY} x2={CX} y2={CY - R + 6} />
		<circle cx={CX} cy={CY} r="5" />
	</g>
</svg>

<style>
	svg {
		width: 100%;
		max-width: 420px;
		display: block;
		margin: 0 auto;
		overflow: visible;
	}
	.scale {
		fill: none;
		stroke: var(--line);
		stroke-width: 3;
	}
	.zone {
		fill: none;
		stroke: var(--in-tune);
		stroke-width: 8;
		opacity: 0.35;
		stroke-linecap: round;
		transition: opacity 150ms;
	}
	.zone.active {
		opacity: 1;
	}
	.tick {
		stroke: var(--muted);
		stroke-width: 1.5;
	}
	.tick.major {
		stroke: var(--text);
		stroke-width: 2.5;
	}
	.label {
		fill: var(--muted);
		font-size: 14px;
		text-anchor: middle;
	}
	.needle {
		transform-origin: 100px 110px;
		transition: transform 90ms linear;
		stroke: var(--off);
		fill: var(--off);
		stroke-width: 3;
		stroke-linecap: round;
	}
	.needle.in-tune {
		stroke: var(--in-tune);
		fill: var(--in-tune);
	}
	.needle.pegged {
		opacity: 0.6;
	}
	.idle .needle {
		stroke: var(--line);
		fill: var(--line);
	}
	@media (prefers-reduced-motion: reduce) {
		.needle {
			transition: none;
		}
	}
</style>
