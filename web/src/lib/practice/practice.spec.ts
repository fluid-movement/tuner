import { describe, expect, it } from 'vitest';
import { analyseChord } from '../theory/chord.ts';
import { UKULELE } from '../theory/lookup.ts';
import { chordKey, chordKeyOf, chordOfTheDay, LEARNING_ORDER } from './daily.ts';
import { DRILL_SECONDS, remaining, startDrill, step } from './drill.ts';
import { chordsIn, PROGRESSIONS } from './progressions.ts';
import {
	CHANGE_MAX_SECONDS,
	SESSION_GAP_MS,
	empty,
	lastSession,
	parse,
	record,
	recordDrill,
	slowPairs,
	minutesText,
	week,
	type Played
} from './progress.ts';
import { hits, makeTarget } from './target.ts';

/** The chord a ukulele shape sounds, e.g. "0003". */
const shape = (frets: string) => analyseChord([...frets].map((f, s) => UKULELE[s] + Number(f)))!;

describe('targets', () => {
	it('hits the same notes in any shape or name', () => {
		const c = makeTarget(0, '', 'idea')!;
		expect(c.symbol).toBe('C');
		expect(c.frets).toEqual([0, 0, 0, 3]);
		expect(hits(c, shape('0003'))).toBe(true);
		expect(hits(c, shape('5433'))).toBe(true);
		expect(hits(c, shape('2000'))).toBe(false); // Am
		// 0000 is Am7 or C6: it hits a C6 target.
		expect(hits(makeTarget(0, '6', 'idea')!, shape('0000'))).toBe(true);
		expect(hits(makeTarget(9, 'm7', 'idea')!, shape('0000'))).toBe(true);
	});

	it('accepts a 9th chord without its 5th, not without its 3rd', () => {
		const c9 = makeTarget(0, '9', 'idea')!;
		expect(hits(c9, shape('3203'))).toBe(true);
		expect(hits(makeTarget(0, '7', 'idea')!, shape('3203'))).toBe(false);
	});

	it('knows no unknown types', () => {
		expect(makeTarget(0, 'zz', 'idea')).toBeNull();
	});
});

const at = (ms: number, symbol: string, onset: number, key = symbol): Played => ({
	key,
	symbol,
	at: ms,
	onset
});

describe('progress', () => {
	const t0 = new Date(2026, 10, 24, 18, 0).getTime();

	it('collects chords, marks the first time as new', () => {
		let r = record(empty(), at(t0, 'C', 1), null);
		expect(r.isNew).toBe(true);
		r = record(r.data, at(t0 + 2000, 'C', 3), at(t0, 'C', 1));
		expect(r.isNew).toBe(false);
		expect(r.data.chords.C).toMatchObject({ symbol: 'C', count: 2, first: '2026-11-24' });
	});

	it('times quick changes and keeps the best per pair', () => {
		const am = at(t0, 'Am', 10);
		let r = record(empty(), am, null);
		r = record(r.data, at(t0 + 1500, 'F', 11.5), am);
		expect(r.change).toEqual({ pair: 'Am → F', seconds: 1.5, best: true });
		r = record(r.data, at(t0 + 4000, 'F', 14), at(t0, 'Am', 12.2));
		expect(r.change?.best).toBe(false);
		expect(r.change?.seconds).toBeCloseTo(1.8);
		expect(r.data.changes['Am → F']).toBe(1.5);
		// Slow ones are just playing on.
		r = record(r.data, at(t0 + 9e4, 'G', 50), at(t0, 'F', 50 - CHANGE_MAX_SECONDS - 1));
		expect(r.change).toBeNull();
	});

	it('adds up playing time per day with idle gaps capped', () => {
		let d = record(empty(), at(t0, 'C', 0), null).data;
		d = record(d, at(t0 + 30_000, 'Am', 30), null).data;
		d = record(d, at(t0 + 10 * 60_000, 'F', 600), null).data; // 9.5 min idle: 60 s
		expect(d.days['2026-11-24']).toBe(90);
		expect(d.session).toMatchObject({ chords: 3, seconds: 90 });
		const w = week(d, t0);
		expect(w).toHaveLength(7);
		expect(w[6]).toMatchObject({ day: '2026-11-24', seconds: 90, today: true });
	});

	it('sums up the last session only once it is over', () => {
		let d = empty();
		for (let i = 0; i < 4; i++) d = record(d, at(t0 + i * 2000, 'C', i * 2), null).data;
		expect(lastSession(d, t0 + 10_000)).toBeNull();
		expect(lastSession(d, t0 + SESSION_GAP_MS + 10_000)).toMatchObject({ chords: 4 });
		// A new session after the gap.
		d = record(d, at(t0 + SESSION_GAP_MS + 20_000, 'G', 0), null).data;
		expect(d.session).toMatchObject({ chords: 1, seconds: 0 });
	});

	it('keeps drill bests and lists slow pairs', () => {
		let { data, best } = recordDrill(empty(), 'C ⇄ Am', 20);
		expect(best).toBe(true);
		({ data, best } = recordDrill(data, 'C ⇄ Am', 18));
		expect(best).toBe(false);
		expect(data.drills['C ⇄ Am']).toBe(20);
		const d = { ...empty(), changes: { 'C → Am': 0.9, 'Am → F': 2.4, 'F → G': 1.6 } };
		expect(slowPairs(d, 2)).toEqual(['Am → F', 'F → G']);
	});

	it('says playing time in words', () => {
		expect(minutesText(10)).toBe('under a minute');
		expect(minutesText(90)).toBe('2 min');
	});

	it('survives broken storage', () => {
		expect(parse('{nope')).toEqual(empty());
		expect(parse(null)).toEqual(empty());
		expect(parse(JSON.stringify({ version: 7 }))).toEqual(empty());
	});
});

describe('chord of the day', () => {
	it('is stable through the day and comes from the next chords to learn', () => {
		const morning = chordOfTheDay(new Date(2026, 10, 24, 8), new Set());
		expect(chordOfTheDay(new Date(2026, 10, 24, 22), new Set())).toEqual(morning);
		const first4 = LEARNING_ORDER.slice(0, 4).map((c) => `${c.root}${c.suffix}`);
		expect(first4).toContain(
			['C', 'C#', 'D', 'Eb', 'E', 'F', 'F#', 'G', 'Ab', 'A', 'Bb', 'B'][morning.root] +
				morning.suffix
		);
	});

	it('moves past chords already played', () => {
		const played = new Set([
			chordKeyOf(0, ''),
			chordKeyOf(9, 'm'),
			chordKeyOf(5, ''),
			chordKeyOf(7, '')
		]);
		const pick = chordOfTheDay(new Date(2026, 10, 24), played);
		expect(played.has(chordKeyOf(pick.root, pick.suffix))).toBe(false);
	});

	it('keys played chords like the lookup', () => {
		expect(chordKey(shape('2000'))).toBe(chordKeyOf(9, 'm'));
		expect(chordKey(shape('0212'))).toBe(chordKeyOf(7, '7'));
	});
});

describe('one-minute changes', () => {
	it('starts on the first chord and counts each switch', () => {
		const c = makeTarget(0, '', 'drill')!;
		const am = makeTarget(9, 'm', 'drill')!;
		let d = startDrill(c, am);
		d = step(d, shape('2000'), 0); // wrong chord first: nothing
		expect(d.startedAt).toBeNull();
		d = step(d, shape('0003'), 1000);
		expect(d).toMatchObject({ startedAt: 1000, count: 0, expect: 'b' });
		d = step(d, shape('2000'), 2000);
		d = step(d, shape('2000'), 2500); // repeat: not a change
		d = step(d, shape('0003'), 3000);
		expect(d.count).toBe(2);
		expect(remaining(d, 31_000)).toBe(30);
		d = step(d, shape('2000'), 1000 + DRILL_SECONDS * 1000);
		expect(d).toMatchObject({ done: true, count: 2 });
	});
});

describe('progressions', () => {
	it('builds I–V–vi–IV in G with shapes', () => {
		const p = PROGRESSIONS.find((x) => x.name === 'I–V–vi–IV')!;
		expect(chordsIn(p, 7).map((t) => `${t.symbol} ${t.frets?.join('')}`)).toEqual([
			'G 0232',
			'D 2220',
			'Em 0402',
			'C 0003'
		]);
	});
});
