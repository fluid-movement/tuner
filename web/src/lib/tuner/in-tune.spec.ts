import { describe, expect, it } from 'vitest';
import { InTuneLatch } from './in-tune';

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
