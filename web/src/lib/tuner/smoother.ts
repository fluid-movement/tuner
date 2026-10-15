/** Settings for {@link PitchSmoother}. */
export interface SmootherOptions {
	/** Number of recent readings whose median is taken. */
	window: number;
	/** Time constant of the exponential smoothing after the median, in ms. */
	timeConstantMs: number;
	/** A median this far (in cents) from the smoothed value restarts smoothing. */
	resetCents: number;
	/** Without a reading for this long, the smoothed pitch is dropped. */
	timeoutMs: number;
}

export const DEFAULT_SMOOTHER: SmootherOptions = {
	window: 5,
	timeConstantMs: 120,
	resetCents: 40,
	timeoutMs: 250
};

const toCents = (frequency: number) => 1200 * Math.log2(frequency / 440);
const toFrequency = (cents: number) => 440 * 2 ** (cents / 1200);

/**
 * Steadies the per-frame readings for display. The median of recent readings
 * rejects outliers such as octave slips while a string decays; exponential
 * smoothing then removes the remaining jitter. Works in cents so smoothing is
 * equally strong at every pitch.
 */
export class PitchSmoother {
	readonly #options: SmootherOptions;
	#history: number[] = [];
	#smoothed: number | null = null;
	#lastReadingAt = -Infinity;
	#lastUpdateAt = 0;

	constructor(options: Partial<SmootherOptions> = {}) {
		this.#options = { ...DEFAULT_SMOOTHER, ...options };
	}

	/**
	 * Adds a reading (`null` when nothing was detected) taken at `nowMs` and
	 * returns the smoothed frequency, or `null` when there is none.
	 */
	push(frequency: number | null, nowMs: number): number | null {
		const { window, timeConstantMs, resetCents, timeoutMs } = this.#options;
		if (frequency === null || !(frequency > 0)) {
			if (nowMs - this.#lastReadingAt > timeoutMs) this.reset();
			return this.#smoothed === null ? null : toFrequency(this.#smoothed);
		}

		this.#history.push(toCents(frequency));
		if (this.#history.length > window) this.#history.shift();
		const median = medianOf(this.#history);

		if (this.#smoothed === null || Math.abs(median - this.#smoothed) > resetCents) {
			this.#smoothed = median;
		} else {
			const dt = Math.max(0, nowMs - this.#lastUpdateAt);
			const alpha = 1 - Math.exp(-dt / timeConstantMs);
			this.#smoothed += alpha * (median - this.#smoothed);
		}
		this.#lastReadingAt = nowMs;
		this.#lastUpdateAt = nowMs;
		return toFrequency(this.#smoothed);
	}

	reset(): void {
		this.#history = [];
		this.#smoothed = null;
	}
}

function medianOf(values: number[]): number {
	const sorted = [...values].sort((a, b) => a - b);
	const mid = sorted.length >> 1;
	return sorted.length % 2 ? sorted[mid] : (sorted[mid - 1] + sorted[mid]) / 2;
}

/**
 * Whether a string counts as in tune, with hysteresis so the indicator does
 * not flicker at the boundary: it turns on within `enterCents` and off beyond
 * `exitCents`.
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
