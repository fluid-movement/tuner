/** Exports of crates/tuner-wasm. Keep in sync with its `extern "C"` functions. */
export interface TunerExports {
	memory: WebAssembly.Memory;
	tuner_version(): number;
	tuner_note_midi(frequency: number): number;
	tuner_note_cents(frequency: number): number;
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

const NOTE_NAMES = ['C', 'C#', 'D', 'D#', 'E', 'F', 'F#', 'G', 'G#', 'A', 'A#', 'B'];

/** Name with octave in scientific pitch notation, such as `"A2"`. */
export function noteName(midi: number): string {
	return `${NOTE_NAMES[((midi % 12) + 12) % 12]}${Math.floor(midi / 12) - 1}`;
}
