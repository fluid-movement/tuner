// Chord lookup: every ukulele shape for a chord picked by root and type.
//
// A chord type is a set of intervals above the root. Shapes are searched
// over the first 12 frets: each string plays a chord note, every required
// note sounds, fretted strings span at most 3 frets (open strings ring
// anywhere). The easiest shape comes first; alternatives sit elsewhere on
// the neck, lowest position first.

import { analyseChord, type ChordAnalysis } from './chord.ts';

/** Ukulele G C E A, MIDI (re-entrant: G4 above C4). */
export const UKULELE = [67, 60, 64, 69];
/** Root names for the picker, by pitch class (0 = C). */
export const ROOTS = ['C', 'C#', 'D', 'Eb', 'E', 'F', 'F#', 'G', 'Ab', 'A', 'Bb', 'B'];

export interface ChordType {
	/** Symbol after the root ('' for major), also the type's id. */
	suffix: string;
	/** Picker label. */
	label: string;
	/** Intervals above the root in semitones; the root first. */
	intervals: number[];
	/** Intervals a 4-string shape may leave out (the 5th of 5-note chords). */
	optional?: number[];
}

export const CHORD_TYPES: readonly ChordType[] = [
	{ suffix: '', label: 'maj', intervals: [0, 4, 7] },
	{ suffix: 'm', label: 'm', intervals: [0, 3, 7] },
	{ suffix: '7', label: '7', intervals: [0, 4, 7, 10] },
	{ suffix: 'maj7', label: 'maj7', intervals: [0, 4, 7, 11] },
	{ suffix: 'm7', label: 'm7', intervals: [0, 3, 7, 10] },
	{ suffix: '6', label: '6', intervals: [0, 4, 7, 9] },
	{ suffix: 'm6', label: 'm6', intervals: [0, 3, 7, 9] },
	{ suffix: 'sus2', label: 'sus2', intervals: [0, 2, 7] },
	{ suffix: 'sus4', label: 'sus4', intervals: [0, 5, 7] },
	{ suffix: '7sus4', label: '7sus4', intervals: [0, 5, 7, 10] },
	{ suffix: 'add9', label: 'add9', intervals: [0, 4, 7, 2] },
	{ suffix: 'madd9', label: 'madd9', intervals: [0, 3, 7, 2] },
	{ suffix: '9', label: '9', intervals: [0, 4, 7, 10, 2], optional: [7] },
	{ suffix: 'maj9', label: 'maj9', intervals: [0, 4, 7, 11, 2], optional: [7] },
	{ suffix: 'm9', label: 'm9', intervals: [0, 3, 7, 10, 2], optional: [7] },
	{ suffix: 'dim', label: 'dim', intervals: [0, 3, 6] },
	{ suffix: 'dim7', label: 'dim7', intervals: [0, 3, 6, 9] },
	{ suffix: 'm7b5', label: 'm7♭5', intervals: [0, 3, 6, 10] },
	{ suffix: 'aug', label: 'aug', intervals: [0, 4, 8] }
];

const MAX_FRET = 12;
const MAX_SPAN = 3;

export interface Shape {
	/** Frets in string order G C E A. */
	frets: number[];
	/** Lowest fretted fret (0 if all open): where on the neck it sits. */
	position: number;
	/** The shape's notes read over the chord's root. */
	analysis: ChordAnalysis;
}

export interface LookedUp {
	/** E.g. "F#m7", spelled as the theory spells it. */
	symbol: string;
	/** The main shape first, then alternatives lowest position first. */
	shapes: Shape[];
}

const mod12 = (n: number) => ((n % 12) + 12) % 12;

/**
 * Playable shapes sounding only `pcs`, every one of `required` among them,
 * cheapest first.
 */
export function ukuleleShapes(
	pcs: number[],
	required: number[] = pcs,
	maxSpan = MAX_SPAN
): { frets: number[]; cost: number }[] {
	const want = new Set(pcs.map(mod12));
	const need = new Set(required.map(mod12));
	const options = UKULELE.map((open) =>
		[...Array(MAX_FRET + 1).keys()].filter((f) => want.has((open + f) % 12))
	);
	const out: { frets: number[]; cost: number }[] = [];
	const walk = (i: number, frets: number[]) => {
		if (i === UKULELE.length) {
			const covered = new Set(frets.map((f, s) => (UKULELE[s] + f) % 12));
			for (const pc of need) if (!covered.has(pc)) return;
			const fretted = frets.filter((f) => f > 0);
			const low = fretted.length ? Math.min(...fretted) : 0;
			const high = fretted.length ? Math.max(...fretted) : 0;
			// Open strings next to a shape high up are hard to keep ringing
			// cleanly; allow them, at a price.
			const opens = frets.length - fretted.length;
			const cost =
				2 * high +
				frets.reduce((a, b) => a + b, 0) +
				(high - low) +
				(want.size - covered.size) * 2 +
				(high > 5 ? opens * 3 : 0) +
				(high - low > MAX_SPAN ? 4 : 0);
			out.push({ frets: [...frets], cost });
			return;
		}
		for (const f of options[i]) {
			const fretted = [...frets, f].filter((x) => x > 0);
			if (fretted.length && Math.max(...fretted) - Math.min(...fretted) > maxSpan) continue;
			walk(i + 1, [...frets, f]);
		}
	};
	walk(0, []);
	return out.sort((a, b) => a.cost - b.cost);
}

/**
 * The easiest playable shape sounding only `pcs` and all of them (frets G C
 * E A), or null if there is none.
 */
export function findUkuleleShape(pcs: number[]): number[] | null {
	return ukuleleShapes(pcs)[0]?.frets ?? null;
}

/**
 * Looks up `root` (pitch class) with chord `type`: up to `count` shapes,
 * each on its own part of the neck.
 */
export function lookUpChord(root: number, type: ChordType, count = 4): LookedUp {
	const pcs = type.intervals.map((i) => mod12(root + i));
	const required = type.intervals
		.filter((i) => !type.optional?.includes(i))
		.map((i) => mod12(root + i));
	const positionOf = (frets: number[]) => {
		const fretted = frets.filter((f) => f > 0);
		return fretted.length ? Math.min(...fretted) : 0;
	};
	// Where the hand is: open strings count as fret 0, so 0003 and 0007
	// are both first-position shapes.
	const anchor = (frets: number[]) => Math.min(...frets);
	let found = ukuleleShapes(pcs, required);
	// Some 4-note chords need a wider stretch on four strings.
	if (!found.length) found = ukuleleShapes(pcs, required, MAX_SPAN + 1);
	const chosen: number[][] = [];
	for (const { frets } of found) {
		const p = anchor(frets);
		// One shape per area of the neck: skip those close to a chosen one.
		if (chosen.some((c) => Math.abs(anchor(c) - p) < 2)) continue;
		chosen.push(frets);
		if (chosen.length === count) break;
	}
	const [main, ...rest] = chosen;
	const ordered = main ? [main, ...rest.sort((a, b) => anchor(a) - anchor(b))] : [];
	const shapes = ordered.map((frets) => ({
		frets,
		position: positionOf(frets),
		analysis: analyseChord(
			frets.map((f, s) => UKULELE[s] + f),
			root
		)!
	}));
	const symbol = shapes[0]?.analysis.symbol ?? `${ROOTS[root]}${type.suffix}`;
	return { symbol, shapes };
}
