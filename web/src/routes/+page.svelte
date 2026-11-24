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
	import ChordDiagram from '#lib/components/ChordDiagram.svelte';
	import TargetStrip from '#lib/components/TargetStrip.svelte';
	import ProgressStrip from '#lib/components/ProgressStrip.svelte';
	import PracticePanel from '#lib/components/PracticePanel.svelte';
	import { CHORD_TYPES } from '#lib/theory/lookup.ts';
	import { detectKey, nextReason, numeralIn } from '#lib/theory/key.ts';
	import { allInTune } from '#lib/tuner/intonation.ts';
	import { makeTarget, type Target, type TargetSource } from '#lib/practice/target.ts';
	import { chordKey, chordKeyOf, chordOfTheDay } from '#lib/practice/daily.ts';
	import { pairName, startDrill } from '#lib/practice/drill.ts';
	import { chordsIn, type Progression } from '#lib/practice/progressions.ts';
	import { advance, aim, tick, type Practice } from '#lib/practice/session.ts';
	import {
		empty,
		lastSession,
		parse,
		record,
		recordDrill,
		slowPairs,
		week,
		type Change,
		type Played,
		type ProgressData,
		type Session
	} from '#lib/practice/progress.ts';
	import { keepScreenOn } from '#lib/wake-lock.ts';

	/** What the app is doing: listening for chords, tuning, or looking up. */
	type Mode = 'chord' | 'strum' | 'lookup';
	/** The tools that slide up over the listening screen. */
	type SheetName = 'tune' | 'chords' | 'practice';
	/** Chords kept in the history (structured, for key detection later). */
	const HISTORY = 8;
	const LEARN_KEY = 'tuner.learn';
	const LOOKUP_KEY = 'tuner.lookup';
	const PROGRESS_KEY = 'tuner.progress';
	/** Set once the first-run intro has been passed. */
	const PRIMED_KEY = 'tuner.primed';
	/** How long "All in tune" shows before the sheet closes. */
	const TUNED_CLOSE_MS = 1500;
	/** Four Strings is a ukulele app; the core also knows the guitar. */
	const instrument: Instrument = 'ukulele';

	let sheet = $state<SheetName | null>(null);
	// Only tuning and the listening screen use the microphone.
	let mode = $derived<Mode>(sheet === 'tune' ? 'strum' : sheet ? 'lookup' : 'chord');
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

	/** What the player has done (saved on the device). */
	let progress = $state<ProgressData>(empty());
	/** What the player is aiming at, if anything. */
	let practice = $state<Practice | null>(null);
	/** Set when the aimed chord lands, to replay the landed moment. */
	let landedKey = $state<number | null>(null);
	/** The chord on display is a first. */
	let isNewChord = $state(false);
	/** The quick change into the chord on display. */
	let change = $state<Change | null>(null);
	/** Just tuned: the listening card invites a strum. */
	let ready = $state(false);
	/** The strum check found every string in tune. */
	let tuned = $state(false);
	/** The previous session, summed up once on return. */
	let summary = $state<Session | null>(null);
	/**
	 * Past the first-run intro: the app may ask for the microphone. Null
	 * until known (storage and the permission state are checked on load).
	 */
	let primed = $state<boolean | null>(null);
	/** ms since the epoch, ticking while a drill runs. */
	let clock = $state(Date.now());
	/** The last settled chord of this listening stretch, for change times. */
	let previousPlayed: Played | null = null;

	let days = $derived(week(progress, clock));
	let playedKeys = $derived(new Set(Object.keys(progress.chords)));
	let daily = $derived.by(() => {
		const d = chordOfTheDay(new Date(clock), playedKeys);
		return makeTarget(d.root, d.suffix, 'daily')!;
	});
	let heardKey = $derived(detectKey(chords.map((c) => c.analysis)));
	/** Learn mode: why the move into a landed target works. */
	let why = $derived.by(() => {
		if (!learn || practice?.kind !== 'target' || practice.landedAt === null || !heardKey)
			return null;
		if (chords.length < 2) return null;
		const from = chords.at(-2)!.analysis;
		const to = chords.at(-1)!.analysis;
		const degree = (c: typeof from) => (c.root - heardKey!.tonic + 12) % 12;
		return `${numeralIn(heardKey, from)} → ${numeralIn(heardKey, to)} · ${nextReason(heardKey.mode, degree(from), degree(to))}`;
	});
	/** E.g. "Last time: 12 min · 9 chords · best change C → Am 1.3 s". */
	function summaryText(s: Session): string {
		const parts = [
			`Last time: ${Math.max(1, Math.round(s.seconds / 60))} min`,
			`${s.chords} chords`
		];
		if (s.bestChange)
			parts.push(`best change ${s.bestChange.pair} ${s.bestChange.seconds.toFixed(1)} s`);
		return parts.join(' · ');
	}
	/** Collection key ("root:suffix") back to root and suffix. */
	const fromKey = (key: string) => {
		const [root, suffix] = key.split(':');
		return { root: Number(root), suffix };
	};
	/** Pairs to drill: the slowest changes first, then common beginner pairs. */
	let pairs = $derived.by(() => {
		const bySymbol = new Map(
			Object.entries(progress.chords)
				.filter(([k]) => !k.startsWith('symbol:'))
				.map(([k, c]) => [c.symbol, fromKey(k)])
		);
		const named = slowPairs(progress, 4).flatMap((pair) => {
			const [a, b] = pair.split(' → ').map((s) => bySymbol.get(s));
			return a && b ? [[a, b] as const] : [];
		});
		const defaults = [
			[0, '', 9, 'm'],
			[0, '', 5, ''],
			[9, 'm', 5, ''],
			[7, '', 0, ''],
			[0, '', 7, '7']
		].map(
			([r1, s1, r2, s2]) =>
				[
					{ root: r1 as number, suffix: s1 as string },
					{ root: r2 as number, suffix: s2 as string }
				] as const
		);
		const out: { a: Target; b: Target; best: number | null }[] = [];
		for (const [x, y] of [...named, ...defaults]) {
			const a = makeTarget(x.root, x.suffix, 'drill');
			const b = makeTarget(y.root, y.suffix, 'drill');
			if (!a || !b || a.symbol === b.symbol) continue;
			if (out.some((p) => new Set([p.a.symbol, p.b.symbol, a.symbol, b.symbol]).size === 2))
				continue;
			out.push({ a, b, best: progress.drills[pairName(a, b)] ?? null });
			if (out.length === 4) break;
		}
		return out;
	});
	let myChords = $derived(
		Object.entries(progress.chords)
			.map(([key, c]) => ({ ...c, key }))
			.sort((a, b) => a.first.localeCompare(b.first))
	);

	function saveProgress() {
		try {
			localStorage.setItem(PROGRESS_KEY, JSON.stringify(progress));
		} catch {
			// Progress is best-effort.
		}
	}

	/** A short tap on phones that can vibrate (not iOS). */
	function buzz(pattern: number | number[] = 12) {
		try {
			navigator.vibrate?.(pattern);
		} catch {
			// No vibration: the visual moment carries it.
		}
	}

	/** Aims at a chord on the listening screen. */
	function aimAt(root: number, suffix: string, source: TargetSource) {
		const target = makeTarget(root, suffix, source);
		if (!target) return;
		practice = { kind: 'target', target, landedAt: null };
		landedKey = null;
	}
	function play(target: Target) {
		practice = { kind: 'target', target, landedAt: null };
		landedKey = null;
		closeSheet();
	}
	function drill(a: Target, b: Target) {
		practice = { kind: 'drill', drill: startDrill(a, b), result: null };
		landedKey = null;
		closeSheet();
	}
	function playAlong(p: Progression, tonic: number) {
		const names = ['C', 'C#', 'D', 'Eb', 'E', 'F', 'F#', 'G', 'Ab', 'A', 'Bb', 'B'];
		const minor = ['C', 'C#', 'D', 'Eb', 'E', 'F', 'F#', 'G', 'G#', 'A', 'Bb', 'B'];
		practice = {
			kind: 'progression',
			name: p.name,
			keyName: p.mode === 'major' ? `${names[tonic]} major` : `${minor[tonic]} minor`,
			chords: chordsIn(p, tonic),
			index: 0,
			rounds: 0,
			landedAt: null
		};
		landedKey = null;
		closeSheet();
	}
	function finishDrill(p: Extract<Practice, { kind: 'drill' }>) {
		const pair = pairName(p.drill.a, p.drill.b);
		const previousBest = progress.drills[pair] ?? 0;
		const r = recordDrill(progress, pair, p.drill.count);
		progress = r.data;
		saveProgress();
		practice = { ...p, result: { count: p.drill.count, best: r.best, previousBest } };
		buzz([12, 80, 12]);
	}

	// The drill clock, and its end even without another chord.
	$effect(() => {
		const p = practice;
		if (p?.kind !== 'drill' || p.result || p.drill.startedAt === null) return;
		const timer = setInterval(() => {
			clock = Date.now();
			const o = practice && tick(practice, clock);
			if (o?.drillDone) finishDrill(o.practice as Extract<Practice, { kind: 'drill' }>);
		}, 250);
		return () => clearInterval(timer);
	});

	/** Records a settled chord: progress, change time, and the aim. */
	function settled(report: StrumReport) {
		const analysis = report.chord?.analysis;
		if (!analysis) return;
		const t = Date.now();
		clock = t;
		const played: Played = {
			key: chordKey(analysis),
			symbol: analysis.symbol,
			at: t,
			onset: report.onsetSeconds
		};
		const r = record(progress, played, previousPlayed);
		progress = r.data;
		saveProgress();
		previousPlayed = played;
		isNewChord = r.isNew;
		change = r.change;
		ready = false;
		// The summary of last time stays a moment into the new session.
		if ((progress.session?.chords ?? 0) >= 3) summary = null;
		landedKey = null;
		if (practice) {
			const o = advance(practice, analysis, t);
			practice = o.practice;
			if (o.landed) {
				landedKey = t;
				buzz();
			}
			if (o.drillDone) finishDrill(o.practice as Extract<Practice, { kind: 'drill' }>);
		}
	}

	/** First run: the intro is passed, now ask for the microphone. */
	function startListening() {
		primed = true;
		try {
			localStorage.setItem(PRIMED_KEY, '1');
		} catch {
			// The intro may show again next time.
		}
		void sync();
	}

	onMount(async () => {
		// A reload starts on the listening screen, not in a sheet.
		if (history.state?.sheet) history.replaceState(null, '');
		try {
			learn = localStorage.getItem(LEARN_KEY) === '1';
			progress = parse(localStorage.getItem(PROGRESS_KEY));
			summary = lastSession(progress, Date.now());
			if (localStorage.getItem(PRIMED_KEY) === '1') primed = true;
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
		if (!primed) {
			// Allowed before (e.g. storage was cleared): no need to explain again.
			try {
				const state = await navigator.permissions?.query({ name: 'microphone' as PermissionName });
				primed = state?.state === 'granted';
			} catch {
				primed = false; // can't tell: show the intro
			}
		}
		// Ask the browser to keep progress (installed apps usually may).
		void navigator.storage?.persist?.().catch(() => {});
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
		tuned = false;
		// Tuning needs the microphone: asking to tune is the go-ahead.
		if (next === 'tune' && primed === false) startListening();
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
		// The strum check found every string in tune: done, back to playing.
		if (mode === 'strum' && report.settled && !tuned && allInTune(report.strings)) {
			tuned = true;
			buzz([12, 60, 12]);
			setTimeout(() => {
				if (sheet === 'tune') {
					closeSheet();
					ready = true;
				}
				tuned = false;
			}, TUNED_CLOSE_MS);
		}
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
			settled(report);
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
			// First run: the intro explains before the browser asks.
			if (primed !== true) return;
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
			previousPlayed = null;
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
		{#if primed === false}
			<section class="intro" data-testid="intro">
				<ChordDiagram names={['G', 'C', 'E', 'A']} frets={[0, 0, 0, 3]} faint label="C shape" />
				<h2>Strum any chord, I'll name it</h2>
				<p>
					Four Strings listens through your microphone to hear what you play, shows the shape and
					suggests what could come next. Nothing is recorded or sent anywhere.
				</p>
				<button type="button" class="primary" onclick={startListening}>Start listening</button>
				<button type="button" class="link" onclick={() => openSheet('chords')}
					>Just look up chords</button
				>
			</section>
		{:else if primed}
			{#if summary}
				<p class="summary" data-testid="session-summary">
					<span>{summaryText(summary)}</span>
					<button type="button" aria-label="Dismiss" onclick={() => (summary = null)}>
						<Icon name="close" size={16} />
					</button>
				</p>
			{/if}

			{#if !practice && !shownChord}
				<p class="today" data-testid="today">
					<span>Today's chord: <strong>{daily.symbol}</strong></span>
					<button type="button" class="link" onclick={() => play(daily)}>Play it</button>
				</p>
			{/if}

			{#if practice}
				<TargetStrip
					{practice}
					now={clock}
					{why}
					onclear={() => {
						practice = null;
						landedKey = null;
					}}
					onshapes={(t) => {
						lookup = { root: t.root, suffix: t.suffix };
						openSheet('chords');
					}}
					onagain={() => {
						if (practice?.kind === 'drill') drill(practice.drill.a, practice.drill.b);
					}}
				/>
			{/if}

			<ChordView
				{strings}
				report={mode === 'chord' ? (strum?.report ?? null) : null}
				shown={shownChord}
				history={chords}
				listening={running}
				bind:learn
				onTarget={aimAt}
				aimed={aim(practice)}
				{landedKey}
				isNew={isNewChord}
				{change}
				{ready}
				onTune={() => openSheet('tune')}
			/>

			<ProgressStrip {days} onpractice={() => openSheet('practice')} />

			{#if running && !sheet}
				<LevelMeter levelDb={reading?.levelDb ?? null} floorDb={reading?.floorDb ?? null} />
			{/if}
		{/if}
	</main>

	{#if error}
		<footer>
			<p role="alert">{error}</p>
			{#if error === 'Microphone access was denied.'}
				<p class="help" data-testid="denied-help">
					To let Four Strings hear you, allow the microphone for this site: tap the icon next to the
					address (or the site settings in your browser's menu) and switch Microphone on. On an
					installed app, look in your phone's settings under the browser. You can look up chords
					meanwhile.
				</p>
			{/if}
			{#if !engine && tunings}
				<button class="primary" onclick={start} disabled={starting}>Try again</button>
			{/if}
			{#if error === 'Microphone access was denied.'}
				<button type="button" class="link" onclick={() => openSheet('chords')}
					>Look up chords</button
				>
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
			{tuned}
		/>
		{#if running}
			<LevelMeter levelDb={reading?.levelDb ?? null} floorDb={reading?.floorDb ?? null} />
		{/if}
	</Sheet>
{:else if sheet === 'chords'}
	<Sheet title="Chords" onclose={closeSheet} testid="chords-sheet">
		<ChordLookup
			bind:root={lookup.root}
			bind:suffix={lookup.suffix}
			bind:learn
			played={playedKeys}
			onplay={(root, suffix) => {
				aimAt(root, suffix, 'lookup');
				closeSheet();
			}}
		/>
	</Sheet>
{:else if sheet === 'practice'}
	<Sheet title="Practice" onclose={closeSheet} testid="practice-sheet">
		<PracticePanel
			{days}
			{daily}
			dailyPlayed={playedKeys.has(chordKeyOf(daily.root, daily.suffix))}
			{pairs}
			{heardKey}
			{myChords}
			onplay={play}
			ondrill={drill}
			onprogression={playAlong}
			onchord={(key) => {
				lookup = fromKey(key);
				openSheet('chords');
			}}
		/>
	</Sheet>
{/if}

<svelte:window
	onpointerdown={() => engine && !running && wake()}
	onfocus={() => void sync()}
	onblur={() => void sync()}
	onpopstate={() => (sheet = history.state?.sheet ?? null)}
/>
<svelte:document onvisibilitychange={() => void sync()} />
{#if paused && primed === true && mode !== 'lookup'}
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
		min-width: 0;
		white-space: nowrap;
		/* Wide fonts on tiny screens: the name gives way, the tools don't. */
		overflow: hidden;
		text-overflow: ellipsis;
		font-size: 1.2rem;
		font-weight: 700;
		letter-spacing: -0.02em;
	}
	nav {
		flex: none;
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
		display: grid;
		justify-items: center;
		gap: 8px;
	}
	.help {
		margin: 0;
		max-width: 420px;
		color: var(--muted);
		font-size: 0.9rem;
	}
	.link {
		min-height: 40px;
		padding: 0 12px;
		border: 0;
		background: transparent;
		color: var(--accent);
		font: inherit;
		font-weight: 600;
		cursor: pointer;
	}
	.intro {
		display: grid;
		justify-items: center;
		gap: 12px;
		padding: 28px 20px 20px;
		background: var(--surface);
		border: 1px solid var(--line);
		border-radius: var(--radius);
		text-align: center;
	}
	.intro h2 {
		margin: 4px 0 0;
		font-size: 1.4rem;
		letter-spacing: -0.01em;
	}
	.intro p {
		margin: 0;
		max-width: 360px;
		color: var(--muted);
	}
	.today {
		margin: 0;
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		padding: 2px 4px 2px 14px;
		border: 1px dashed var(--line);
		border-radius: var(--radius);
		color: var(--muted);
		font-size: 0.95rem;
	}
	.today strong {
		color: var(--text);
	}
	.summary {
		margin: 0;
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 8px 8px 14px;
		border-radius: var(--radius);
		background: color-mix(in srgb, var(--accent) 10%, var(--surface));
		color: var(--text);
		font-size: 0.9rem;
	}
	.summary span {
		flex: 1;
	}
	.summary button {
		display: grid;
		place-items: center;
		width: 32px;
		height: 32px;
		border: 0;
		border-radius: 999px;
		background: transparent;
		color: var(--muted);
		cursor: pointer;
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
