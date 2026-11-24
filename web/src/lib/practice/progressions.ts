// Common progressions to play along, built in any key.

import type { Key } from '../theory/key.ts';
import { makeTarget, type Target } from './target.ts';

export interface Progression {
	name: string;
	/** Degrees as semitones above the tonic with chord type suffix. */
	steps: [number, string][];
	mode: 'major' | 'minor';
}

export const PROGRESSIONS: Progression[] = [
	{
		name: 'I–V–vi–IV',
		mode: 'major',
		steps: [
			[0, ''],
			[7, ''],
			[9, 'm'],
			[5, '']
		]
	},
	{
		name: 'I–vi–IV–V',
		mode: 'major',
		steps: [
			[0, ''],
			[9, 'm'],
			[5, ''],
			[7, '']
		]
	},
	{
		name: 'I–IV–V',
		mode: 'major',
		steps: [
			[0, ''],
			[5, ''],
			[7, '']
		]
	},
	{
		name: 'ii–V–I',
		mode: 'major',
		steps: [
			[2, 'm7'],
			[7, '7'],
			[0, 'maj7']
		]
	},
	{
		name: 'vi–IV–I–V',
		mode: 'major',
		steps: [
			[9, 'm'],
			[5, ''],
			[0, ''],
			[7, '']
		]
	},
	{
		name: 'i–VI–III–VII',
		mode: 'minor',
		steps: [
			[0, 'm'],
			[8, ''],
			[3, ''],
			[10, '']
		]
	},
	{
		name: 'i–iv–V',
		mode: 'minor',
		steps: [
			[0, 'm'],
			[5, 'm'],
			[7, '7']
		]
	}
];

/** The progression's chords in `key` (its tonic; the mode comes from the progression). */
export function chordsIn(progression: Progression, tonic: number): Target[] {
	return progression.steps.map(([degree, suffix]) =>
		makeTarget((tonic + degree) % 12, suffix, 'progression')!
	);
}

/** Progressions that suit `key` (same mode first). */
export function progressionsFor(key: Pick<Key, 'mode'> | null): Progression[] {
	const mode = key?.mode ?? 'major';
	return [...PROGRESSIONS].sort((a, b) => Number(b.mode === mode) - Number(a.mode === mode));
}
