import type { Instrument, StrumReport, TrackReading } from '../wasm/tuner.ts';

/** Registered name of the AudioWorkletProcessor in tuner-processor.ts. */
export const PROCESSOR_NAME = 'tuner-processor';

/** Passed as `processorOptions` when creating the AudioWorkletNode. */
export interface ProcessorOptions {
	wasmBytes: ArrayBuffer;
	/** Samples between analyses. */
	hop: number;
}

/** Messages posted by the processor over its port. */
export type ProcessorMessage =
	| { type: 'ready'; frameSize: number }
	| { type: 'track'; reading: TrackReading }
	| { type: 'error'; message: string };

/** Messages from the main thread to the processor. */
export type ProcessorControl =
	/** Forward audio to the strum worker over this port (null: stop). */
	{ type: 'strum-port'; port: MessagePort | null };

/** Samples per chunk forwarded to the strum worker. */
export const STRUM_CHUNK = 1024;

/** Messages from the main thread to the strum worker. */
export type StrumWorkerControl =
	| {
			type: 'init';
			wasmBytes: ArrayBuffer;
			sampleRate: number;
			instrument: Instrument;
			/** Audio chunks (Float32Array) from the processor arrive here. */
			audio: MessagePort;
	  }
	| { type: 'instrument'; instrument: Instrument };

/** Messages from the strum worker. */
export type StrumWorkerMessage =
	| { type: 'ready' }
	| { type: 'strum'; instrument: Instrument; report: StrumReport }
	| { type: 'error'; message: string };
