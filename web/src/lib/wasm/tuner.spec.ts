import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { WasmPitchDetector, instantiateTuner, noteName } from './tuner';

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
