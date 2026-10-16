// AudioWorkletProcessor that runs the tuner WASM module on the audio thread.
// Bundled on its own by Vite (imported with `?worker&url`), so it must not
// depend on DOM or SvelteKit modules. The worklet scope has no fetch, so the
// module's bytes arrive in processorOptions.

import { FrameAccumulator } from './frame-accumulator.ts';
import { PROCESSOR_NAME, type ProcessorMessage, type ProcessorOptions } from './messages.ts';
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

	constructor(options: { processorOptions: ProcessorOptions }) {
		super(options);
		const { wasmBytes, hop } = options.processorOptions;
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
