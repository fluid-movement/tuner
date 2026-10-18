// Web Worker running the strum check (tuner-core StrumListener in its own
// WASM instance). An analysis takes several milliseconds, too long for the
// audio thread: the worklet only forwards audio chunks over a MessagePort.

import type { StrumWorkerControl, StrumWorkerMessage } from './messages.ts';
import { WasmStrumListener, instantiateTuner } from '../wasm/tuner.ts';
import type { Instrument } from '../wasm/tuner.ts';

let listener: WasmStrumListener | null = null;
let instrument: Instrument = 'guitar';

const post = (message: StrumWorkerMessage) => self.postMessage(message);

self.onmessage = async ({ data }: MessageEvent<StrumWorkerControl>) => {
	if (data.type === 'instrument') {
		instrument = data.instrument;
		listener?.setInstrument(data.instrument);
		return;
	}
	try {
		instrument = data.instrument;
		const exports = await instantiateTuner(data.wasmBytes);
		listener = new WasmStrumListener(exports, data.sampleRate, instrument);
		data.audio.onmessage = ({ data: chunk }: MessageEvent<Float32Array>) => {
			const report = listener?.push(chunk);
			if (report) post({ type: 'strum', instrument, report });
		};
		post({ type: 'ready' });
	} catch (err) {
		post({ type: 'error', message: String(err) });
	}
};
