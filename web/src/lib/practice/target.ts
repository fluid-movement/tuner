// A chord the player is aiming at, and whether a played chord hits it.

import type { ChordAnalysis } from '../theory/chord.ts';
import { CHORD_TYPES, lookUpChord } from '../theory/lookup.ts';

export type TargetSource = 'idea' | 'history' | 'lookup' | 'daily' | 'drill' | 'progression';

export interface Target {
	root: number;
	/** Chord-type suffix as in the lookup ('' major, 'm', '7' …). */
	suffix: string;
	/** Spelled symbol, e.g. "F#m". */
	symbol: string;
	/** Easiest ukulele shape (G C E A), or null. */
	frets: number[] | null;
	/** Every pitch class of the chord. */
	pcs: number[];
	/** Pitch classes a voicing must contain (all but an optional 5th). */
	required: number[];
	source: TargetSource;
}

const mod12 = (n: number) => ((n % 12) + 12) % 12;

/** The target for `root` with chord type `suffix`, or null for an unknown type. */
export function makeTarget(root: number, suffix: string, source: TargetSource): Target | null {
	const type = CHORD_TYPES.find((t) => t.suffix === suffix);
	if (!type) return null;
	const found = lookUpChord(root, type, 1);
	return {
		root,
		suffix,
		symbol: found.symbol,
		frets: found.shapes[0]?.frets ?? null,
		pcs: type.intervals.map((i) => mod12(root + i)),
		required: type.intervals.filter((i) => !type.optional?.includes(i)).map((i) => mod12(root + i)),
		source
	};
}

/**
 * Whether `played` is the target: the same notes (any voicing, any name for
 * them, e.g. 0000 as C6 or Am7), an optional 5th may be missing.
 */
export function hits(target: Target, played: Pick<ChordAnalysis, 'tones'>): boolean {
	const notes = new Set(played.tones.map((t) => mod12(t.pc)));
	return (
		[...notes].every((pc) => target.pcs.includes(pc)) &&
		target.required.every((pc) => notes.has(pc))
	);
}
