<script lang="ts">
	import { onDestroy } from 'svelte';
	import { describeStartError, startTuner, type TunerEngine } from '#lib/audio/tuner-engine.ts';
	import { noteName, type PitchReading } from '#lib/wasm/tuner.ts';

	let engine: TunerEngine | null = $state(null);
	let starting = $state(false);
	let error: string | null = $state(null);
	let reading: PitchReading | null = $state(null);

	async function start() {
		starting = true;
		error = null;
		try {
			engine = await startTuner({
				onReading: (r) => (reading = r),
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
		reading = null;
	}

	onDestroy(() => engine?.stop());
</script>

<h1>Tuner</h1>

{#if engine}
	<button onclick={stop}>Stop</button>
	<p>Listening at {engine.sampleRate} Hz</p>
	{#if reading}
		<dl data-testid="reading">
			<dt>Frequency</dt>
			<dd data-testid="frequency">{reading.frequency.toFixed(2)} Hz</dd>
			<dt>Note</dt>
			<dd data-testid="note">{noteName(reading.midi)}</dd>
			<dt>Cents</dt>
			<dd>{reading.cents >= 0 ? '+' : ''}{reading.cents.toFixed(1)}</dd>
			<dt>Clarity</dt>
			<dd>{reading.clarity.toFixed(2)}</dd>
		</dl>
	{:else}
		<p>No pitch detected</p>
	{/if}
{:else}
	<button onclick={start} disabled={starting}>{starting ? 'Starting…' : 'Start'}</button>
{/if}

{#if error}
	<p role="alert">{error}</p>
{/if}
