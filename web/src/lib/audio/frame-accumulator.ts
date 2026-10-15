/**
 * Collects audio blocks of any size (the AudioWorklet delivers 128 samples at
 * a time) into overlapping frames: every `hop` samples it hands out the most
 * recent `frameSize` samples. It allocates only in the constructor, so it is
 * safe on the audio thread; the frame passed to the callback is reused.
 */
export class FrameAccumulator {
	readonly frameSize: number;
	readonly hop: number;
	readonly #ring: Float32Array;
	readonly #frame: Float32Array;
	#write = 0;
	#filled = 0;
	#sinceFrame = 0;

	constructor(frameSize: number, hop: number) {
		if (hop <= 0 || hop > frameSize) throw new RangeError('hop must be in 1..=frameSize');
		this.frameSize = frameSize;
		this.hop = hop;
		this.#ring = new Float32Array(frameSize);
		this.#frame = new Float32Array(frameSize);
	}

	push(block: Float32Array, onFrame: (frame: Float32Array) => void): void {
		for (let i = 0; i < block.length; i++) {
			this.#ring[this.#write] = block[i];
			this.#write = (this.#write + 1) % this.frameSize;
			if (this.#filled < this.frameSize) this.#filled++;
			this.#sinceFrame++;
			if (this.#filled === this.frameSize && this.#sinceFrame >= this.hop) {
				this.#sinceFrame = 0;
				// Oldest sample is at the write position.
				const tail = this.#ring.subarray(this.#write);
				this.#frame.set(tail);
				this.#frame.set(this.#ring.subarray(0, this.#write), tail.length);
				onFrame(this.#frame);
			}
		}
	}
}
