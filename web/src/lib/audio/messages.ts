import type { TrackReading } from '../wasm/tuner.ts';

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
