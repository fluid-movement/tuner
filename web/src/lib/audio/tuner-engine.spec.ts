import { afterEach, expect, it, vi } from 'vitest';
import type { StrumWorkerMessage } from './messages.ts';

vi.mock('$app/paths', () => ({ asset: (path: string) => `/${path}` }));
vi.mock('./tuner-processor.ts?worker&url', () => ({ default: '/processor.js' }));

afterEach(() => {
	vi.unstubAllGlobals();
	vi.resetModules();
});

it.each(['before init', 'during init'])(
	'applies instrument changes %s after worker startup and ignores stopped workers',
	async (timing) => {
		const connect = vi.fn();
		const node = { port: { postMessage: vi.fn(), onmessage: null }, connect };
		const mute = { connect };
		connect.mockReturnValue(mute);
		vi.stubGlobal(
			'AudioContext',
			class {
				sampleRate = 48_000;
				audioWorklet = { addModule: vi.fn() };
				createMediaStreamSource() {
					return { connect: () => node };
				}
				resume = vi.fn();
				close = vi.fn();
			}
		);
		vi.stubGlobal(
			'AudioWorkletNode',
			class {
				constructor() {
					return node;
				}
			}
		);
		vi.stubGlobal(
			'GainNode',
			class {
				constructor() {
					return mute;
				}
			}
		);
		vi.stubGlobal('navigator', {
			mediaDevices: { getUserMedia: async () => ({ getTracks: () => [{ stop: vi.fn() }] }) }
		});
		vi.stubGlobal('fetch', async () => ({ ok: true, arrayBuffer: async () => new ArrayBuffer(8) }));
		vi.stubGlobal(
			'MessageChannel',
			class {
				port1 = {};
				port2 = {};
			}
		);
		const workers: FakeWorker[] = [];
		class FakeWorker {
			onmessage: ((event: { data: StrumWorkerMessage }) => void) | null = null;
			postMessage = vi.fn();
			terminate = vi.fn();
			constructor() {
				workers.push(this);
			}
		}
		vi.stubGlobal('Worker', FakeWorker);
		const { startTuner } = await import('./tuner-engine.ts');
		const onStrum = vi.fn();
		const engine = await startTuner({ onReading: vi.fn(), onStrum });
		try {
			// Switch while the main thread is still awaiting the WASM bytes.
			const starting = engine.setStrum('guitar');
			if (timing === 'before init') await engine.setStrum('ukulele');
			await starting;
			const worker = workers[0];
			// Instrument commands during instantiation arrive before the listener exists.
			if (timing === 'during init') await engine.setStrum('ukulele');
			worker.postMessage.mockClear();
			worker.onmessage?.({ data: { type: 'ready' } });
			expect(worker.postMessage).toHaveBeenLastCalledWith({
				type: 'instrument',
				instrument: 'ukulele'
			});
			await engine.setStrum('guitar');
			expect(worker.postMessage).toHaveBeenLastCalledWith({
				type: 'instrument',
				instrument: 'guitar'
			});
			const report = { onsetSeconds: 1, strings: [] };
			// A completed report may already be queued when the selection changes.
			worker.onmessage?.({ data: { type: 'strum', instrument: 'ukulele', report } });
			expect(onStrum).not.toHaveBeenCalled();
			worker.onmessage?.({ data: { type: 'strum', instrument: 'guitar', report } });
			expect(onStrum).toHaveBeenCalledWith(report);
			onStrum.mockClear();
			await engine.setStrum(null);
			worker.postMessage.mockClear();
			worker.onmessage?.({ data: { type: 'ready' } });
			worker.onmessage?.({ data: { type: 'strum', instrument: 'guitar', report } });
			expect(worker.postMessage).not.toHaveBeenCalled();
			expect(onStrum).not.toHaveBeenCalled();
		} finally {
			await engine.stop();
		}
	}
);
