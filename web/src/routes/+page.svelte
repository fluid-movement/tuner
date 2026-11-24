<script lang="ts">
	import { onDestroy, onMount, untrack } from 'svelte';
	import {
		describeStartError,
		loadTunerBytes,
		startTuner,
		type TunerEngine
	} from '#lib/audio/tuner-engine.ts';
	import {
		Tunings,
		instantiateTuner,
		type Instrument,
		type StrumReport,
		type TrackReading
	} from '#lib/wasm/tuner.ts';
	import type { StrumConfig } from '#lib/audio/messages.ts';
	import Sheet from '#lib/components/Sheet.svelte';
	import Icon from '#lib/components/Icon.svelte';
	import StrumView from '#lib/components/StrumView.svelte';
	import ChordView, { type HistoryEntry } from '#lib/components/ChordView.svelte';
	import LevelMeter from '#lib/components/LevelMeter.svelte';
	import ChordLookup from '#lib/components/ChordLookup.svelte';
	import { CHORD_TYPES } from '#lib/theory/lookup.ts';
	import { keepScreenOn } from '#lib/wake-lock.ts';

	/** What the app is doing: listening for chords, tuning, or looking up. */
	type Mode = 'chord' | 'strum' | 'lookup';
	/** The tools that slide up over the listening screen. */
	type SheetName = 'tune' | 'chords';
	/** Chords kept in the history (structured, for key detection later). */
	const HISTORY = 8;
	const LEARN_KEY = 'tuner.learn';
	const LOOKUP_KEY = 'tuner.lookup';
	/** Four Strings is a ukulele app; the core also knows the guitar. */
	const instrument: Instrument = 'ukulele';

	let sheet = $state<SheetName | null>(null);
	let mode = $derived<Mode>(sheet === 'chords' ? 'lookup' : sheet === 'tune' ? 'strum' : 'chord');
	/** Explanations on (Learn) or a clean view (Simple, the default). */
	let learn = $state(false);
	/** The chord shown in the lookup. */
	let lookup = $state({ root: 0, suffix: '' });
	/** Last strum report (kept until the next strum) and when its strum began. */
	let strum = $state<{ report: StrumReport; at: number } | null>(null);
	/**
	 * The chord on display: the latest report that named one. It stays until
	 * the next strum names a chord, so a pending or unrecognised strum doesn't
	 * blank it.
	 */
	let shownChord = $state<StrumReport | null>(null);
	/** Settled chords, oldest first; repeats of the same chord collapse. */
	let chords = $state<HistoryEntry[]>([]);
	/** Whether audio is flowing; false while the browser waits for a gesture. */
	let running = $state(false);
	let now = $state(0);
	let tunings = $state<Tunings | null>(null);
	let engine = $state<TunerEngine | null>(null);
	let starting = $state(false);
	let error = $state<string | null>(null);
	/** Latest level reading from the audio thread (for the meter), or null. */
	let reading = $state<TrackReading | null>(null);

	let strings = $derived(tunings?.strings(instrument) ?? []);

	onMount(async () => {
		// A reload starts on the listening screen, not in a sheet.
		if (history.state?.sheet) history.replaceState(null, '');
		try {
			learn = localStorage.getItem(LEARN_KEY) === '1';
			const chord = JSON.parse(localStorage.getItem(LOOKUP_KEY) ?? 'null');
			if (
				Number.isInteger(chord?.root) &&
				chord.root >= 0 &&
				chord.root < 12 &&
				CHORD_TYPES.some((t) => t.suffix === chord.suffix)
			)
				lookup = { root: chord.root, suffix: chord.suffix };
		} catch {
			// Storage may be unavailable (private mode); the default is fine.
		}
		try {
			tunings = new Tunings(await instantiateTuner(await loadTunerBytes()));
		} catch (err) {
			error = describeStartError(err);
			return;
		}
		await sync();
	});

	$effect(() => {
		try {
			localStorage.setItem(LEARN_KEY, learn ? '1' : '0');
			localStorage.setItem(LOOKUP_KEY, JSON.stringify(lookup));
		} catch {
			// Not remembering the choice is acceptable.
		}
	});

	// Strum analysis runs in the strum tuner and while listening for chords;
	// results for the other analysis don't apply.
	let strumConfig = $derived<StrumConfig | null>(
		mode === 'strum'
			? { instrument, analysis: 'tuning' }
			: mode === 'chord'
				? { instrument: 'ukulele', analysis: 'chord' }
				: null
	);
	// A different analysis: the last report doesn't apply. The chord on
	// display stays (tuning reports never name one), so it is still there
	// after tuning. (Not on engine restarts, which keep what was shown.)
	$effect(() => {
		void strumConfig;
		strum = null;
	});
	$effect(() => {
		const config = strumConfig;
		engine?.setStrum(config).catch((err) => (error = describeStartError(err)));
	});

	/** Opens a sheet; the back button or gesture closes it again. */
	function openSheet(next: SheetName) {
		if (sheet === next) return;
		if (sheet) history.replaceState({ sheet: next }, '');
		else history.pushState({ sheet: next }, '');
		sheet = next;
	}
	function closeSheet() {
		if (history.state?.sheet) history.back();
		else sheet = null;
	}

	/** The last settled report that named a chord. */
	let settledChord: StrumReport | null = null;
	/** Onset of the last chord added to the history. */
	let recordedOnset: number | null = null;

	function onStrum(report: StrumReport) {
		// Early and live reports refine the same strum: keep its start time.
		const same = strum?.report.onsetSeconds === report.onsetSeconds;
		strum = { report, at: same && strum ? strum.at : performance.now() };
		// A tentative chord that settles to none falls back to the last
		// settled chord.
		if (report.chord) shownChord = report;
		else if (shownChord && !shownChord.settled && shownChord.onsetSeconds === report.onsetSeconds)
			shownChord = settledChord;
		if (report.chord && report.settled) settledChord = report;
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
		// Focus may have gone while starting (blur is ignored meanwhile, as the
		// permission prompt can take focus).
		await sync();
	}

	/**
	 * Whether to listen: the app is on screen and focused, and the mode uses
	 * the microphone (chord lookup doesn't).
	 */
	const focused = () =>
		mode !== 'lookup' && document.visibilityState === 'visible' && document.hasFocus();
	// Entering or leaving chord lookup starts or stops listening.
	$effect(() => {
		void mode;
		if (tunings) untrack(() => void sync());
	});
	/** Stopped (unfocused, or looking up chords); starts again when that ends. */
	let paused = $state(false);

	/**
	 * Uses the microphone only while focused: stops the engine (releasing the
	 * microphone) when the app is hidden or loses focus, and starts it again
	 * when focus returns. Results on screen are kept.
	 */
	async function sync() {
		if (starting) return;
		if (focused()) {
			if (paused || (!engine && !error)) {
				paused = false;
				await start();
			}
		} else if (engine) {
			paused = true;
			const stopping = engine;
			engine = null;
			running = false;
			reading = null;
			recordedOnset = null; // onsets restart with the next engine
			await stopping.stop();
		} else if (!error) {
			paused = true; // opened in the background: start on focus
		}
	}

	/** Browsers that hold audio back until a gesture: the first tap starts it. */
	function wake() {
		engine?.resume().catch((err) => (error = describeStartError(err)));
	}

	onDestroy(() => engine?.stop());
</script>

<svelte:head>
	<title>Four Strings</title>
	<meta
		name="description"
		content="Strum your ukulele: see the chord, what comes next, and tune up"
	/>
</svelte:head>

<div class="app">
	<header>
		<h1 class="brand">Four Strings</h1>
		<nav aria-label="Tools">
			<button type="button" class="tool" aria-haspopup="dialog" onclick={() => openSheet('tune')}>
				<Icon name="tune" />
				<span>Tune</span>
			</button>
			<button type="button" class="tool" aria-haspopup="dialog" onclick={() => openSheet('chords')}>
				<Icon name="chords" />
				<span>Chords</span>
			</button>
		</nav>
	</header>

	<main>
		<ChordView
			{strings}
			report={mode === 'chord' ? (strum?.report ?? null) : null}
			shown={shownChord}
			history={chords}
			listening={running}
			bind:learn
			onLookup={(root, suffix) => {
				lookup = { root, suffix };
				openSheet('chords');
			}}
			onTune={() => openSheet('tune')}
		/>

		{#if running && !sheet}
			<LevelMeter levelDb={reading?.levelDb ?? null} floorDb={reading?.floorDb ?? null} />
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

{#if sheet === 'tune'}
	<Sheet title="Tune" onclose={closeSheet} testid="tune-sheet">
		<StrumView
			{strings}
			report={strum?.report ?? null}
			age={strum ? Math.max(0, (now - strum.at) / 1000) : null}
			listening={running}
		/>
		{#if running}
			<LevelMeter levelDb={reading?.levelDb ?? null} floorDb={reading?.floorDb ?? null} />
		{/if}
	</Sheet>
{:else if sheet === 'chords'}
	<Sheet title="Chords" onclose={closeSheet} testid="chords-sheet">
		<ChordLookup bind:root={lookup.root} bind:suffix={lookup.suffix} bind:learn />
	</Sheet>
{/if}

<svelte:window
	onpointerdown={() => engine && !running && wake()}
	onfocus={() => void sync()}
	onblur={() => void sync()}
	onpopstate={() => (sheet = history.state?.sheet ?? null)}
/>
<svelte:document onvisibilitychange={() => void sync()} />
{#if paused && mode !== 'lookup'}
	<div class="wake" data-testid="paused" role="status">
		Microphone off while the app isn't focused
	</div>
{:else if engine && !running}
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
	header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
	}
	.brand {
		margin: 0;
		white-space: nowrap;
		font-size: 1.2rem;
		font-weight: 700;
		letter-spacing: -0.02em;
	}
	nav {
		display: flex;
		gap: 6px;
	}
	.tool {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		min-height: 40px;
		padding: 0 14px 0 10px;
		border: 1px solid var(--line);
		border-radius: 999px;
		background: var(--surface);
		color: var(--text);
		font-weight: 550;
		font-size: 0.95rem;
		cursor: pointer;
	}
	/* Narrow phones: the name and both tools on one row. */
	@media (max-width: 360px) {
		.brand {
			font-size: 1.05rem;
		}
		.tool {
			gap: 4px;
			padding: 0 10px 0 8px;
			font-size: 0.9rem;
		}
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
		z-index: 20;
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
