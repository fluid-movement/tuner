<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import {
		describeStartError,
		loadTunerBytes,
		startTuner,
		type TunerEngine
	} from '#lib/audio/tuner-engine.ts';
	import {
		INSTRUMENTS,
		Tunings,
		instantiateTuner,
		type Instrument,
		type StrumReport,
		type TrackReading
	} from '#lib/wasm/tuner.ts';
	import type { StrumConfig } from '#lib/audio/messages.ts';
	import { InTuneLatch } from '#lib/tuner/in-tune.ts';
	import CentsMeter from '#lib/components/CentsMeter.svelte';
	import StringRow from '#lib/components/StringRow.svelte';
	import Segmented from '#lib/components/Segmented.svelte';
	import StrumView from '#lib/components/StrumView.svelte';
	import ChordView, { type HistoryEntry } from '#lib/components/ChordView.svelte';
	import LevelMeter from '#lib/components/LevelMeter.svelte';
	import { keepScreenOn } from '#lib/wake-lock.ts';

	type Mode = 'chord' | 'strum' | 'string';
	const MODES: readonly Mode[] = ['chord', 'strum', 'string'];
	/** Chords kept in the history (structured, for key detection later). */
	const HISTORY = 8;
	const STORAGE_KEY = 'tuner.instrument';
	const MODE_KEY = 'tuner.mode';
	const INSTRUMENT_OPTIONS = [
		{ value: 'guitar', label: 'Guitar' },
		{ value: 'ukulele', label: 'Ukulele' }
	] as const;
	const MODE_OPTIONS = [
		{ value: 'chord', label: 'Chord' },
		{ value: 'strum', label: 'Strum' },
		{ value: 'string', label: 'String' }
	] as const;

	let instrument = $state<Instrument>('guitar');
	let mode = $state<Mode>('chord');
	/** Last strum report (kept until the next strum) and when its strum began. */
	let strum = $state<{ report: StrumReport; at: number } | null>(null);
	/** Settled chords, oldest first; repeats of the same chord collapse. */
	let chords = $state<HistoryEntry[]>([]);
	/** Whether audio is flowing; false while the browser waits for a gesture. */
	let running = $state(false);
	let now = $state(0);
	let tunings = $state<Tunings | null>(null);
	let engine = $state<TunerEngine | null>(null);
	let starting = $state(false);
	let error = $state<string | null>(null);
	/** Latest output of the note tracker (smoothed, held), or null. */
	let reading = $state<TrackReading | null>(null);
	let debug = $state(false);

	const latch = new InTuneLatch();

	let frequency = $derived(reading?.frequency ?? null);
	let holding = $derived(reading?.state === 'holding');
	let strings = $derived(tunings?.strings(instrument) ?? []);
	let target = $derived(frequency && tunings ? tunings.nearest(instrument, frequency) : null);
	let note = $derived(frequency && tunings ? tunings.note(frequency) : null);
	let inTune = $derived(latch.update(target?.cents ?? null));

	onMount(async () => {
		debug = new URLSearchParams(location.search).has('debug');
		try {
			const saved = localStorage.getItem(STORAGE_KEY);
			if (INSTRUMENTS.includes(saved as Instrument)) instrument = saved as Instrument;
			const savedMode = localStorage.getItem(MODE_KEY);
			if (MODES.includes(savedMode as Mode)) mode = savedMode as Mode;
		} catch {
			// Storage may be unavailable (private mode); the default is fine.
		}
		try {
			tunings = new Tunings(await instantiateTuner(await loadTunerBytes()));
		} catch (err) {
			error = describeStartError(err);
			return;
		}
		await start();
	});

	$effect(() => {
		try {
			localStorage.setItem(STORAGE_KEY, instrument);
			localStorage.setItem(MODE_KEY, mode);
		} catch {
			// Not remembering the choice is acceptable.
		}
	});

	// Strum analysis runs in strum mode (tuning, selected instrument) and
	// chord mode (ukulele only for now); results for another setting don't
	// apply.
	let strumConfig = $derived<StrumConfig | null>(
		mode === 'strum'
			? { instrument, analysis: 'tuning' }
			: mode === 'chord'
				? { instrument: 'ukulele', analysis: 'chord' }
				: null
	);
	// The single-note tracker only runs in string mode.
	$effect(() => {
		engine?.setPitch(mode === 'string');
	});
	$effect(() => {
		const config = strumConfig;
		strum = null;
		engine?.setStrum(config).catch((err) => (error = describeStartError(err)));
	});
	let ukulele = $derived(tunings?.strings('ukulele') ?? []);

	/** Onset of the last chord added to the history. */
	let recordedOnset: number | null = null;

	function onStrum(report: StrumReport) {
		// Early and live reports refine the same strum: keep its start time.
		const same = strum?.report.onsetSeconds === report.onsetSeconds;
		strum = { report, at: same && strum ? strum.at : performance.now() };
		const analysis = report.chord?.analysis;
		if (analysis && report.settled && recordedOnset !== report.onsetSeconds) {
			recordedOnset = report.onsetSeconds;
			const last = chords.at(-1);
			chords =
				last?.analysis.symbol === analysis.symbol
					? [...chords.slice(0, -1), { analysis, count: last.count + 1 }]
					: [...chords, { analysis, count: 1 }].slice(-HISTORY);
		}
	}

	// Keep the screen on while listening.
	$effect(() => {
		if (running) return keepScreenOn();
	});

	// Ticks the "last strum" age.
	$effect(() => {
		if (!strum) return;
		now = performance.now();
		const timer = setInterval(() => (now = performance.now()), 1000);
		return () => clearInterval(timer);
	});

	async function start() {
		if (starting || engine) return;
		starting = true;
		error = null;
		try {
			engine = await startTuner({
				onReading: (r) => (reading = r),
				onStrum,
				onRunningChange: (r) => (running = r),
				onError: (message) => (error = message)
			});
			running = engine.running;
		} catch (err) {
			error = describeStartError(err);
		} finally {
			starting = false;
		}
	}

	/** Browsers that hold audio back until a gesture: the first tap starts it. */
	function wake() {
		engine?.resume().catch((err) => (error = describeStartError(err)));
	}

	onDestroy(() => engine?.stop());

	const fmtCents = (c: number) => `${c >= 0 ? '+' : '−'}${Math.abs(c).toFixed(1)}`;
	let status = $derived(
		!running
			? 'Starting microphone…'
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
		<Segmented label="Mode" options={MODE_OPTIONS} bind:value={mode} />
	</header>
	{#if mode !== 'chord'}
		<div class="instrument">
			<Segmented label="Instrument" options={INSTRUMENT_OPTIONS} bind:value={instrument} />
		</div>
	{/if}

	<main>
		{#if mode === 'chord'}
			<ChordView
				strings={ukulele}
				report={strum?.report ?? null}
				history={chords}
				listening={running}
				onTune={() => {
					instrument = 'ukulele';
					mode = 'strum';
				}}
			/>
		{:else if mode === 'strum'}
			<StrumView
				{strings}
				report={strum?.report ?? null}
				age={strum ? Math.max(0, (now - strum.at) / 1000) : null}
				listening={running}
			/>
		{:else}
			<section class="readout" class:holding aria-live="polite">
				<div class="note" class:in-tune={inTune}>
					{#if note}
						<span data-testid="note"
							>{note.name.replace(/-?\d+$/, '')}<sub>{note.name.match(/-?\d+$/)?.[0]}</sub></span
						>
					{:else}
						<span class="placeholder">–</span>
					{/if}
				</div>
				<p class="status" data-testid="status" class:in-tune={inTune}>
					{status}
					{#if holding}<span class="hold" data-testid="hold">Hold</span>{/if}
				</p>
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
		{/if}

		{#if running}
			<LevelMeter levelDb={reading?.levelDb ?? null} floorDb={reading?.floorDb ?? null} />
		{/if}

		{#if debug && reading}
			<dl class="debug" data-testid="debug">
				<dt>state</dt>
				<dd>{reading.state}{reading.locked ? ' (locked)' : ''}</dd>
				<dt>raw</dt>
				<dd>{reading.rawFrequency?.toFixed(2) ?? '–'} Hz</dd>
				<dt>clarity</dt>
				<dd>{reading.clarity.toFixed(2)}</dd>
				<dt>level</dt>
				<dd>{reading.levelDb.toFixed(1)} dBFS</dd>
				<dt>floor</dt>
				<dd>{reading.floorDb.toFixed(1)} dBFS</dd>
			</dl>
		{/if}
	</main>

	{#if error}
		<footer>
			<p role="alert">{error}</p>
			{#if !engine && tunings}
				<button class="primary" onclick={start} disabled={starting}>Try again</button>
			{/if}
		</footer>
	{/if}
</div>

<svelte:window onpointerdown={() => engine && !running && wake()} />
{#if engine && !running}
	<!-- Taps go through to the page; any tap (window listener) starts audio. -->
	<div class="wake" data-testid="wake" role="status">Tap anywhere to start listening</div>
{/if}

<style>
	.app {
		min-height: 100dvh;
		max-width: 560px;
		margin: 0 auto;
		/* Clear the notch and home indicator when installed. */
		padding: max(12px, env(safe-area-inset-top)) max(16px, env(safe-area-inset-right))
			max(16px, env(safe-area-inset-bottom)) max(16px, env(safe-area-inset-left));
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.instrument {
		display: flex;
		justify-content: center;
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
		/* Top-aligned: on a phone the reading sits where the eye lands,
		   without a band of empty space above it. */
		justify-content: flex-start;
		gap: 16px;
	}
	.readout {
		transition: opacity 200ms;
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
	/* The string has faded: keep the last reading, visibly older. */
	.readout.holding {
		opacity: 0.55;
	}
	.hold {
		margin-left: 6px;
		padding: 1px 6px;
		border: 1px solid currentColor;
		border-radius: 4px;
		font-size: 0.75rem;
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}
	.debug {
		display: grid;
		grid-template-columns: auto 1fr;
		gap: 2px 12px;
		margin: 0;
		font:
			0.8rem ui-monospace,
			monospace;
		color: var(--muted);
	}
	.debug dd {
		margin: 0;
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
		margin: 0 0 12px;
	}
	/* Some browsers start audio only after a gesture: one tap anywhere. */
	.wake {
		position: fixed;
		inset: 0;
		display: grid;
		place-items: center;
		padding: 16px;
		text-align: center;
		pointer-events: none;
		background: color-mix(in srgb, var(--bg) 82%, transparent);
		backdrop-filter: blur(3px);
		color: var(--text);
		font-size: 1.2rem;
		font-weight: 600;
	}
</style>
