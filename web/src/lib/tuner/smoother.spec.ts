import { describe, expect, it } from 'vitest';
import { InTuneLatch, PitchSmoother } from './smoother';

const cents = (f: number, ref: number) => 1200 * Math.log2(f / ref);
const STEP = 21; // ms between readings

describe('PitchSmoother', () => {
	it('settles on a steady pitch and damps jitter', () => {
		const s = new PitchSmoother();
		let out = 0;
		for (let i = 0; i < 40; i++) {
			const jitter = i % 2 ? 1.003 : 0.997; // ±5 cents
			out = s.push(110 * jitter, i * STEP)!;
		}
		expect(Math.abs(cents(out, 110))).toBeLessThan(1.5);
	});

	it('rejects a single outlier such as an octave slip', () => {
		const s = new PitchSmoother();
		for (let i = 0; i < 10; i++) s.push(110, i * STEP);
		const out = s.push(220, 10 * STEP)!;
		expect(Math.abs(cents(out, 110))).toBeLessThan(0.5);
	});

	it('restarts on a jump to a new string instead of gliding', () => {
		const s = new PitchSmoother();
		for (let i = 0; i < 10; i++) s.push(110, i * STEP);
		let out = 0;
		// The median flips after three readings at the new pitch.
		for (let i = 10; i < 13; i++) out = s.push(146.83, i * STEP)!;
		expect(Math.abs(cents(out, 146.83))).toBeLessThan(0.5);
	});

	it('holds briefly through dropouts, then clears', () => {
		const s = new PitchSmoother({ timeoutMs: 250 });
		s.push(110, 0);
		expect(s.push(null, 100)).toBeCloseTo(110, 3);
		expect(s.push(null, 300)).toBeNull();
		// After clearing, a new reading is taken as is.
		expect(s.push(196, 320)).toBeCloseTo(196, 3);
	});
});

describe('InTuneLatch', () => {
	it('enters within 5 cents and leaves beyond 7', () => {
		const l = new InTuneLatch();
		expect([6, 4.9, 6.5, 7, 7.1, 6, 5].map((c) => l.update(c))).toEqual([
			false,
			true,
			true,
			true,
			false,
			false,
			true
		]);
	});

	it('works for flat offsets and resets without a reading', () => {
		const l = new InTuneLatch();
		expect(l.update(-3)).toBe(true);
		expect(l.update(-6.9)).toBe(true);
		expect(l.update(null)).toBe(false);
		expect(l.update(-6)).toBe(false);
	});
});
