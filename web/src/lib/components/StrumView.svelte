<script lang="ts">
	import type { StrumReport, TunerString } from '#lib/wasm/tuner.ts';

	interface Props {
		strings: TunerString[];
		/** Last strum's report, or null before the first strum. */
		report: StrumReport | null;
		/** Seconds since the last strum, or null. */
		age: number | null;
		/** Whether the microphone is on. */
		listening: boolean;
	}
	let { strings, report, age, listening }: Props = $props();

	const IN_TUNE_CENTS = 5;
	type Verdict = 'in-tune' | 'flat' | 'sharp' | 'not-heard' | 'none';
	const LABELS: Record<Verdict, string> = {
		'in-tune': 'In tune',
		flat: 'Flat',
		sharp: 'Sharp',
		'not-heard': 'Not heard',
		none: '–'
	};

	function verdict(i: number): Verdict {
		const s = report?.strings[i];
		if (!s) return 'none';
		if (s.present === false) return 'not-heard';
		if (s.cents === null) return 'none';
		if (Math.abs(s.cents) <= IN_TUNE_CENTS) return 'in-tune';
		return s.cents < 0 ? 'flat' : 'sharp';
	}
	const fmt = (c: number) => {
		const r = Math.round(c);
		return `${r > 0 ? '+' : r < 0 ? '−' : '±'}${Math.abs(r)} c`;
	};
	/** Bar position for ±50 cents. */
	const offset = (c: number) => Math.max(-50, Math.min(50, c));
</script>

<section class="strum" aria-live="polite">
	<p class="prompt" data-testid="strum-status">
		{#if report && age !== null}
			Last strum {age < 1 ? 'just now' : `${Math.round(age)} s ago`} · strum again to re-check
		{:else if listening}
			Strum all strings
		{:else}
			Press Start and strum all strings
		{/if}
	</p>
	<ol>
		{#each strings as s, i (s.index)}
			{@const v = verdict(i)}
			{@const cents = report?.strings[i]?.cents ?? null}
			<li data-testid="strum-{s.name}" data-verdict={v} class={v}>
				<span class="name"
					>{s.name.replace(/-?\d+$/, '')}<sub>{s.name.match(/-?\d+$/)?.[0]}</sub></span
				>
				<span class="bar" aria-hidden="true">
					<span class="zone"></span>
					{#if cents !== null && v !== 'not-heard'}
						<span class="dot" style:left="{50 + offset(cents)}%"></span>
					{/if}
				</span>
				<span class="verdict">{LABELS[v]}</span>
				<span class="cents">{cents !== null && v !== 'not-heard' ? fmt(cents) : ''}</span>
			</li>
		{/each}
	</ol>
</section>

<style>
	.strum {
		background: var(--surface);
		border: 1px solid var(--line);
		border-radius: var(--radius);
		padding: 16px;
	}
	.prompt {
		margin: 0 0 12px;
		color: var(--muted);
		text-align: center;
		font-weight: 500;
	}
	ol {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		gap: 8px;
	}
	li {
		display: grid;
		grid-template-columns: 2.6rem 1fr 5.2rem 3.4rem;
		align-items: center;
		gap: 10px;
		min-height: 44px;
	}
	.name {
		font-weight: 700;
		font-size: 1.15rem;
	}
	sub {
		font-size: 0.65em;
		font-weight: 500;
		color: var(--muted);
	}
	.bar {
		position: relative;
		height: 8px;
		border-radius: 4px;
		background: var(--line);
	}
	.zone {
		position: absolute;
		left: 45%;
		width: 10%;
		top: 0;
		bottom: 0;
		background: var(--in-tune);
		opacity: 0.35;
	}
	.dot {
		position: absolute;
		top: 50%;
		width: 14px;
		height: 14px;
		border-radius: 50%;
		background: var(--off);
		transform: translate(-50%, -50%);
	}
	.verdict {
		font-weight: 600;
		color: var(--muted);
	}
	.cents {
		color: var(--muted);
		font-variant-numeric: tabular-nums;
		text-align: right;
		font-size: 0.9rem;
	}
	.in-tune .verdict {
		color: var(--in-tune);
	}
	.in-tune .dot {
		background: var(--in-tune);
	}
	.flat .verdict,
	.sharp .verdict {
		color: var(--off);
	}
	.not-heard {
		opacity: 0.5;
	}
</style>
