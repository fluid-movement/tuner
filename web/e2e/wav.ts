import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname } from 'node:path';

/**
 * A length near `seconds` holding a whole number of cycles (within 1/1000 of
 * one), so the looped file has no click at the seam.
 */
function loopLength(frequency: number, seconds: number, rate: number): number {
	const start = Math.round(seconds * rate);
	let best = start;
	let bestError = 1;
	for (let n = start; n < start * 1.5 && bestError > 1e-3; n++) {
		const cycles = (frequency * n) / rate;
		const error = Math.abs(cycles - Math.round(cycles));
		if (error < bestError) [best, bestError] = [n, error];
	}
	return best;
}

/**
 * Writes a mono 16-bit PCM WAV of a harmonic tone, used as Chrome's fake
 * microphone (`--use-file-for-fake-audio-capture`, which loops the file).
 */
export function writeToneWav(path: string, frequency: number, seconds = 2, rate = 48_000) {
	const n = loopLength(frequency, seconds, rate);
	const data = Buffer.alloc(n * 2);
	const harmonics = [0.3, 0.15, 0.08];
	for (let i = 0; i < n; i++) {
		const t = i / rate;
		const v = harmonics.reduce(
			(s, a, h) => s + a * Math.sin(2 * Math.PI * frequency * (h + 1) * t),
			0
		);
		data.writeInt16LE(Math.round(v * 32767), i * 2);
	}
	const header = Buffer.alloc(44);
	header.write('RIFF', 0);
	header.writeUInt32LE(36 + data.length, 4);
	header.write('WAVEfmt ', 8);
	header.writeUInt32LE(16, 16); // fmt chunk size
	header.writeUInt16LE(1, 20); // PCM
	header.writeUInt16LE(1, 22); // mono
	header.writeUInt32LE(rate, 24);
	header.writeUInt32LE(rate * 2, 28); // byte rate
	header.writeUInt16LE(2, 32); // block align
	header.writeUInt16LE(16, 34); // bits per sample
	header.write('data', 36);
	header.writeUInt32LE(data.length, 40);
	mkdirSync(dirname(path), { recursive: true });
	writeFileSync(path, Buffer.concat([header, data]));
}

/**
 * Writes a mono 16-bit WAV of one plucked-string note: harmonics decaying
 * exponentially (higher ones faster), then silence until `period` seconds,
 * so Chrome's looping fake microphone re-plucks it every `period`.
 */
export function writePluckWav(
	path: string,
	frequency: number,
	{ period = 3, level = 0.02, decay = 1.2, rate = 48_000 } = {}
) {
	const n = Math.round(period * rate);
	const samples = new Float32Array(n);
	for (let h = 1; h <= 8; h++) {
		const amp = level / h ** 0.9;
		const tau = decay / (1 + 0.35 * (h - 1));
		for (let i = 0; i < n; i++) {
			const t = i / rate;
			samples[i] += amp * Math.exp(-t / tau) * Math.sin(2 * Math.PI * frequency * h * t);
		}
	}
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
