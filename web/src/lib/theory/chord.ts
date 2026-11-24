// Chord theory: names any set of notes and says what it is.
//
// For every sounding pitch class as a candidate root, the notes are read as
// intervals above it and sorted into parts: third, fifth, seventh, sixth,
// extensions (9, 11, 13), alterations (b9, #9, #11, b13, b5, #5), sus and
// added tones. A chord symbol is assembled from the parts and scored by how
// complicated the reading is; the simplest reading is the name, the next
// ones are alternatives (0000 on ukulele: Am7, also C6). Any combination of
// notes gets a name; single notes and intervals are described as such.

/** Root spellings by pitch class (0 = C). */
const ROOT_NAMES = ['C', 'C#', 'D', 'Eb', 'E', 'F', 'F#', 'G', 'Ab', 'A', 'Bb', 'B'];
const LETTERS = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];
/** Pitch class of each natural letter. */
const LETTER_PCS = [0, 2, 4, 5, 7, 9, 11];
const INTERVAL_NAMES = [
	'unison',
	'minor second',
	'major second',
	'minor third',
	'major third',
	'perfect fourth',
	'tritone',
	'perfect fifth',
	'minor sixth',
	'major sixth',
	'minor seventh',
	'major seventh'
];
const MAJOR_SCALE = [0, 2, 4, 5, 7, 9, 11];
const NUMERALS = ['I', 'II', 'III', 'IV', 'V', 'VI', 'VII'];

/**
 * How much the lowest sounding note counts as the bass. Low: on a
 * re-entrant ukulele the lowest note is just whatever the C string plays.
 */
const NOT_BASS_COST = 0.2;

/** One note of the chord and its role relative to the root. */
export interface ChordTone {
	/** Pitch class, 0 = C. */
	pc: number;
	/** Spelled for this chord, e.g. "Bb" in C7. */
	name: string;
	/** Role above the root: "R", "3", "b3", "5", "b7", "9", "#11" … */
	role: string;
}

/** A key the chord belongs to and its function there. */
export interface KeyFit {
	/** E.g. "C major". */
	key: string;
	/** Roman numeral with the chord's suffix, e.g. "IVadd9", "vi7", "V7". */
	numeral: string;
}

export interface ChordAnalysis {
	/** Chord symbol, e.g. "Fadd9", "Am7", "C7(b9)". */
	symbol: string;
	/** Root pitch class and its spelling. */
	root: number;
	rootName: string;
	/** In words, e.g. "major triad, added 9th". */
	description: string;
	/** Other readings of the same notes, simplest first. */
	alternatives: string[];
	/** The input notes in input order (e.g. per string), with roles. */
	tones: ChordTone[];
	/** Major keys containing every note, most natural function first. */
	keys: KeyFit[];
	/** Triad quality, for Roman numerals in any key. */
	triad: 'major' | 'minor' | 'diminished' | 'augmented' | 'other';
	/** Symbol after the root and minor "m", e.g. "7", "add9", "m7b5". */
	numeralSuffix: string;
}

interface Reading {
	root: number;
	symbol: string;
	words: string[];
	/** Role per interval above the root (0..11). */
	roles: Map<number, string>;
	score: number;
	/** Triad quality, for Roman numerals. */
	triad: 'major' | 'minor' | 'diminished' | 'augmented' | 'other';
	/** Symbol after the root and quality letters, for Roman numerals. */
	numeralSuffix: string;
}

/** Letters above the root for a role (R → 0, 3 → 2, b7 → 6, 9 → 1 …). */
function degree(role: string): number {
	const n = parseInt(role.replace(/^[b#]+/, ''), 10);
	if (role === 'R') return 0;
	if (role === 'bb7') return 6;
	return (((n - 1) % 7) + 7) % 7;
}

/** Both names of the black-key roots; the one needing fewer accidentals wins. */
const ENHARMONIC: Record<number, string[]> = {
	1: ['C#', 'Db'],
	3: ['Eb', 'D#'],
	6: ['F#', 'Gb'],
	8: ['Ab', 'G#'],
	10: ['Bb', 'A#']
};

/**
 * Spells the note `interval` above `root` (named `rootName`) as the chord
 * degree `role`.
 */
function spell(root: number, interval: number, role: string, rootName = ROOT_NAMES[root]): string {
	const rootLetter = LETTERS.indexOf(rootName[0]);
	const letter = (rootLetter + degree(role)) % 7;
	const pc = (root + interval) % 12;
	const diff = (((pc - LETTER_PCS[letter]) % 12) + 12) % 12;
	const accidental = { 0: '', 1: '#', 2: '##', 11: 'b', 10: 'bb' }[diff];
	// Unusual spellings (more than a double accidental) fall back to the
	// root table's spelling.
	return accidental === undefined ? ROOT_NAMES[pc] : LETTERS[letter] + accidental;
}

/** Reads the pitch-class set (intervals above `root`) as a chord. */
function read(root: number, intervals: Set<number>, bass: number): Reading {
	const has = (i: number) => intervals.has(i);
	const roles = new Map<number, string>([[0, 'R']]);
	const words: string[] = [];
	let score = root === bass ? 0 : NOT_BASS_COST;

	// Third and suspensions.
	let third: 'major' | 'minor' | null = null;
	let sus: '' | 'sus4' | 'sus2' = '';
	if (has(4)) {
		third = 'major';
		roles.set(4, '3');
	} else if (has(3)) {
		third = 'minor';
		roles.set(3, 'b3');
	} else if (has(5)) {
		sus = 'sus4';
		roles.set(5, '4');
	} else if (has(2)) {
		sus = 'sus2';
		roles.set(2, '2');
	}

	// Fifth.
	let fifth: 'perfect' | 'flat' | 'sharp' | null = null;
	if (has(7)) {
		fifth = 'perfect';
		roles.set(7, '5');
	} else if (third === 'minor' && has(6)) {
		fifth = 'flat';
		roles.set(6, 'b5');
	} else if (third === 'major' && has(8)) {
		fifth = 'sharp';
		roles.set(8, '#5');
	} else if (third === 'major' && has(6) && !(has(11) && !has(10))) {
		// Over a major 7th the tritone is the Lydian #11, not a flat fifth.
		fifth = 'flat';
		roles.set(6, 'b5');
	}

	// Seventh (a diminished seventh is the major sixth over a dim triad).
	let seventh: 'minor' | 'major' | 'diminished' | null = null;
	if (has(10)) {
		seventh = 'minor';
		roles.set(10, 'b7');
	} else if (has(11)) {
		seventh = 'major';
		roles.set(11, '7');
	} else if (third === 'minor' && fifth === 'flat' && has(9)) {
		seventh = 'diminished';
		roles.set(9, 'bb7');
	}

	// Sixth (no seventh), extensions and alterations.
	const six = seventh === null && has(9);
	if (six) roles.set(9, '6');
	const nine = has(2) && sus !== 'sus2';
	if (nine) roles.set(2, '9');
	const eleven = has(5) && sus !== 'sus4';
	if (eleven) roles.set(5, '11');
	const thirteen = seventh !== null && seventh !== 'diminished' && has(9);
	if (thirteen) roles.set(9, '13');
	const altered: string[] = [];
	if (has(1)) {
		altered.push('b9');
		roles.set(1, 'b9');
	}
	if (third === 'major' && has(3)) {
		altered.push('#9');
		roles.set(3, '#9');
	}
	if (has(6) && fifth !== 'flat') {
		altered.push('#11');
		roles.set(6, '#11');
	}
	if (has(8) && fifth !== 'sharp') {
		altered.push('b13');
		roles.set(8, 'b13');
	}
	// Both sevenths: the major one is an added tone.
	const extraMaj7 = seventh === 'minor' && has(11);
	if (extraMaj7) roles.set(11, '7');

	// Assemble the symbol.
	let quality = '';
	let triad: Reading['triad'] = 'other';
	let core: string;
	const highest = thirteen ? '13' : eleven && seventh ? '11' : nine && seventh ? '9' : '7';
	const adds: string[] = [];
	if (third === 'minor' && fifth === 'flat') {
		triad = 'diminished';
		if (seventh === 'diminished') {
			core = 'dim7';
			words.push('diminished 7th');
			score += 1.4;
		} else if (seventh === 'minor') {
			core = 'm7b5';
			words.push('half-diminished 7th');
			score += 1.3;
		} else if (seventh === 'major') {
			core = 'dim(maj7)';
			words.push('diminished triad, major 7th');
			score += 2.5;
		} else {
			core = 'dim';
			words.push('diminished triad');
			score += 1.2;
		}
	} else if (third === 'major' && fifth === 'sharp') {
		triad = 'augmented';
		if (seventh === 'minor') {
			core = '7#5';
			words.push('augmented 7th');
			score += 2;
		} else if (seventh === 'major') {
			core = 'maj7#5';
			words.push('augmented major 7th');
			score += 2.2;
		} else {
			core = 'aug';
			words.push('augmented triad');
			score += 1.3;
		}
	} else {
		if (third === 'minor') {
			quality = 'm';
			triad = 'minor';
			score += 0.2;
		} else if (third === 'major') {
			triad = 'major';
		}
		const kind = third === 'minor' ? 'minor' : third === 'major' ? 'major' : '';
		if (seventh === 'minor') {
			core = quality + highest;
			words.push(
				third === 'minor' ? `minor ${highest}th` : third ? `dominant ${highest}th` : `${highest}th`
			);
			score += third === 'minor' ? 0.9 : 0.8;
		} else if (seventh === 'major') {
			core = third === 'minor' ? `m(maj${highest})` : `maj${highest}`;
			words.push(third === 'minor' ? `minor-major ${highest}th` : `major ${highest}th`);
			score += third === 'minor' ? 2 : 1;
		} else if (six) {
			core = quality + (nine ? '6/9' : '6');
			words.push(`${kind ? kind + ' ' : ''}6th${nine ? ' with 9th' : ''}`.trim());
			score += nine ? 1.8 : 1.5;
		} else {
			core = quality;
			if (third) words.push(`${kind} triad`);
		}
		if (seventh && highest !== '7') score += highest === '9' ? 0.4 : 0.6;
		if (fifth === 'flat') altered.unshift('b5');
	}
	// Extensions without a seventh (or above a dim/aug triad) are added tones.
	if (!seventh || triad === 'diminished' || triad === 'augmented') {
		if (nine && !six) adds.push('9');
		if (eleven) adds.push('11');
		if (thirteen) adds.push('13');
		if (six && triad === 'augmented') adds.push('6');
	}
	if (extraMaj7) adds.push('maj7');

	let symbol = ROOT_NAMES[root] + core;
	if (sus) {
		symbol += sus;
		words.push(sus === 'sus4' ? 'suspended 4th' : 'suspended 2nd');
		// sus4 is the more common reading of the same notes (Gsus4 = Csus2).
		score += sus === 'sus4' ? 0.8 : 0.9;
	}
	if (!third && !sus) {
		if (intervals.size === 2 && fifth === 'perfect') {
			symbol = ROOT_NAMES[root] + '5';
			words.push('power chord (root and 5th)');
			score += 0.5;
		} else {
			symbol += '(no3)';
			words.push('no 3rd');
			score += 2.5;
		}
	}
	if (adds.length === 1) symbol += `add${adds[0]}`;
	else if (adds.length > 1) symbol += `(add${adds.join(',')})`;
	for (const a of adds) words.push(`added ${a === 'maj7' ? 'major 7th' : `${a}th`}`);
	score += 1.2 * adds.length;
	if (altered.length) symbol += `(${altered.join(',')})`;
	for (const a of altered) words.push(`${a.replace('b', '♭').replace('#', '♯')}`);
	// Common tensions cost less than rare ones.
	const ALTERATION_COST: Record<string, number> = { '#11': 1, b5: 1.2, '#9': 1.3, b13: 1.3 };
	for (const a of altered) score += ALTERATION_COST[a] ?? 1.5;
	// A missing fifth is normal in four-note voicings; still prefer it there.
	if (fifth === null && third && intervals.size >= 3) score += 0.2;

	// Anything left unexplained (shouldn't happen) is listed as added.
	for (const i of intervals) {
		if (!roles.has(i)) {
			roles.set(i, `+${i}`);
			score += 2;
		}
	}
	// Roman numerals show minor by case: drop the minor "m" (not "maj").
	const numeralSuffix =
		(triad === 'minor' ? core.replace(/^m(?!aj)/, '') : core) +
		symbol.slice((ROOT_NAMES[root] + core).length);
	return { root, symbol, words, roles, score, triad, numeralSuffix };
}

/** Roman numeral of a reading whose root is `degreeIndex` in a major key. */
function numeral(r: Reading, degreeIndex: number): string {
	let n = NUMERALS[degreeIndex];
	let suffix = r.numeralSuffix;
	if (r.triad === 'minor' || r.triad === 'diminished') n = n.toLowerCase();
	if (r.triad === 'diminished') {
		suffix = suffix.replace(/^m7b5/, 'ø7').replace(/^dim/, '°');
	} else if (r.triad === 'augmented') {
		suffix = suffix.replace(/^aug/, '+');
	}
	return n + suffix;
}

/** Major keys containing all of `pcs`, with the reading's function there. */
function keysFor(pcs: Set<number>, r: Reading): KeyFit[] {
	const fits: { fit: KeyFit; rank: number }[] = [];
	for (let key = 0; key < 12; key++) {
		const scale = MAJOR_SCALE.map((s) => (s + key) % 12);
		if (![...pcs].every((pc) => scale.includes(pc))) continue;
		const d = scale.indexOf(r.root);
		// Home, dominant and subdominant functions read most naturally.
		const rank = [0, 4, 3, 5, 1, 2, 6].indexOf(d);
		fits.push({ fit: { key: `${ROOT_NAMES[key]} major`, numeral: numeral(r, d) }, rank });
	}
	return fits.sort((a, b) => a.rank - b.rank).map((f) => f.fit);
}

/**
 * Names and explains the chord formed by `midi` notes (in any order, e.g.
 * per string; doublings allowed). Null for no notes. With `root`, the notes
 * are read over that root (as for a chord looked up by name: 0000 is C6
 * over C, not Am7); other readings become the alternatives.
 */
export function analyseChord(midi: number[], root?: number): ChordAnalysis | null {
	if (!midi.length) return null;
	const pcs = new Set(midi.map((m) => ((m % 12) + 12) % 12));
	const bass = ((Math.min(...midi) % 12) + 12) % 12;
	const readings = [...pcs]
		.map((root) => {
			const intervals = new Set([...pcs].map((pc) => (pc - root + 12) % 12));
			return read(root, intervals, bass);
		})
		.sort((a, b) => a.score - b.score || a.symbol.length - b.symbol.length);
	const forced = root === undefined ? -1 : readings.findIndex((r) => r.root === root);
	if (forced > 0) readings.unshift(...readings.splice(forced, 1));
	const best = readings[0];

	let description = best.words.join(', ');
	if (pcs.size === 1) {
		description = 'a single note';
		best.symbol = ROOT_NAMES[best.root];
	}

	// Spell the root whichever way needs fewer accidentals in the chord.
	const spellAll = (rootName: string) =>
		midi.map((m) => {
			const pc = ((m % 12) + 12) % 12;
			const interval = (pc - best.root + 12) % 12;
			const role = best.roles.get(interval) ?? '?';
			return { pc, name: spell(best.root, interval, role, rootName), role };
		});
	const accidentals = (tones: ChordTone[]) => tones.reduce((n, t) => n + t.name.length - 1, 0);
	let rootName = ROOT_NAMES[best.root];
	let tones = spellAll(rootName);
	for (const other of ENHARMONIC[best.root] ?? []) {
		const t = spellAll(other);
		if (accidentals(t) < accidentals(tones)) [rootName, tones] = [other, t];
	}
	const symbol = rootName + best.symbol.slice(ROOT_NAMES[best.root].length);

	if (pcs.size === 2) {
		// The interval up from the root, e.g. "perfect fifth (C–G)".
		const other = tones.find((t) => t.role !== 'R')!;
		const interval = (other.pc - best.root + 12) % 12;
		description = `${INTERVAL_NAMES[interval]} (${rootName}–${other.name})`;
	}
	const alternatives = [
		...new Set(
			readings
				.slice(1)
				.map((r) => r.symbol)
				.filter((s) => s !== best.symbol)
		)
	].slice(0, 3);

	return {
		symbol,
		root: best.root,
		rootName,
		description,
		alternatives: pcs.size === 1 ? [] : alternatives,
		tones,
		keys: keysFor(pcs, best),
		triad: best.triad,
		numeralSuffix: pcs.size === 1 ? '' : best.numeralSuffix
	};
}

/** Degree names by semitones above the tonic. */
const DEGREES = ['I', 'bII', 'II', 'bIII', 'III', 'IV', '#IV', 'V', 'bVI', 'VI', 'bVII', 'VII'];

/**
 * Roman numeral of a chord in the key with tonic `tonic` (any key, borrowed
 * chords included: bVII, bVI …). Lower case for minor and diminished.
 */
export function romanNumeral(
	tonic: number,
	chord: Pick<ChordAnalysis, 'root' | 'triad' | 'numeralSuffix'>
): string {
	let n = DEGREES[(chord.root - tonic + 12) % 12];
	let suffix = chord.numeralSuffix;
	if (chord.triad === 'minor' || chord.triad === 'diminished') n = n.toLowerCase();
	if (chord.triad === 'diminished') suffix = suffix.replace(/^m7b5/, 'ø7').replace(/^dim/, '°');
	else if (chord.triad === 'augmented') suffix = suffix.replace(/^aug/, '+');
	return n + suffix;
}
