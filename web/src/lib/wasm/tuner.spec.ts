import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { Tunings, WasmMonoTuner, instantiateTuner, noteName } from './tuner';

const bytes = readFileSync(new URL('../../../static/wasm/tuner.wasm', import.meta.url));

function sine(freq: number, rate: number, n: number) {
	return Float32Array.from(
		{ length: n },
		(_, i) => 0.4 * Math.sin((2 * Math.PI * freq * i) / rate)
	);
}

describe('tuner wasm', () => {
	it('loads the built module and calls an export', async () => {
		const tuner = await instantiateTuner(bytes);
		expect(tuner.tuner_version()).toBe(1);
	});

	it('tracks a 110 Hz tone and holds it after silence', async () => {
		const tuner = new WasmMonoTuner(await instantiateTuner(bytes), 48_000, 1024);
		expect(tuner.frameSize).toBe(2048);
		const quiet = Float32Array.from({ length: 2048 }, (_, i) => 1e-4 * Math.sin(i));
		for (let i = 0; i < 5; i++) expect(tuner.process(quiet).state).toBe('idle');

		const tone = sine(110, 48_000, 2048);
		expect(tuner.process(tone).state).toBe('idle'); // a new note is confirmed first
		const locked = tuner.process(tone);
		expect(locked).toMatchObject({ state: 'tracking', locked: true });
		expect(locked.frequency).toBeCloseTo(110, 1);
		expect(locked.clarity).toBeGreaterThan(0.9);
		expect(locked.levelDb).toBeGreaterThan(locked.floorDb + 12);

		let last = locked;
		for (let i = 0; i < 20; i++) last = tuner.process(quiet);
		expect(last.state).toBe('holding');
		expect(last.frequency).toBeCloseTo(110, 1);

		tuner.reset();
		expect(tuner.process(quiet)).toMatchObject({ state: 'idle', frequency: null });
		tuner.free();
	});
});

describe('noteName', () => {
	it('uses sharps and scientific octaves', () => {
		expect([noteName(40), noteName(60), noteName(61), noteName(69), noteName(11)]).toEqual([
			'E2',
			'C4',
			'C#4',
			'A4',
			'B-1'
		]);
	});
});

describe('Tunings', () => {
	it('reads standard tunings and nearest strings from the module', async () => {
		const t = new Tunings(await instantiateTuner(bytes));
		expect(t.strings('guitar').map((s) => s.name)).toEqual(['E2', 'A2', 'D3', 'G3', 'B3', 'E4']);
		expect(t.strings('ukulele').map((s) => s.name)).toEqual(['G4', 'C4', 'E4', 'A4']);
		expect(t.strings('guitar')[1].frequency).toBeCloseTo(110, 4);

		const near = t.nearest('guitar', 108);
		expect(near.string.name).toBe('A2');
		expect(near.cents).toBeCloseTo(1200 * Math.log2(108 / 110), 3);
		expect(t.nearest('ukulele', 393).string.index).toBe(0);
		expect(t.note(446).name).toBe('A4');
	});
});
