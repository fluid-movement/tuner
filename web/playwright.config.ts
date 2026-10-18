import { defineConfig } from '@playwright/test';
import { fileURLToPath } from 'node:url';
import { writePluckWav, writeStrumWav, writeToneWav } from './e2e/wav.ts';

/**
 * Tones played by Chrome's fake microphone. Chrome takes the file at launch,
 * so each tone is a project; tests opt in with a tag such as `@e2`.
 */
export const TONES = {
	a2: 110,
	e2: 82.40689,
	'a2-flat': 110 * 2 ** (-10 / 1200),
	c4: 261.62557
} as const;

/** Test an already running deployment instead of a local preview build. */
const external = process.env.E2E_BASE_URL;
/** Preview port; set E2E_PORT to run several checkouts' tests in parallel. */
const port = Number(process.env.E2E_PORT ?? 4173);

/** A quiet (≈ −34 dBFS peak) decaying A2 pluck, re-plucked every 3 s. */
export const PLUCK = { name: 'pluck', frequency: 110, period: 3 } as const;

const cents = (f: number, c: number) => f * 2 ** (c / 1200);
/** Strums with known detuning (cents from standard tuning, null = muted). */
export const STRUMS = {
	'strum-guitar': {
		instrument: 'Guitar',
		strings: ['E2', 'A2', 'D3', 'G3', 'B3', 'E4'],
		cents: [1, -2, 0, 3, 9, -12],
		frequencies: [82.40689, 110, 146.83238, 195.99772, 246.94165, 329.62756]
	},
	'strum-uke-muted': {
		instrument: 'Ukulele',
		strings: ['G4', 'C4', 'E4', 'A4'],
		cents: [-15, 4, 0, null],
		frequencies: [391.99544, 261.62557, 329.62756, 440]
	}
} as const;

function fakeMic(name: string, write: (path: string) => void) {
	const wav = fileURLToPath(new URL(`./test-results/fake-mic-${name}.wav`, import.meta.url));
	write(wav);
	return {
		name,
		grep: new RegExp(`@${name}(?![\\w-])`),
		use: {
			launchOptions: {
				args: [
					'--use-fake-device-for-media-stream',
					'--use-fake-ui-for-media-stream',
					`--use-file-for-fake-audio-capture=${wav}`
				]
			}
		}
	};
}

export default defineConfig({
	testDir: 'e2e',
	webServer: external
		? undefined
		: { command: `pnpm build && pnpm preview --port ${port} --strictPort`, port },
	use: {
		baseURL: external ?? `http://localhost:${port}`,
		// The installed Chrome, so no browser download is needed.
		channel: 'chrome'
	},
	projects: [
		...Object.entries(TONES).map(([name, frequency]) =>
			fakeMic(name, (wav) => writeToneWav(wav, frequency))
		),
		fakeMic(PLUCK.name, (wav) => writePluckWav(wav, PLUCK.frequency, { period: PLUCK.period })),
		...Object.entries(STRUMS).map(([name, strum]) =>
			fakeMic(name, (wav) =>
				writeStrumWav(
					wav,
					strum.frequencies.map((f, i) => {
						const c = strum.cents[i];
						return c === null ? null : cents(f, c);
					})
				)
			)
		)
	]
});
