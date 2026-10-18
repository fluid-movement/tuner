/** Exports of crates/tuner-wasm. Keep in sync with its `extern "C"` functions. */
export interface TunerExports {
	memory: WebAssembly.Memory;
	tuner_version(): number;
	tuner_note_midi(frequency: number): number;
	tuner_note_cents(frequency: number): number;
	tuner_string_count(instrument: number): number;
	tuner_string_midi(instrument: number, index: number): number;
	tuner_string_frequency(instrument: number, index: number): number;
	tuner_nearest_string(instrument: number, frequency: number): number;
	tuner_cents_between(frequency: number, reference: number): number;
	tuner_new(sampleRate: number, hop: number): number;
	tuner_free(handle: number): void;
	tuner_frame_size(handle: number): number;
	tuner_input(handle: number): number;
	tuner_process(handle: number): number;
	tuner_reset(handle: number): void;
	tuner_frequency(handle: number): number;
	tuner_raw_frequency(handle: number): number;
	tuner_clarity(handle: number): number;
	tuner_level_db(handle: number): number;
	tuner_floor_db(handle: number): number;
	tuner_locked(handle: number): number;
	strum_new(sampleRate: number, instrument: number): number;
	strum_free(handle: number): void;
	strum_input(handle: number): number;
	strum_chunk_size(): number;
	strum_push(handle: number, len: number): number;
	strum_set_instrument(handle: number, instrument: number): void;
	strum_onset(handle: number): number;
	strum_frequency(handle: number, index: number): number;
	strum_cents(handle: number, index: number): number;
	strum_present(handle: number, index: number): number;
}

/** Path of the module under static/, built by `pnpm build:wasm`. */
export const TUNER_WASM_PATH = 'wasm/tuner.wasm' as const;

/**
 * Instantiates the tuner module from its bytes. Takes bytes rather than a URL
 * so the same code works inside an AudioWorklet, which cannot fetch.
 */
export async function instantiateTuner(bytes: BufferSource): Promise<TunerExports> {
	const { instance } = await WebAssembly.instantiate(bytes, {});
	return instance.exports as unknown as TunerExports;
}

/** What the display should show (tuner-core `TrackState`). */
export type TrackState = 'idle' | 'tracking' | 'holding';
const TRACK_STATES: readonly TrackState[] = ['idle', 'tracking', 'holding'];

/** One analysed frame from the note tracker. */
export interface TrackReading {
	state: TrackState;
	/** Smoothed frequency (Hz) of the tracked or held note; null when idle. */
	frequency: number | null;
	/** This frame's raw detector frequency, if any (for diagnostics). */
	rawFrequency: number | null;
	/** This frame's periodicity, 0..1. */
	clarity: number;
	/** This frame's level in dBFS. */
	levelDb: number;
	/** Estimated noise floor in dBFS. */
	floorDb: number;
	/** True on the frame where a (new) note was locked onto. */
	locked: boolean;
}

/**
 * tuner-core's MonoTuner (detector + note tracker) living in the module's
 * linear memory. Call `process` with the latest `frameSize` samples every
 * `hop` samples.
 */
export class WasmMonoTuner {
	readonly frameSize: number;
	readonly #exports: TunerExports;
	#handle: number;

	constructor(exports: TunerExports, sampleRate: number, hop: number) {
		this.#exports = exports;
		this.#handle = exports.tuner_new(sampleRate, hop);
		this.frameSize = exports.tuner_frame_size(this.#handle);
	}

	/** Analyses `frame`, which must hold exactly `frameSize` samples. */
	process(frame: Float32Array): TrackReading {
		const e = this.#exports;
		const h = this.#handle;
		// Views are created per call because memory growth detaches old ones.
		new Float32Array(e.memory.buffer, e.tuner_input(h), this.frameSize).set(frame);
		const state = TRACK_STATES[e.tuner_process(h)] ?? 'idle';
		const frequency = e.tuner_frequency(h);
		const raw = e.tuner_raw_frequency(h);
		return {
			state,
			frequency: frequency > 0 ? frequency : null,
			rawFrequency: raw > 0 ? raw : null,
			clarity: e.tuner_clarity(h),
			levelDb: e.tuner_level_db(h),
			floorDb: e.tuner_floor_db(h),
			locked: e.tuner_locked(h) === 1
		};
	}

	reset(): void {
		this.#exports.tuner_reset(this.#handle);
	}

	free(): void {
		this.#exports.tuner_free(this.#handle);
		this.#handle = 0;
	}
}

export type Instrument = 'guitar' | 'ukulele';
export const INSTRUMENTS: readonly Instrument[] = ['guitar', 'ukulele'];
/** Ids used by the tuner-wasm tuning exports. */
const INSTRUMENT_IDS: Record<Instrument, number> = { guitar: 0, ukulele: 1 };

export interface TunerString {
	/** Position in string order (for ukulele, 0 is the re-entrant G4). */
	index: number;
	midi: number;
	/** Note name with octave, such as `"E2"`. */
	name: string;
	frequency: number;
}

/** Standard tunings and nearest-string lookup, from tuner-core via WASM. */
export class Tunings {
	readonly #exports: TunerExports;
	readonly #cache = new Map<Instrument, TunerString[]>();

	constructor(exports: TunerExports) {
		this.#exports = exports;
	}

	strings(instrument: Instrument): TunerString[] {
		let strings = this.#cache.get(instrument);
		if (!strings) {
			const e = this.#exports;
			const id = INSTRUMENT_IDS[instrument];
			strings = Array.from({ length: e.tuner_string_count(id) }, (_, index) => {
				const midi = e.tuner_string_midi(id, index);
				return {
					index,
					midi,
					name: noteName(midi),
					frequency: e.tuner_string_frequency(id, index)
				};
			});
			this.#cache.set(instrument, strings);
		}
		return strings;
	}

	/** The string nearest to `frequency` and the offset from it in cents. */
	nearest(instrument: Instrument, frequency: number): { string: TunerString; cents: number } {
		const e = this.#exports;
		const string =
			this.strings(instrument)[e.tuner_nearest_string(INSTRUMENT_IDS[instrument], frequency)];
		return { string, cents: e.tuner_cents_between(frequency, string.frequency) };
	}

	/** Note nearest to `frequency`. */
	note(frequency: number): { midi: number; name: string; cents: number } {
		const midi = this.#exports.tuner_note_midi(frequency);
		return { midi, name: noteName(midi), cents: this.#exports.tuner_note_cents(frequency) };
	}
}

/** One string's result in a strum report. */
export interface StrumStringReading {
	/** Estimated frequency in Hz, or null. */
	frequency: number | null;
	/** Offset from the string's target in cents, or null. */
	cents: number | null;
	/** Played? `false` = not heard; `null` = can't tell (guitar B3/E4). */
	present: boolean | null;
}

export interface StrumReport {
	/** Onset of the strum, seconds since listening started. */
	onsetSeconds: number;
	/** In string order of the instrument. */
	strings: StrumStringReading[];
}

/** tuner-core's StrumListener (run it in a Web Worker, not the worklet). */
export class WasmStrumListener {
	readonly #exports: TunerExports;
	readonly #chunk: number;
	#handle: number;
	#strings: number;

	constructor(exports: TunerExports, sampleRate: number, instrument: Instrument) {
		this.#exports = exports;
		this.#handle = exports.strum_new(sampleRate, INSTRUMENT_IDS[instrument]);
		this.#chunk = exports.strum_chunk_size();
		this.#strings = exports.tuner_string_count(INSTRUMENT_IDS[instrument]);
	}

	setInstrument(instrument: Instrument): void {
		this.#exports.strum_set_instrument(this.#handle, INSTRUMENT_IDS[instrument]);
		this.#strings = this.#exports.tuner_string_count(INSTRUMENT_IDS[instrument]);
	}

	/** Feeds audio; returns a report when a strum's analysis has completed. */
	push(samples: Float32Array): StrumReport | null {
		const e = this.#exports;
		let report: StrumReport | null = null;
		for (let offset = 0; offset < samples.length; offset += this.#chunk) {
			const part = samples.subarray(offset, offset + this.#chunk);
			new Float32Array(e.memory.buffer, e.strum_input(this.#handle), part.length).set(part);
			if (e.strum_push(this.#handle, part.length)) report = this.#read();
		}
		return report;
	}

	#read(): StrumReport {
		const e = this.#exports;
		const h = this.#handle;
		return {
			onsetSeconds: e.strum_onset(h),
			strings: Array.from({ length: this.#strings }, (_, i) => {
				const frequency = e.strum_frequency(h, i);
				const cents = e.strum_cents(h, i);
				const present = e.strum_present(h, i);
				return {
					frequency: frequency > 0 ? frequency : null,
					cents: Number.isNaN(cents) ? null : cents,
					present: present < 0 ? null : present === 1
				};
			})
		};
	}

	free(): void {
		this.#exports.strum_free(this.#handle);
		this.#handle = 0;
	}
}

const NOTE_NAMES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];

/** Name with octave in scientific pitch notation, such as `"A2"`. */
export function noteName(midi: number): string {
	return `${NOTE_NAMES[((midi % 12) + 12) % 12]}${Math.floor(midi / 12) - 1}`;
}
