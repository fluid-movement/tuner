import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { Tunings, WasmPitchDetector, instantiateTuner, noteName } from './tuner';

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

	it('detects a 110 Hz tone written into its input buffer', async () => {
		const detector = new WasmPitchDetector(await instantiateTuner(bytes), 48_000);
		expect(detector.frameSize).toBe(2048);

		const reading = detector.detect(sine(110, 48_000, 2048));
		expect(reading?.frequency).toBeCloseTo(110, 1);
		expect(reading?.midi).toBe(45);
		expect(noteName(reading!.midi)).toBe('A2');
		expect(Math.abs(reading!.cents)).toBeLessThan(1);

		expect(detector.detect(new Float32Array(2048))).toBeNull();
		detector.free();
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
