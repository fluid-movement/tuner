import { defineConfig } from '@playwright/test';
import { fileURLToPath } from 'node:url';
import { writeProgressionWav, writeStrumWav } from './e2e/wav.ts';

/** Test an already running deployment instead of a local preview build. */
const external = process.env.E2E_BASE_URL;
/** Preview port; set E2E_PORT to run several checkouts' tests in parallel. */
const port = Number(process.env.E2E_PORT ?? 4173);

const cents = (f: number, c: number) => f * 2 ** (c / 1200);
/** Strums with known detuning (cents from standard tuning, null = muted). */
export const STRUMS = {
	'strum-uke-tuned': {
		strings: ['G4', 'C4', 'E4', 'A4'],
		cents: [1, -1, 0, 2],
		frequencies: [391.99544, 261.62557, 329.62756, 440]
	},
	'strum-uke-muted': {
		strings: ['G4', 'C4', 'E4', 'A4'],
		cents: [-15, 4, 0, null],
		frequencies: [391.99544, 261.62557, 329.62756, 440]
	}
} as const;

const UKULELE = [391.99544, 261.62557, 329.62756, 440];
/** Ukulele chords strummed every 3 s (frets in string order G C E A). */
export const CHORDS = {
	'chord-uke-c': { name: 'C', frets: [0, 0, 0, 3] },
	'chord-uke-g7': { name: 'G7', frets: [0, 2, 1, 2] },
	'chord-uke-fadd9': { name: 'Fadd9', frets: [0, 0, 1, 0] },
	/** C with the C string 22 cents flat (see DETUNE). */
	'chord-uke-c-flat': { name: 'C', frets: [0, 0, 0, 3] }
} as const;

/** Per-string detuning (cents) of some chord takes. */
const DETUNE: Record<string, number[]> = { 'chord-uke-c-flat': [0, -22, 0, 0] };

/** C Am F G, one strum each 1.5 s, looping. */
export const PROGRESSION = {
	name: 'chord-uke-progression',
	frets: ['0003', '2000', '2010', '0232']
};

/**
 * Chrome's fake microphone plays a WAV file chosen at launch, so each
 * recording is a project; tests opt in with a tag such as `@chord-uke-c`.
 */
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
		),
		...Object.entries(CHORDS).map(([name, chord]) =>
			fakeMic(name, (wav) =>
				writeStrumWav(
					wav,
					chord.frets.map((f, i) => cents(UKULELE[i], 100 * f + (DETUNE[name]?.[i] ?? 0)))
				)
			)
		),
		fakeMic(PROGRESSION.name, (wav) =>
			writeProgressionWav(
				wav,
				PROGRESSION.frets.map((f) => [...f].map((x, i) => cents(UKULELE[i], 100 * Number(x))))
			)
		)
	]
});
