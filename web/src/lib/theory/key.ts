// Key detection, next-chord suggestions and ukulele shapes.
//
// The key is estimated from the recent chords: every chord is scored
// against each of the 24 major and minor keys by how many of its notes are
// in the key's scale, whether its quality matches the key's chord on that
// degree, and its function (tonic, dominant, subdominant); recent chords
// weigh more. Suggestions follow common progressions from the last chord's
// degree (V→I, ii→V, IV→V, vi→IV …), each with a playable ukulele shape
// found by search.

import { analyseChord, romanNumeral, type ChordAnalysis } from './chord.ts';
import { findUkuleleShape } from './lookup.ts';

export { findUkuleleShape };

const NAMES = ['C', 'Db', 'D', 'Eb', 'E', 'F', 'F#', 'G', 'Ab', 'A', 'Bb', 'B'];
const MINOR_NAMES = ['C', 'C#', 'D', 'Eb', 'E', 'F', 'F#', 'G', 'G#', 'A', 'Bb', 'B'];

export type Mode = 'major' | 'minor';
type Triad = ChordAnalysis['triad'];

const SCALES: Record<Mode, number[]> = {
	major: [0, 2, 4, 5, 7, 9, 11],
	minor: [0, 2, 3, 5, 7, 8, 10]
};

/** The key's own triad on each scale degree (minor: harmonic V). */
const DIATONIC: Record<Mode, Record<number, Triad>> = {
	major: {
		0: 'major',
		2: 'minor',
		4: 'minor',
		5: 'major',
		7: 'major',
		9: 'minor',
		11: 'diminished'
	},
	minor: {
		0: 'minor',
		2: 'diminished',
		3: 'major',
		5: 'minor',
		7: 'major',
		8: 'major',
		10: 'major'
	}
};

/** Bonus for chords on the tonic, dominant and subdominant. */
const FUNCTION_WEIGHT: Record<number, number> = { 0: 1, 7: 0.4, 5: 0.3 };
/** Older chords count less: weight RECENCY^age. */
const RECENCY = 0.85;

/** Likely next degrees (semitones above the tonic), most common first. */
const NEXT: Record<Mode, Record<number, number[]>> = {
	major: {
		0: [5, 7, 9, 2],
		2: [7, 5, 9],
		4: [9, 5],
		5: [7, 0, 2],
		7: [0, 9, 5],
		9: [5, 2, 7],
		11: [0, 4]
	},
	minor: {
		0: [5, 8, 10, 7],
		2: [7, 0],
		3: [8, 5],
		5: [7, 0, 10],
		7: [0, 8],
		8: [10, 5, 3],
		10: [3, 0]
	}
};
const FALLBACK_NEXT = [0, 5, 7];

const TRIAD_INTERVALS: Record<Exclude<Triad, 'other'>, number[]> = {
	major: [0, 4, 7],
	minor: [0, 3, 7],
	diminished: [0, 3, 6],
	augmented: [0, 4, 8]
};

export interface Key {
	tonic: number;
	mode: Mode;
	/** E.g. "C major", "F# minor". */
	name: string;
	/** 0..1: how clearly this key beats the next best. */
	confidence: number;
}

export interface Suggestion {
	symbol: string;
	/** Roman numeral in the key, e.g. "IV". */
	numeral: string;
	/** Ukulele frets G C E A, or null if no playable shape was found. */
	frets: number[] | null;
	/** Root pitch class and chord-type suffix ('', 'm', 'dim'), for lookup. */
	root: number;
	suffix: string;
	/** Why it can come next, e.g. "the dominant resolves home". */
	reason: string;
}

/**
 * Why a chord on degree `to` can follow one on `from` (semitones above the
 * tonic): specific moves first, then what the target chord does in the key.
 */
const REASONS: Record<Mode, { moves: Record<string, string>; roles: Record<number, string> }> = {
	major: {
		moves: {
			'7>0': 'the dominant resolves home',
			'5>0': 'the plagal "amen" return home',
			'11>0': 'the leading tone resolves up to the tonic',
			'2>7': 'the classic set-up for the dominant (ii–V–I)',
			'5>7': 'steps up and builds tension before home',
			'0>7': 'out to the dominant: tension that wants to come back',
			'9>7': 'steps down into the dominant',
			'0>5': 'a step away from home, bright and open',
			'9>5': 'the pop-song move (I–V–vi–IV)',
			'7>5': 'a softer turn back, common in rock',
			'2>5': 'a gentle step up, same family as ii',
			'0>9': 'the relative minor: shares two notes with I',
			'7>9': 'the deceptive cadence: you expect I and get vi',
			'4>9': 'down a fifth, the strongest root move',
			'5>2': 'same job as IV, a little darker',
			'9>2': 'down a fifth, heading for V',
			'0>2': 'up a step to a minor colour, often leads to V',
			'11>4': 'down a fifth into iii'
		},
		roles: {
			0: 'home: the tonic',
			2: 'a minor chord that leads to V',
			4: 'shares two notes with I, a soft colour',
			5: 'the subdominant: moves away from home',
			7: 'the dominant: pulls back to I',
			9: 'the relative minor of the key',
			11: 'unstable, wants to resolve to I'
		}
	},
	minor: {
		moves: {
			'7>0': 'the raised 7th in V pulls home to i',
			'5>0': 'the minor plagal return home',
			'10>0': 'the folk and rock return home (♭VII–i)',
			'2>7': 'sets up the dominant (ii°–V–i)',
			'5>7': 'steps up and builds tension before home',
			'0>7': 'out to the dominant: tension that wants to come back',
			'0>5': 'a step away from home, darker',
			'0>8': 'a warm lift to the major ♭VI',
			'3>8': 'down a fifth, the strongest root move',
			'8>10': 'climbs back to the tonic (♭VI–♭VII–i)',
			'5>10': 'a bright step towards ♭III or i',
			'10>3': 'down a fifth to the relative major',
			'8>5': 'steps down to the subdominant',
			'8>3': 'a lift to the relative major',
			'0>10': 'a step down to the major ♭VII'
		},
		roles: {
			0: 'home: the minor tonic',
			2: 'diminished, leads to V',
			3: 'the relative major: same notes as the key',
			5: 'the minor subdominant: moves away from home',
			7: 'the dominant (major in minor keys): pulls to i',
			8: 'the major ♭VI, a warm colour',
			10: 'the major ♭VII, leads to ♭III or i'
		}
	}
};

/** Why degree `to` can follow degree `from` in `mode`. */
export function nextReason(mode: Mode, from: number, to: number): string {
	const r = REASONS[mode];
	return r.moves[`${from}>${to}`] ?? r.roles[to] ?? 'fits the key';
}

const TRIAD_SUFFIX: Record<Exclude<Triad, 'other'>, string> = {
	major: '',
	minor: 'm',
	diminished: 'dim',
	augmented: 'aug'
};

const pcsOf = (c: ChordAnalysis) => [...new Set(c.tones.map((t) => t.pc))];

function keyName(tonic: number, mode: Mode): string {
	return `${(mode === 'major' ? NAMES : MINOR_NAMES)[tonic]} ${mode}`;
}

function chordScore(c: ChordAnalysis, tonic: number, mode: Mode): number {
	const degree = (c.root - tonic + 12) % 12;
	const scale = SCALES[mode].map((s) => (s + tonic) % 12);
	// Minor keys borrow the raised 7th for their major V.
	if (mode === 'minor' && degree === 7 && c.triad === 'major') scale.push((tonic + 11) % 12);
	const pcs = pcsOf(c);
	const fit = pcs.filter((pc) => scale.includes(pc)).length / pcs.length;
	const expected = DIATONIC[mode][degree];
	const quality = expected === undefined ? -1 : expected === c.triad ? 1 : -0.5;
	return 2 * fit + quality + (FUNCTION_WEIGHT[degree] ?? 0);
}

/**
 * The most likely key of `chords` (oldest first), or null with fewer than
 * two different chords.
 */
export function detectKey(chords: ChordAnalysis[]): Key | null {
	if (new Set(chords.map((c) => c.symbol)).size < 2) return null;
	const scores: { tonic: number; mode: Mode; score: number }[] = [];
	for (const mode of ['major', 'minor'] as const) {
		for (let tonic = 0; tonic < 12; tonic++) {
			let score = 0;
			chords.forEach((c, i) => {
				score += RECENCY ** (chords.length - 1 - i) * chordScore(c, tonic, mode);
			});
			// Songs tend to start on the tonic.
			if ((chords[0].root - tonic + 12) % 12 === 0) score += 0.5;
			scores.push({ tonic, mode, score });
		}
	}
	scores.sort((a, b) => b.score - a.score);
	const [best, second] = scores;
	const weight = chords.reduce((w, _, i) => w + RECENCY ** i, 0);
	return {
		tonic: best.tonic,
		mode: best.mode,
		name: keyName(best.tonic, best.mode),
		confidence: Math.max(0, Math.min(1, (best.score - second.score) / weight))
	};
}

/** Roman numeral of `chord` in `key`. */
export function numeralIn(key: Key, chord: ChordAnalysis): string {
	return romanNumeral(key.tonic, chord);
}

/** Likely next chords after `last` in `key`, with ukulele shapes. */
export function suggestNext(key: Key, last: ChordAnalysis, count = 3): Suggestion[] {
	const degree = (last.root - key.tonic + 12) % 12;
	const inKey = DIATONIC[key.mode][degree] === last.triad;
	const degrees = (inKey ? NEXT[key.mode][degree] : undefined) ?? FALLBACK_NEXT;
	const out: Suggestion[] = [];
	for (const d of degrees) {
		const triad = DIATONIC[key.mode][d];
		if (!triad || triad === 'other') continue;
		const root = (key.tonic + d) % 12;
		const pcs = TRIAD_INTERVALS[triad].map((i) => (root + i) % 12);
		const analysis = analyseChord(pcs.map((pc) => 60 + pc))!;
		if (analysis.symbol === last.symbol) continue;
		out.push({
			symbol: analysis.symbol,
			numeral: romanNumeral(key.tonic, analysis),
			frets: findUkuleleShape(pcs),
			root,
			suffix: TRIAD_SUFFIX[triad],
			reason: nextReason(key.mode, inKey ? degree : -1, d)
		});
		if (out.length === count) break;
	}
	return out;
}
