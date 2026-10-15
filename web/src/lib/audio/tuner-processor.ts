// AudioWorkletProcessor that runs the tuner WASM module on the audio thread.
// Bundled on its own by Vite (imported with `?worker&url`), so it must not
// depend on DOM or SvelteKit modules. The worklet scope has no fetch, so the
// module's bytes arrive in processorOptions.

import { FrameAccumulator } from './frame-accumulator.ts';
import { PROCESSOR_NAME, type ProcessorMessage, type ProcessorOptions } from './messages.ts';
import { WasmPitchDetector, instantiateTuner } from '../wasm/tuner.ts';

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
	#detector: WasmPitchDetector | null = null;
	#frames: FrameAccumulator | null = null;

	constructor(options: { processorOptions: ProcessorOptions }) {
		super(options);
		const { wasmBytes, hop } = options.processorOptions;
		instantiateTuner(wasmBytes).then(
			(exports) => {
				this.#detector = new WasmPitchDetector(exports, sampleRate);
				this.#frames = new FrameAccumulator(this.#detector.frameSize, hop);
				this.#post({ type: 'ready', frameSize: this.#detector.frameSize });
			},
			(err: unknown) => this.#post({ type: 'error', message: String(err) })
		);
	}

	process(inputs: Float32Array[][]): boolean {
		const channel = inputs[0]?.[0];
		const detector = this.#detector;
		if (channel && detector && this.#frames) {
			this.#frames.push(channel, (frame) => {
				this.#post({ type: 'pitch', reading: detector.detect(frame) });
			});
		}
		return true;
	}

	#post(message: ProcessorMessage) {
		this.port.postMessage(message);
	}
}

registerProcessor(PROCESSOR_NAME, TunerProcessor);
