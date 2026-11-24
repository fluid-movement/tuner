// What the player is aiming at on the listening screen.

import type { ChordAnalysis } from '../theory/chord.ts';
import { DRILL_SECONDS, current, startDrill, step, type Drill } from './drill.ts';
import { hits, type Target } from './target.ts';

export type Practice =
	/** One chord to play; `landedAt` once played (it stays). */
	| { kind: 'target'; target: Target; landedAt: number | null }
	/** One-minute changes; `result` once the minute is over. */
	| {
			kind: 'drill';
			drill: Drill;
			result: { count: number; best: boolean; previousBest: number } | null;
	  }
	/** Playing along a progression; it moves on as each chord lands. */
	| {
			kind: 'progression';
			name: string;
			/** E.g. "G major". */
			keyName: string;
			chords: Target[];
			index: number;
			rounds: number;
			landedAt: number | null;
	  };

/** The chord the player should play now, or null (drill over). */
export function aim(p: Practice | null): Target | null {
	if (!p) return null;
	if (p.kind === 'target') return p.target;
	if (p.kind === 'drill') return p.result ? null : current(p.drill);
	return p.chords[p.index];
}

export interface Outcome {
	practice: Practice;
	/** The aimed chord was played: show the landed moment. */
	landed: boolean;
	/** A drill's minute just ended. */
	drillDone: boolean;
}

/** Advances `p` with a settled chord played at `now`. */
export function advance(p: Practice, played: Pick<ChordAnalysis, 'tones'>, now: number): Outcome {
	if (p.kind === 'target') {
		const landed = hits(p.target, played);
		return {
			practice: landed ? { ...p, landedAt: now } : p,
			landed,
			drillDone: false
		};
	}
	if (p.kind === 'drill') {
		if (p.result) return { practice: p, landed: false, drillDone: false };
		const next = step(p.drill, played, now);
		const landed =
			next.count > p.drill.count || (p.drill.startedAt === null && next.startedAt !== null);
		return { practice: { ...p, drill: next }, landed, drillDone: next.done && !p.drill.done };
	}
	if (!hits(p.chords[p.index], played)) return { practice: p, landed: false, drillDone: false };
	const index = (p.index + 1) % p.chords.length;
	return {
		practice: { ...p, index, rounds: p.rounds + (index === 0 ? 1 : 0), landedAt: now },
		landed: true,
		drillDone: false
	};
}

/** The drill ends by the clock even without another chord. */
export function tick(p: Practice, now: number): Outcome | null {
	if (p.kind !== 'drill' || p.result || p.drill.startedAt === null) return null;
	if (now - p.drill.startedAt < DRILL_SECONDS * 1000) return null;
	return { practice: { ...p, drill: { ...p.drill, done: true } }, landed: false, drillDone: true };
}

export { startDrill };
