<script lang="ts">
	import { asset } from '$app/paths';
	import { TUNER_WASM_PATH, instantiateTuner } from '#lib/wasm/tuner.ts';

	async function loadVersion() {
		const res = await fetch(asset(TUNER_WASM_PATH));
		const tuner = await instantiateTuner(await res.arrayBuffer());
		return tuner.tuner_version();
	}
</script>

<h1>Tuner</h1>

{#await loadVersion()}
	<p>Loading tuner core…</p>
{:then version}
	<p>Tuner core v{version} loaded.</p>
{:catch error}
	<p>Failed to load tuner core: {error.message}</p>
{/await}
