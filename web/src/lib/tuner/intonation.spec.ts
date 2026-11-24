import { describe, expect, it } from 'vitest';
import { allInTune, describeOff, offStrings } from './intonation.ts';

const UKE = ['G4', 'C4', 'E4', 'A4'];

describe('offStrings', () => {
	it('reports strings beyond 15 cents, worst first', () => {
		const off = offStrings(UKE, [3, -21.4, null, 16.2]);
		expect(off).toEqual([
			{ name: 'C', cents: -21.4 },
			{ name: 'A', cents: 16.2 }
		]);
		expect(describeOff(off)).toBe('C string ~21 c flat, A string ~16 c sharp');
	});

	it('is empty when everything is close or unmeasured', () => {
		expect(offStrings(UKE, [2, -14.9, null, 0])).toEqual([]);
	});
});

describe('allInTune', () => {
	const s = (cents: number | null, present: boolean | null = true) => ({ present, cents });
	it('needs every string heard and within 5 cents', () => {
		expect(allInTune([s(1), s(-4), s(0), s(5)])).toBe(true);
		expect(allInTune([s(1), s(-6), s(0), s(2)])).toBe(false);
		expect(allInTune([s(1), s(null, false), s(0), s(2)])).toBe(false);
		expect(allInTune([s(1), s(null), s(0), s(2)])).toBe(false);
		expect(allInTune([])).toBe(false);
	});
});
