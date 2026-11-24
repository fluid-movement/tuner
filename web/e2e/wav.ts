import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname } from 'node:path';

// Mono 16-bit PCM WAVs for Chrome's fake microphone
// (`--use-file-for-fake-audio-capture`, which loops the file).

/**
 * Writes a mono WAV of one strum (strings low to high, 15 ms apart), then
 * decay until `period` seconds, so the looping fake microphone re-strums it.
 * A `null` frequency is a muted string (not played at all).
 */
export function writeStrumWav(
	path: string,
	frequencies: (number | null)[],
	{ period = 3, level = 0.05, decay = 1.2, rate = 48_000 } = {}
) {
	const n = Math.round(period * rate);
	const samples = new Float32Array(n);
	frequencies.forEach((frequency, k) => {
		if (frequency === null) return;
		const start = Math.round((0.01 + 0.015 * k) * rate);
		const tauString = decay * (110 / frequency) ** 0.4;
		for (let h = 1; h <= 10; h++) {
			const amp = level / h ** 0.9;
			const tau = tauString / (1 + 0.35 * (h - 1));
			for (let i = start; i < n; i++) {
				const t = (i - start) / rate;
				samples[i] += amp * Math.exp(-t / tau) * Math.sin(2 * Math.PI * frequency * h * t);
			}
		}
	});
	writeSamples(path, samples, rate);
}

/**
 * A chord progression: strum `strums[k]` (frequencies per string, null =
 * muted) at k·`period` seconds; each strum's notes stop at the next strum
 * (re-plucked strings), and the file loops.
 */
export function writeProgressionWav(
	path: string,
	strums: (number | null)[][],
	{ period = 1.5, level = 0.05, decay = 1.2, rate = 48_000 } = {}
) {
	const step = Math.round(period * rate);
	const samples = new Float32Array(step * strums.length);
	strums.forEach((frequencies, s) => {
		const begin = s * step;
		frequencies.forEach((frequency, k) => {
			if (frequency === null) return;
			const start = begin + Math.round((0.01 + 0.015 * k) * rate);
			const end = begin + step;
			const tauString = decay * (110 / frequency) ** 0.4;
			for (let h = 1; h <= 10; h++) {
				const amp = level / h ** 0.9;
				const tau = tauString / (1 + 0.35 * (h - 1));
				for (let i = start; i < end; i++) {
					const t = (i - start) / rate;
					// 10 ms fade before the next strum re-plucks the string.
					const fade = Math.min(1, (end - i) / (0.01 * rate));
					samples[i] += fade * amp * Math.exp(-t / tau) * Math.sin(2 * Math.PI * frequency * h * t);
				}
			}
		});
	});
	writeSamples(path, samples, rate);
}

function writeSamples(path: string, samples: Float32Array, rate: number) {
	const data = Buffer.alloc(samples.length * 2);
	samples.forEach((v, i) =>
		data.writeInt16LE(Math.round(Math.max(-1, Math.min(1, v)) * 32767), i * 2)
	);
	const header = Buffer.alloc(44);
	header.write('RIFF', 0);
	header.writeUInt32LE(36 + data.length, 4);
	header.write('WAVEfmt ', 8);
	header.writeUInt32LE(16, 16);
	header.writeUInt16LE(1, 20);
	header.writeUInt16LE(1, 22);
	header.writeUInt32LE(rate, 24);
	header.writeUInt32LE(rate * 2, 28);
	header.writeUInt16LE(2, 32);
	header.writeUInt16LE(16, 34);
	header.write('data', 36);
	header.writeUInt32LE(data.length, 40);
	mkdirSync(dirname(path), { recursive: true });
	writeFileSync(path, Buffer.concat([header, data]));
}
