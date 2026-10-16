import { defineConfig } from '@playwright/test';
import { fileURLToPath } from 'node:url';
import { writeToneWav } from './e2e/wav.ts';

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

export default defineConfig({
	testDir: 'e2e',
	webServer: external
		? undefined
		: { command: 'pnpm build && pnpm preview --port 4173 --strictPort', port: 4173 },
	use: {
		baseURL: external ?? 'http://localhost:4173',
		// The installed Chrome, so no browser download is needed.
		channel: 'chrome'
	},
	projects: Object.entries(TONES).map(([name, frequency]) => {
		const wav = fileURLToPath(new URL(`./test-results/fake-mic-${name}.wav`, import.meta.url));
		writeToneWav(wav, frequency);
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
	})
});
