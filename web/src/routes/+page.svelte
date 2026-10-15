<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import {
		describeStartError,
		loadTunerBytes,
		startTuner,
		type TunerEngine
	} from '#lib/audio/tuner-engine.ts';
	import { INSTRUMENTS, Tunings, instantiateTuner, type Instrument } from '#lib/wasm/tuner.ts';
	import { InTuneLatch, PitchSmoother } from '#lib/tuner/smoother.ts';
	import CentsMeter from '#lib/components/CentsMeter.svelte';
	import StringRow from '#lib/components/StringRow.svelte';
	import InstrumentToggle from '#lib/components/InstrumentToggle.svelte';

	const STORAGE_KEY = 'tuner.instrument';

	let instrument = $state<Instrument>('guitar');
	let tunings = $state<Tunings | null>(null);
	let engine = $state<TunerEngine | null>(null);
	let starting = $state(false);
	let error = $state<string | null>(null);
	/** Smoothed frequency, or null when nothing is heard. */
	let frequency = $state<number | null>(null);

	const smoother = new PitchSmoother();
	const latch = new InTuneLatch();

	let strings = $derived(tunings?.strings(instrument) ?? []);
	let target = $derived(frequency && tunings ? tunings.nearest(instrument, frequency) : null);
	let note = $derived(frequency && tunings ? tunings.note(frequency) : null);
	let inTune = $derived(latch.update(target?.cents ?? null));

	onMount(async () => {
		try {
			const saved = localStorage.getItem(STORAGE_KEY);
			if (INSTRUMENTS.includes(saved as Instrument)) instrument = saved as Instrument;
		} catch {
			// Storage may be unavailable (private mode); the default is fine.
		}
		try {
			tunings = new Tunings(await instantiateTuner(await loadTunerBytes()));
		} catch (err) {
			error = describeStartError(err);
		}
	});

	$effect(() => {
		try {
			localStorage.setItem(STORAGE_KEY, instrument);
		} catch {
			// Not remembering the choice is acceptable.
		}
	});

	async function start() {
		starting = true;
		error = null;
		try {
			engine = await startTuner({
				onReading: (r) => (frequency = smoother.push(r?.frequency ?? null, performance.now())),
				onError: (message) => (error = message)
			});
		} catch (err) {
			error = describeStartError(err);
		} finally {
			starting = false;
		}
	}

	async function stop() {
		await engine?.stop();
		engine = null;
		smoother.reset();
		frequency = null;
	}

	onDestroy(() => engine?.stop());

	const fmtCents = (c: number) => `${c >= 0 ? '+' : '−'}${Math.abs(c).toFixed(1)}`;
	let status = $derived(
		!engine
			? 'Press Start and play a string'
			: !target
				? 'Play a string'
				: inTune
					? 'In tune'
					: target.cents < 0
						? 'Flat · tune up'
						: 'Sharp · tune down'
	);
</script>

<svelte:head>
	<title>Tuner</title>
	<meta name="description" content="Tuner for acoustic guitar and ukulele" />
</svelte:head>

<div class="app">
	<header>
		<h1>Tuner</h1>
		<InstrumentToggle bind:value={instrument} />
	</header>

	<main>
		<section class="readout" aria-live="polite">
			<div class="note" class:in-tune={inTune}>
				{#if note}
					<span data-testid="note"
						>{note.name.replace(/-?\d+$/, '')}<sub>{note.name.match(/-?\d+$/)?.[0]}</sub></span
					>
				{:else}
					<span class="placeholder">–</span>
				{/if}
			</div>
			<p class="status" data-testid="status" class:in-tune={inTune}>{status}</p>
			<CentsMeter cents={target?.cents ?? null} {inTune} />
			<p class="numbers">
				{#if target && frequency}
					<span data-testid="cents">{fmtCents(target.cents)} cents</span>
					<span class="sep">·</span>
					<span data-testid="frequency">{frequency.toFixed(2)} Hz</span>
				{:else}
					&nbsp;
				{/if}
			</p>
		</section>

		<StringRow {strings} active={target?.string.index ?? null} {inTune} />
	</main>

	<footer>
		{#if engine}
			<button class="primary" onclick={stop}>Stop</button>
		{:else}
			<button class="primary" onclick={start} disabled={starting || !tunings}>
				{starting ? 'Starting…' : 'Start'}
			</button>
		{/if}
		{#if error}
			<p role="alert">{error}</p>
		{/if}
	</footer>
</div>

<style>
	.app {
		min-height: 100dvh;
		max-width: 560px;
		margin: 0 auto;
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 20px;
	}
	header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
	}
	h1 {
		font-size: 1.25rem;
		margin: 0;
	}
	main {
		flex: 1;
		display: flex;
		flex-direction: column;
		justify-content: center;
		gap: 24px;
	}
	.readout {
		background: var(--surface);
		border: 1px solid var(--line);
		border-radius: var(--radius);
		padding: 20px 16px 12px;
		text-align: center;
	}
	.note {
		font-size: clamp(4rem, 22vw, 6.5rem);
		font-weight: 700;
		line-height: 1;
		font-variant-numeric: tabular-nums;
		transition: color 120ms;
	}
	.note sub {
		font-size: 0.35em;
		font-weight: 500;
		color: var(--muted);
	}
	.note.in-tune {
		color: var(--in-tune);
	}
	.placeholder {
		color: var(--line);
	}
	.status {
		margin: 8px 0 12px;
		color: var(--muted);
		font-weight: 500;
	}
	.status.in-tune {
		color: var(--in-tune);
	}
	.numbers {
		margin: 8px 0 0;
		color: var(--muted);
		font-variant-numeric: tabular-nums;
		font-size: 0.9rem;
	}
	.sep {
		margin: 0 6px;
	}
	footer {
		text-align: center;
	}
	.primary {
		width: 100%;
		max-width: 320px;
		padding: 14px;
		border: 0;
		border-radius: 999px;
		background: var(--accent);
		color: #fff;
		font-weight: 600;
		font-size: 1.05rem;
		cursor: pointer;
	}
	.primary:disabled {
		opacity: 0.6;
		cursor: default;
	}
	[role='alert'] {
		color: var(--off);
		margin: 12px 0 0;
	}
</style>
