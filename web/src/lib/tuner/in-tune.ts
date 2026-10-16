/**
 * Whether a string counts as in tune, with hysteresis so the indicator does
 * not flicker at the boundary: it turns on within `enterCents` and off beyond
 * `exitCents`. (Smoothing of the pitch itself happens in tuner-core's note
 * tracker.)
 */
export class InTuneLatch {
	#inTune = false;

	constructor(
		readonly enterCents = 5,
		readonly exitCents = 7
	) {}

	update(cents: number | null): boolean {
		if (cents === null) this.#inTune = false;
		else if (this.#inTune) this.#inTune = Math.abs(cents) <= this.exitCents;
		else this.#inTune = Math.abs(cents) <= this.enterCents;
		return this.#inTune;
	}
}
