import { describe, expect, it } from 'vitest';
import { FrameAccumulator } from './frame-accumulator';

/** Blocks of `size` samples whose values are their running index. */
function* blocks(count: number, size: number) {
	for (let b = 0; b < count; b++) {
		yield Float32Array.from({ length: size }, (_, i) => b * size + i);
	}
}

describe('FrameAccumulator', () => {
	it('emits the latest frameSize samples every hop samples, in order', () => {
		const acc = new FrameAccumulator(2048, 1024);
		const frames: number[][] = [];
		for (const block of blocks(64, 128)) {
			acc.push(block, (f) => frames.push([f[0], f[f.length - 1], f.length]));
		}
		// 8192 samples: first frame once 2048 have arrived, then every 1024.
		expect(frames).toEqual([
			[0, 2047, 2048],
			[1024, 3071, 2048],
			[2048, 4095, 2048],
			[3072, 5119, 2048],
			[4096, 6143, 2048],
			[5120, 7167, 2048],
			[6144, 8191, 2048]
		]);
	});

	it('keeps frames contiguous across the ring wrap with odd block sizes', () => {
		const acc = new FrameAccumulator(16, 5);
		let checked = 0;
		for (const block of blocks(20, 7)) {
			acc.push(block, (f) => {
				for (let i = 1; i < f.length; i++) expect(f[i]).toBe(f[i - 1] + 1);
				checked++;
			});
		}
		expect(checked).toBeGreaterThan(20);
	});

	it('rejects a hop larger than the frame', () => {
		expect(() => new FrameAccumulator(16, 17)).toThrow(RangeError);
	});
});
