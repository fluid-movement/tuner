// Main-thread side of the audio pipeline: microphone → AudioWorklet running
// the tuner WASM module → readings delivered to a callback.

import { asset } from '$app/paths';
import processorUrl from './tuner-processor.ts?worker&url';
import { PROCESSOR_NAME, type ProcessorMessage, type ProcessorOptions } from './messages.ts';
import { TUNER_WASM_PATH, type PitchReading } from '../wasm/tuner.ts';

let bytes: Promise<ArrayBuffer> | undefined;

/**
 * Fetches tuner.wasm once. Callers get a copy, because posting the bytes to
 * the worklet may transfer (detach) the buffer.
 */
export async function loadTunerBytes(): Promise<ArrayBuffer> {
	bytes ??= fetch(asset(TUNER_WASM_PATH)).then((r) => {
		if (!r.ok) throw new Error(`Failed to load ${TUNER_WASM_PATH}: ${r.status}`);
		return r.arrayBuffer();
	});
	bytes.catch(() => (bytes = undefined));
	return (await bytes).slice(0);
}

/** Samples between analyses: about 21 ms at 48 kHz. */
const HOP = 1024;

export interface TunerEngine {
	readonly sampleRate: number;
	stop(): Promise<void>;
}

export interface TunerEngineEvents {
	onReading(reading: PitchReading | null): void;
	onError?(message: string): void;
}

/**
 * Starts listening. Call it from a user gesture (click, tap): browsers only
 * let an AudioContext start and ask for the microphone in response to one.
 * Rejects if the microphone is unavailable or permission is denied.
 */
export async function startTuner(events: TunerEngineEvents): Promise<TunerEngine> {
	// Create the context synchronously inside the gesture.
	const context = new AudioContext({ latencyHint: 'interactive' });
	let stream: MediaStream | undefined;
	try {
		const wasm = loadTunerBytes();
		stream = await navigator.mediaDevices.getUserMedia({
			audio: { echoCancellation: false, noiseSuppression: false, autoGainControl: false }
		});
		await context.audioWorklet.addModule(processorUrl);
		const processorOptions: ProcessorOptions = { wasmBytes: await wasm, hop: HOP };
		const node = new AudioWorkletNode(context, PROCESSOR_NAME, {
			numberOfInputs: 1,
			numberOfOutputs: 1,
			channelCount: 1,
			channelCountMode: 'explicit',
			processorOptions
		});
		node.port.onmessage = ({ data }: MessageEvent<ProcessorMessage>) => {
			if (data.type === 'pitch') events.onReading(data.reading);
			else if (data.type === 'error') events.onError?.(data.message);
		};

		// The output is silent, but routing it to the destination keeps the
		// node processing in every browser.
		const mute = new GainNode(context, { gain: 0 });
		context
			.createMediaStreamSource(stream)
			.connect(node)
			.connect(mute)
			.connect(context.destination);
		await context.resume();

		const tracks = stream.getTracks();
		return {
			sampleRate: context.sampleRate,
			async stop() {
				node.port.onmessage = null;
				tracks.forEach((t) => t.stop());
				await context.close();
			}
		};
	} catch (err) {
		stream?.getTracks().forEach((t) => t.stop());
		await context.close();
		throw err;
	}
}

/** A user-facing message for errors thrown by {@link startTuner}. */
export function describeStartError(err: unknown): string {
	if (err instanceof DOMException) {
		if (err.name === 'NotAllowedError') return 'Microphone access was denied.';
		if (err.name === 'NotFoundError') return 'No microphone was found.';
	}
	return err instanceof Error ? err.message : String(err);
}
