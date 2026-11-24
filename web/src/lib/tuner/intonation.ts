/**
 * A tuning hint from a recognised chord: strings whose sounding note is
 * clearly off the fretted pitch. A detuned ukulele makes every chord sound
 * wrong, so chord mode points it out.
 */

/** Offsets beyond this many cents are reported. */
export const OFF_CENTS = 15;

export interface OffString {
	/** String name without octave, e.g. "C". */
	name: string;
	/** Offset in cents (negative = flat). */
	cents: number;
}

/** Strings off by more than `OFF_CENTS`, worst first. */
export function offStrings(names: string[], cents: (number | null)[]): OffString[] {
	return cents
		.map((c, i) => ({ name: names[i].replace(/-?\d+$/, ''), cents: c }))
		.filter((s): s is OffString => s.cents !== null && Math.abs(s.cents) > OFF_CENTS)
		.sort((a, b) => Math.abs(b.cents) - Math.abs(a.cents));
}

/** E.g. "C string ~20 c flat" or "C string ~20 c flat, A string ~16 c sharp". */
export function describeOff(off: OffString[]): string {
	return off
		.map(
			(s) =>
				`${s.name} string ~${Math.round(Math.abs(s.cents))} c ${s.cents < 0 ? 'flat' : 'sharp'}`
		)
		.join(', ');
}

/** The strum check calls a string in tune within this many cents. */
export const IN_TUNE_CENTS = 5;

/**
 * Whether a settled strum check found every string sounding and in tune
 * (`strings` in string order, as in a tuning report).
 */
export function allInTune(
	strings: readonly { present: boolean | null; cents: number | null }[]
): boolean {
	return (
		strings.length > 0 &&
		strings.every(
			(s) => s.present !== false && s.cents !== null && Math.abs(s.cents) <= IN_TUNE_CENTS
		)
	);
}
