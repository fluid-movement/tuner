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
	tuner_new(sampleRate: number): number;
	tuner_free(handle: number): void;
	tuner_frame_size(handle: number): number;
	tuner_input(handle: number): number;
	tuner_detect(handle: number): number;
	tuner_frequency(handle: number): number;
	tuner_clarity(handle: number): number;
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

export interface PitchReading {
	/** Fundamental frequency in Hz. */
	frequency: number;
	/** Periodicity of the frame at that frequency, 0..1. */
	clarity: number;
	/** MIDI number of the nearest equal-tempered note. */
	midi: number;
	/** Offset from that note in cents, -50..50 (positive is sharp). */
	cents: number;
}

/** A tuner-core PitchDetector living in the module's linear memory. */
export class WasmPitchDetector {
	readonly frameSize: number;
	readonly #exports: TunerExports;
	#handle: number;

	constructor(exports: TunerExports, sampleRate: number) {
		this.#exports = exports;
		this.#handle = exports.tuner_new(sampleRate);
		this.frameSize = exports.tuner_frame_size(this.#handle);
	}

	/** Analyses `frame`, which must hold exactly `frameSize` samples. */
	detect(frame: Float32Array): PitchReading | null {
		const e = this.#exports;
		// Views are created per call because memory growth detaches old ones.
		new Float32Array(e.memory.buffer, e.tuner_input(this.#handle), this.frameSize).set(frame);
		if (!e.tuner_detect(this.#handle)) return null;
		const frequency = e.tuner_frequency(this.#handle);
		return {
			frequency,
			clarity: e.tuner_clarity(this.#handle),
			midi: e.tuner_note_midi(frequency),
			cents: e.tuner_note_cents(frequency)
		};
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

const NOTE_NAMES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];

/** Name with octave in scientific pitch notation, such as `"A2"`. */
export function noteName(midi: number): string {
	return `${NOTE_NAMES[((midi % 12) + 12) % 12]}${Math.floor(midi / 12) - 1}`;
}
