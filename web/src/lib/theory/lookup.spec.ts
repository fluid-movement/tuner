import { describe, expect, it } from 'vitest';
import { CHORD_TYPES, ROOTS, lookUpChord, ukuleleShapes } from './lookup.ts';

const type = (suffix: string) => CHORD_TYPES.find((t) => t.suffix === suffix)!;
const look = (root: string, suffix: string) => lookUpChord(ROOTS.indexOf(root), type(suffix));

describe('lookUpChord', () => {
	it.each([
		['C', '', '0003'],
		['A', 'm', '2000'],
		['F', '', '2010'],
		['G', '', '0232'],
		['D', '', '2220'],
		['E', 'm', '0402'],
		['G', '7', '0212'],
		['C', '7', '0001'],
		['A', '7', '0100'],
		['D', 'm', '2210'],
		['Bb', '', '3211'],
		['E', '', '1402'],
		['C', 'maj7', '0002'],
		['A', 'm7', '0000']
	])('finds the common shape for %s%s: %s', (root, suffix, frets) => {
		expect(look(root, suffix).shapes[0].frets.join('')).toBe(frets);
	});

	it('reads the notes over the looked-up root (0000 is C6 here, not Am7)', () => {
		const c6 = look('C', '6');
		expect(c6.symbol).toBe('C6');
		expect(c6.shapes[0].frets.join('')).toBe('0000');
		expect(c6.shapes[0].analysis.alternatives).toContain('Am7');
	});

	it('gives alternatives elsewhere on the neck, lowest first', () => {
		const shapes = look('C', '').shapes;
		expect(shapes.map((s) => s.frets.join('.'))).toEqual([
			'0.0.0.3',
			'5.4.3.3',
			'5.7.8.7',
			'9.7.8.7'
		]);
		const anchors = shapes.slice(1).map((s) => Math.min(...s.frets));
		expect(anchors).toEqual([...anchors].sort((a, b) => a - b));
	});

	it('leaves out the 5th of 9th chords', () => {
		const c9 = look('C', '9');
		expect(c9.symbol).toBe('C9');
		const pcs = new Set(c9.shapes[0].analysis.tones.map((t) => t.pc));
		expect(pcs).toEqual(new Set([0, 4, 10, 2]));
	});

	it('names every root and type as the type, with at least one shape', () => {
		for (let root = 0; root < 12; root++) {
			for (const t of CHORD_TYPES) {
				const found = lookUpChord(root, t);
				expect(found.shapes.length, `${ROOTS[root]}${t.suffix}`).toBeGreaterThan(0);
				expect(found.symbol.endsWith(t.suffix), found.symbol).toBe(true);
				expect(found.shapes[0].analysis.root).toBe(root);
				for (const s of found.shapes) expect(s.frets.every((f) => f >= 0 && f <= 12)).toBe(true);
			}
		}
	});

	it('spells roots the way the chord reads best', () => {
		expect(look('Eb', 'm').symbol).toBe('Ebm');
		expect(look('F#', '').symbol).toBe('F#');
	});
});

describe('ukuleleShapes', () => {
	it('keeps fretted strings within a 3-fret span by default', () => {
		for (const { frets } of ukuleleShapes([0, 4, 7])) {
			const fretted = frets.filter((f) => f > 0);
			if (fretted.length)
				expect(Math.max(...fretted) - Math.min(...fretted)).toBeLessThanOrEqual(3);
		}
	});
});
