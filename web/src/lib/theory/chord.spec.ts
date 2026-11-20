import { describe, expect, it } from 'vitest';
import { analyseChord } from './chord.ts';

/** Ukulele G C E A, frets in string order. */
const UKE = [67, 60, 64, 69];
const uke = (frets: string) => analyseChord([...frets].map((f, i) => UKE[i] + Number(f)))!;
const name = (frets: string) => uke(frets).symbol;

describe('analyseChord on ukulele shapes', () => {
	it.each([
		['0003', 'C'],
		['2000', 'Am'],
		['2010', 'F'],
		['0232', 'G'],
		['0212', 'G7'],
		['0000', 'Am7'],
		['0001', 'C7'],
		['0002', 'Cmaj7'],
		['2210', 'Dm'],
		['2220', 'D'],
		['2223', 'D7'],
		['0432', 'Em'],
		['1202', 'E7'],
		['2100', 'A'],
		['0100', 'A7'],
		['3211', 'Bb'],
		['4222', 'Bm'],
		['2322', 'B7'],
		['2213', 'Dm7'],
		['2413', 'Fmaj7'],
		['0231', 'Gm'],
		['0222', 'Gmaj7'],
		['0013', 'Csus4'],
		['0233', 'Gsus4'],
		['3333', 'Cm7'],
		['1003', 'Caug'],
		['0010', 'Fadd9'],
		['0203', 'Cadd9'],
		['2002', 'Amadd9'],
		['1212', 'Ddim7'],
		['0200', 'A7sus4'],
		['0211', 'Gm7'],
		['2212', 'Bm7b5'],
		['0202', 'Em7'],
		['3433', 'C7']
	])('%s is %s', (frets, expected) => {
		expect(name(frets)).toBe(expected);
	});

	it('explains 0010: notes, roles, alternatives and keys', () => {
		const a = uke('0010');
		expect(a.description).toBe('major triad, added 9th');
		expect(a.tones.map((t) => `${t.name}:${t.role}`)).toEqual(['G:9', 'C:5', 'F:R', 'A:3']);
		expect(a.alternatives.length).toBeGreaterThan(0);
		expect(a.keys.slice(0, 3)).toEqual([
			{ key: 'F major', numeral: 'Iadd9' },
			{ key: 'Bb major', numeral: 'Vadd9' },
			{ key: 'C major', numeral: 'IVadd9' }
		]);
	});

	it('gives the other reading of ambiguous shapes as an alternative', () => {
		expect(uke('0000').alternatives).toContain('C6');
		expect(uke('0233').alternatives).toContain('Csus2');
		expect(uke('2212').alternatives).toContain('Dm6');
		expect(uke('3333').alternatives).toContain('Eb6');
	});

	it('picks the enharmonic root that spells the chord with fewer accidentals', () => {
		const a = uke('0123'); // G C# F# B
		expect(a.symbol.startsWith('Db')).toBe(true);
		expect(a.tones.map((t) => t.name)).toEqual(['G', 'Db', 'Gb', 'C']);
	});

	it('spells notes for the chord', () => {
		expect(uke('3211').tones.map((t) => t.name)).toEqual(['Bb', 'D', 'F', 'Bb']);
		expect(uke('0001').tones.map((t) => `${t.name}:${t.role}`)).toEqual([
			'G:5',
			'C:R',
			'E:3',
			'Bb:b7'
		]);
	});

	it('uses Roman numerals with the right case and symbols', () => {
		expect(uke('0000').keys[0]).toEqual({ key: 'C major', numeral: 'vi7' });
		expect(uke('0212').keys[0]).toEqual({ key: 'C major', numeral: 'V7' });
		const half = analyseChord([71, 74, 77, 81])!; // B D F A
		expect(half.symbol).toBe('Bm7b5');
		expect(half.keys[0].numeral).toBe('viiø7');
	});
});

describe('analyseChord on anything', () => {
	it('names extended and altered chords', () => {
		expect(analyseChord([60, 64, 67, 70, 74])!.symbol).toBe('C9');
		expect(analyseChord([60, 64, 70, 73])!.symbol).toBe('C7(b9)');
		expect(analyseChord([60, 64, 67, 69, 74])!.symbol).toBe('C6/9');
		expect(analyseChord([60, 63, 67, 70, 74, 77])!.symbol).toBe('Cm11');
		expect(analyseChord([60, 64, 70, 74, 81])!.symbol).toBe('C13');
		expect(analyseChord([60, 63, 67, 71])!.symbol).toBe('Cm(maj7)');
		expect(analyseChord([60, 64, 71, 78])!.symbol).toBe('Cmaj7(#11)');
		expect(analyseChord([60, 67])!.symbol).toBe('C5');
	});

	it('describes single notes and intervals', () => {
		expect(analyseChord([60])).toMatchObject({ symbol: 'C', description: 'a single note' });
		expect(analyseChord([60, 64])!.description).toBe('major third (C–E)');
		expect(analyseChord([67, 60, 67, 72])).toMatchObject({
			symbol: 'C5',
			description: 'perfect fifth (C–G)'
		});
	});

	it('names every combination of notes without unexplained tones', () => {
		for (let mask = 1; mask < 1 << 12; mask++) {
			const notes = [...Array(12).keys()].filter((i) => mask & (1 << i)).map((i) => 60 + i);
			const a = analyseChord(notes)!;
			expect(a.symbol.length, notes.join(' ')).toBeGreaterThan(0);
			expect(
				a.tones.every((t) => !t.role.startsWith('+') && t.role !== '?'),
				`${a.symbol}: ${a.tones.map((t) => t.role)}`
			).toBe(true);
		}
	});

	it('is empty for no notes', () => {
		expect(analyseChord([])).toBeNull();
	});
});
