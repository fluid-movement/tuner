// The chord of the day: picked from the date (so it works offline and is
// the same all day), just beyond what the player already plays.

import type { ChordAnalysis } from '../theory/chord.ts';
import { ROOTS, lookupTarget } from '../theory/lookup.ts';

/** Common ukulele chords, roughly in the order players learn them. */
export const LEARNING_ORDER: { root: string; suffix: string }[] = [
	['C', ''],
	['A', 'm'],
	['F', ''],
	['G', ''],
	['G', '7'],
	['C', '7'],
	['D', 'm'],
	['E', 'm'],
	['A', ''],
	['D', ''],
	['A', '7'],
	['E', '7'],
	['A', 'm7'],
	['F', 'maj7'],
	['C', 'maj7'],
	['D', 'm7'],
	['Bb', ''],
	['D', '7'],
	['G', 'm'],
	['E', ''],
	['B', 'm'],
	['A', 'sus4'],
	['D', 'sus2'],
	['C', 'add9'],
	['G', 'sus4'],
	['E', 'm7'],
	['F#', 'm'],
	['B', '7'],
	['Bb', 'maj7'],
	['Eb', ''],
	['C', '6'],
	['G', 'maj7'],
	['D', 'maj7'],
	['C', 'm'],
	['F', 'm'],
	['A', 'maj7'],
	['B', 'm7b5'],
	['C#', 'dim7'],
	['E', 'aug'],
	['G', '9']
].map(([root, suffix]) => ({ root, suffix }));

/** Days since 1970 in local time, for a stable daily pick. */
function dayNumber(date: Date): number {
	return Math.floor((date.getTime() - date.getTimezoneOffset() * 60_000) / 86_400_000);
}

/**
 * Today's chord as { root (pitch class), suffix }: one of the next few
 * chords in LEARNING_ORDER not yet played (`played` holds chordKey values),
 * or any once all are played.
 */
export function chordOfTheDay(date: Date, played: Set<string>): { root: number; suffix: string } {
	const all = LEARNING_ORDER.map((c) => ({ root: ROOTS.indexOf(c.root), suffix: c.suffix }));
	const fresh = all.filter((c) => !played.has(chordKeyOf(c.root, c.suffix)));
	const pool = fresh.length ? fresh.slice(0, 4) : all;
	return pool[dayNumber(date) % pool.length];
}

/** Collection key for a chord by root and lookup type. */
export function chordKeyOf(root: number, suffix: string): string {
	return `${root}:${suffix}`;
}

/** Collection key for a played chord (its lookup entry, else its symbol). */
export function chordKey(analysis: Pick<ChordAnalysis, 'root' | 'rootName' | 'symbol'>): string {
	const t = lookupTarget(analysis);
	return t ? chordKeyOf(t.root, t.suffix) : `symbol:${analysis.symbol}`;
}
