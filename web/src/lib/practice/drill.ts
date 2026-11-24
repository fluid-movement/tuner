// One-minute changes: switch between two chords as often as you can in
// 60 seconds; each clean switch counts.

import type { ChordAnalysis } from '../theory/chord.ts';
import { hits, type Target } from './target.ts';

export const DRILL_SECONDS = 60;

export interface Drill {
	a: Target;
	b: Target;
	/** ms since the epoch; null before the first chord starts the clock. */
	startedAt: number | null;
	/** Clean changes so far. */
	count: number;
	/** The chord to play next. */
	expect: 'a' | 'b';
	done: boolean;
}

export const pairName = (a: Target, b: Target) => `${a.symbol} ⇄ ${b.symbol}`;

export function startDrill(a: Target, b: Target): Drill {
	return { a, b, startedAt: null, count: 0, expect: 'a', done: false };
}

/** The expected chord: the drill's current target. */
export const current = (d: Drill): Target => (d.expect === 'a' ? d.a : d.b);

/**
 * A settled chord at `now`: the first hit of A starts the clock; then each
 * hit of the expected chord counts a change. Chords after time are ignored.
 */
export function step(d: Drill, played: Pick<ChordAnalysis, 'tones'>, now: number): Drill {
	if (d.done) return d;
	if (d.startedAt !== null && now - d.startedAt >= DRILL_SECONDS * 1000)
		return { ...d, done: true };
	if (!hits(current(d), played)) return d;
	if (d.startedAt === null) return { ...d, startedAt: now, expect: 'b' };
	return { ...d, count: d.count + 1, expect: d.expect === 'a' ? 'b' : 'a' };
}

/** Seconds left (60 before the start). */
export function remaining(d: Drill, now: number): number {
	if (d.startedAt === null) return DRILL_SECONDS;
	return Math.max(0, DRILL_SECONDS - (now - d.startedAt) / 1000);
}
