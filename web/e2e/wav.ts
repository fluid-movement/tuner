import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname } from 'node:path';

/**
 * Writes a mono 16-bit PCM WAV of a harmonic tone, used as Chrome's fake
 * microphone (`--use-file-for-fake-audio-capture`, which loops the file).
 */
export function writeToneWav(path: string, frequency: number, seconds = 2, rate = 48_000) {
	const n = Math.round(seconds * rate);
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
