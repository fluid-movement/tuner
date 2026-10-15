import { defineConfig } from '@playwright/test';
import { fileURLToPath } from 'node:url';
import { writeToneWav } from './e2e/wav.ts';

/** The fake microphone plays this tone: A2, the guitar's fifth string. */
export const FAKE_MIC_FREQUENCY = 110;
const fakeMic = fileURLToPath(new URL('./test-results/fake-mic.wav', import.meta.url));
writeToneWav(fakeMic, FAKE_MIC_FREQUENCY);

export default defineConfig({
	testDir: 'e2e',
	webServer: { command: 'pnpm build && pnpm preview --port 4173 --strictPort', port: 4173 },
	use: {
		baseURL: 'http://localhost:4173',
		// The installed Chrome, so no browser download is needed.
		channel: 'chrome',
		launchOptions: {
			args: [
				'--use-fake-device-for-media-stream',
				'--use-fake-ui-for-media-stream',
				`--use-file-for-fake-audio-capture=${fakeMic}`
			]
		}
	}
});
