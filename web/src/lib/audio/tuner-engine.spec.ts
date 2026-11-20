import { afterEach, expect, it, vi } from 'vitest';
import type { StrumWorkerMessage } from './messages.ts';

vi.mock('$app/paths', () => ({ asset: (path: string) => `/${path}` }));
vi.mock('./tuner-processor.ts?worker&url', () => ({ default: '/processor.js' }));

afterEach(() => {
	vi.unstubAllGlobals();
	vi.resetModules();
});

it.each(['before init', 'during init'])(
	'applies strum config changes %s after worker startup and ignores stopped workers',
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
				state = 'running';
				onstatechange = null;
				resume = vi.fn(async () => {});
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
		const guitar = { instrument: 'guitar', analysis: 'tuning' } as const;
		const uke = { instrument: 'ukulele', analysis: 'chord' } as const;
		const onStrum = vi.fn();
		const engine = await startTuner({ onReading: vi.fn(), onStrum });
		try {
			// Switch while the main thread is still awaiting the WASM bytes.
			const starting = engine.setStrum(guitar);
			if (timing === 'before init') await engine.setStrum(uke);
			await starting;
			const worker = workers[0];
			// Config commands during instantiation arrive before the listener exists.
			if (timing === 'during init') await engine.setStrum(uke);
			worker.postMessage.mockClear();
			worker.onmessage?.({ data: { type: 'ready' } });
			expect(worker.postMessage).toHaveBeenLastCalledWith({ type: 'config', config: uke });
			await engine.setStrum(guitar);
			expect(worker.postMessage).toHaveBeenLastCalledWith({ type: 'config', config: guitar });
			const report = { onsetSeconds: 1, settled: true, strings: [], chord: null };
			// A completed report may already be queued when the selection changes.
			worker.onmessage?.({ data: { type: 'strum', config: uke, report } });
			expect(onStrum).not.toHaveBeenCalled();
			worker.onmessage?.({ data: { type: 'strum', config: guitar, report } });
			expect(onStrum).toHaveBeenCalledWith(report);
			onStrum.mockClear();
			await engine.setStrum(null);
			worker.postMessage.mockClear();
			worker.onmessage?.({ data: { type: 'ready' } });
			worker.onmessage?.({ data: { type: 'strum', config: guitar, report } });
			expect(worker.postMessage).not.toHaveBeenCalled();
			expect(onStrum).not.toHaveBeenCalled();
		} finally {
			await engine.stop();
		}
	}
);
