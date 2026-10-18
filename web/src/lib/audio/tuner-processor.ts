// AudioWorkletProcessor that runs the tuner WASM module on the audio thread.
// Bundled on its own by Vite (imported with `?worker&url`), so it must not
// depend on DOM or SvelteKit modules. The worklet scope has no fetch, so the
// module's bytes arrive in processorOptions.

import { FrameAccumulator } from './frame-accumulator.ts';
import {
	PROCESSOR_NAME,
	STRUM_CHUNK,
	type ProcessorControl,
	type ProcessorMessage,
	type ProcessorOptions
} from './messages.ts';
import { WasmMonoTuner, instantiateTuner } from '../wasm/tuner.ts';

// The AudioWorkletGlobalScope is not part of TypeScript's libs.
declare class AudioWorkletProcessor {
	readonly port: MessagePort;
	constructor(options?: { processorOptions?: unknown });
}
declare function registerProcessor(
	name: string,
	ctor: new (options: { processorOptions: ProcessorOptions }) => AudioWorkletProcessor
): void;
declare const sampleRate: number;

class TunerProcessor extends AudioWorkletProcessor {
	#tuner: WasmMonoTuner | null = null;
	#frames: FrameAccumulator | null = null;
	/** Forwards raw audio to the strum worker while strum mode is on. */
	#strumPort: MessagePort | null = null;
	readonly #strumChunks = new FrameAccumulator(STRUM_CHUNK, STRUM_CHUNK);

	constructor(options: { processorOptions: ProcessorOptions }) {
		super(options);
		const { wasmBytes, hop } = options.processorOptions;
		this.port.onmessage = ({ data }: MessageEvent<ProcessorControl>) => {
			if (data.type === 'strum-port') this.#strumPort = data.port;
		};
		instantiateTuner(wasmBytes).then(
			(exports) => {
				this.#tuner = new WasmMonoTuner(exports, sampleRate, hop);
				this.#frames = new FrameAccumulator(this.#tuner.frameSize, hop);
				this.#post({ type: 'ready', frameSize: this.#tuner.frameSize });
			},
			(err: unknown) => this.#post({ type: 'error', message: String(err) })
		);
	}

	process(inputs: Float32Array[][]): boolean {
		const channel = inputs[0]?.[0];
		const strumPort = this.#strumPort;
		if (channel && strumPort) {
			// The worker does the analysis; here we only copy and post.
			this.#strumChunks.push(channel, (chunk) => strumPort.postMessage(chunk.slice()));
		}
		const tuner = this.#tuner;
		if (channel && tuner && this.#frames) {
			this.#frames.push(channel, (frame) => {
				this.#post({ type: 'track', reading: tuner.process(frame) });
			});
		}
		return true;
	}

	#post(message: ProcessorMessage) {
		this.port.postMessage(message);
	}
}

registerProcessor(PROCESSOR_NAME, TunerProcessor);
