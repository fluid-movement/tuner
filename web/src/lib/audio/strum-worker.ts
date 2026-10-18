// Web Worker running the strum check (tuner-core StrumListener in its own
// WASM instance). An analysis takes several milliseconds, too long for the
// audio thread: the worklet only forwards audio chunks over a MessagePort.

import type { StrumWorkerControl, StrumWorkerMessage } from './messages.ts';
import { WasmStrumListener, instantiateTuner } from '../wasm/tuner.ts';

let listener: WasmStrumListener | null = null;

const post = (message: StrumWorkerMessage) => self.postMessage(message);

self.onmessage = async ({ data }: MessageEvent<StrumWorkerControl>) => {
	if (data.type === 'instrument') {
		listener?.setInstrument(data.instrument);
		return;
	}
	try {
		const exports = await instantiateTuner(data.wasmBytes);
		listener = new WasmStrumListener(exports, data.sampleRate, data.instrument);
		data.audio.onmessage = ({ data: chunk }: MessageEvent<Float32Array>) => {
			const report = listener?.push(chunk);
			if (report) post({ type: 'strum', report });
		};
		post({ type: 'ready' });
	} catch (err) {
		post({ type: 'error', message: String(err) });
	}
};
