import { describe, expect, it } from 'vitest';
import { analyseChord, type ChordAnalysis } from './chord.ts';
import { detectKey, findUkuleleShape, numeralIn, suggestNext } from './key.ts';

const UKE = [67, 60, 64, 69];
const uke = (frets: string): ChordAnalysis =>
	analyseChord([...frets].map((f, i) => UKE[i] + Number(f)))!;
const C = uke('0003');
const Am = uke('2000');
const F = uke('2010');
const G = uke('0232');
const G7 = uke('0212');
const Dm = uke('2210');
const E7 = uke('1202');
const D = uke('2220');
const Em = uke('0432');

describe('detectKey', () => {
	it('needs two different chords', () => {
		expect(detectKey([C])).toBeNull();
		expect(detectKey([C, C, C])).toBeNull();
	});

	it.each([
		['C Am F G', [C, Am, F, G], 'C major'],
		['C F G7 C', [C, F, G7, C], 'C major'],
		['Am Dm E7 Am', [Am, Dm, E7, Am], 'A minor'],
		['G D Em C', [G, D, Em, C], 'G major'],
		['Dm G7 C', [Dm, G7, C], 'C major']
	])('%s is in %s', (_, chords, key) => {
		expect(detectKey(chords)!.name).toBe(key);
	});

	it('reads the ambiguous Am F C G as C major or A minor, with low confidence', () => {
		const key = detectKey([Am, F, C, G])!;
		expect(['C major', 'A minor']).toContain(key.name);
		expect(key.confidence).toBeLessThan(detectKey([C, F, G7, C])!.confidence);
	});

	it('is more confident with more evidence', () => {
		const short = detectKey([C, G])!;
		const long = detectKey([C, Am, F, G, C, F, G7, C])!;
		expect(long.confidence).toBeGreaterThan(short.confidence);
	});
});

describe('numeralIn', () => {
	it('labels diatonic and borrowed chords', () => {
		const key = detectKey([C, Am, F, G])!;
		expect([C, Am, F, G, G7, Dm].map((c) => numeralIn(key, c))).toEqual([
			'I',
			'vi',
			'IV',
			'V',
			'V7',
			'ii'
		]);
		expect(numeralIn(key, uke('3211'))).toBe('bVII'); // Bb in C
		const minor = detectKey([Am, Dm, E7, Am])!;
		expect(numeralIn(minor, E7)).toBe('V7');
		expect(numeralIn(minor, C)).toBe('bIII');
	});
});

describe('suggestNext', () => {
	it('follows common progressions with shapes', () => {
		const key = detectKey([C, Am, F, G])!;
		expect(suggestNext(key, G).map((s) => `${s.symbol} ${s.numeral} ${s.frets?.join('')}`)).toEqual(
			['C I 0003', 'Am vi 2000', 'F IV 2010']
		);
		expect(suggestNext(key, Dm)[0].symbol).toBe('G');
		const minor = detectKey([Am, Dm, E7, Am])!;
		expect(suggestNext(minor, E7)[0].symbol).toBe('Am');
	});

	it('says why each chord can come next and how to look it up', () => {
		const key = detectKey([C, Am, F, G])!;
		const [first, second] = suggestNext(key, G);
		expect(first).toMatchObject({
			symbol: 'C',
			root: 0,
			suffix: '',
			reason: 'the dominant resolves home'
		});
		expect(second.reason).toMatch(/deceptive/);
		expect(second).toMatchObject({ root: 9, suffix: 'm' });
		const minor = detectKey([Am, Dm, E7, Am])!;
		expect(suggestNext(minor, E7)[0].reason).toMatch(/raised 7th/);
	});

	it('every suggestion has a reason', () => {
		for (const k of [detectKey([C, Am, F, G])!, detectKey([Am, Dm, E7, Am])!]) {
			for (const last of [C, Am, F, G, Dm, E7]) {
				for (const s of suggestNext(k, last)) expect(s.reason.length).toBeGreaterThan(5);
			}
		}
	});

	it('never suggests the chord just played', () => {
		const key = detectKey([C, F, G])!;
		expect(suggestNext(key, C).map((s) => s.symbol)).not.toContain('C');
	});
});

describe('findUkuleleShape', () => {
	it.each([
		[[0, 4, 7], '0003'], // C
		[[9, 0, 4], '2000'], // Am
		[[5, 9, 0], '2010'], // F
		[[7, 11, 2], '0232'], // G
		[[2, 6, 9], '2220'], // D
		[[10, 2, 5], '3211'] // Bb
	])('%j plays as %s', (pcs, frets) => {
		expect(findUkuleleShape(pcs)!.join('')).toBe(frets);
	});

	it('covers every note and nothing else', () => {
		for (let root = 0; root < 12; root++) {
			for (const t of [
				[0, 4, 7],
				[0, 3, 7],
				[0, 4, 7, 10]
			]) {
				const pcs = t.map((i) => (root + i) % 12);
				const shape = findUkuleleShape(pcs)!;
				const sounding = new Set(shape.map((f, s) => (UKE[s] + f) % 12));
				expect([...sounding].sort()).toEqual([...new Set(pcs)].sort());
			}
		}
	});
});
