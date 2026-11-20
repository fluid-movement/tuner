// Web Worker running the strum analysis (tuner-core StrumListener in its own
// WASM instance). An analysis takes several milliseconds, too long for the
// audio thread: the worklet only forwards audio chunks over a MessagePort.

import type { StrumConfig, StrumWorkerControl, StrumWorkerMessage } from './messages.ts';
import { WasmStrumListener, instantiateTuner } from '../wasm/tuner.ts';

let listener: WasmStrumListener | null = null;
let config: StrumConfig = { instrument: 'ukulele', analysis: 'chord' };

const post = (message: StrumWorkerMessage) => self.postMessage(message);

function apply(next: StrumConfig) {
	config = next;
	listener?.setInstrument(next.instrument);
	listener?.setAnalysis(next.analysis);
}

self.onmessage = async ({ data }: MessageEvent<StrumWorkerControl>) => {
	if (data.type === 'config') {
		apply(data.config);
		return;
	}
	try {
		const exports = await instantiateTuner(data.wasmBytes);
		listener = new WasmStrumListener(exports, data.sampleRate, data.config.instrument);
		apply(data.config);
		data.audio.onmessage = ({ data: chunk }: MessageEvent<Float32Array>) => {
			const report = listener?.push(chunk);
			if (report) post({ type: 'strum', config, report });
		};
		post({ type: 'ready' });
	} catch (err) {
		post({ type: 'error', message: String(err) });
	}
};
