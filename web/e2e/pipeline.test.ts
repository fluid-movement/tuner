import { expect, test } from '@playwright/test';
import { begin } from './open.ts';

test('a first visit explains before asking for the microphone @chord-uke-c', async ({ page }) => {
	await page.addInitScript(() => {
		const w = window as unknown as { asked: number };
		w.asked = 0;
		// A browser that hasn't asked yet (Chrome's fake microphone is allowed).
		const query = navigator.permissions.query.bind(navigator.permissions);
		navigator.permissions.query = (d) =>
			d.name === ('microphone' as PermissionName)
				? Promise.resolve({ state: 'prompt' } as PermissionStatus)
				: query(d);
		const gum = navigator.mediaDevices.getUserMedia.bind(navigator.mediaDevices);
		navigator.mediaDevices.getUserMedia = (c) => {
			w.asked++;
			return gum(c);
		};
	});
	const asked = () => page.evaluate(() => (window as unknown as { asked: number }).asked);
	await page.goto('/');
	await expect(page.getByRole('heading', { name: 'Four Strings' })).toBeVisible();
	await expect(page.getByTestId('intro')).toBeVisible();
	await page.waitForTimeout(500);
	expect(await asked()).toBe(0);
	await page.getByRole('button', { name: 'Start listening' }).click();
	await expect(page.getByTestId('chord-name')).toHaveText('C', { timeout: 15_000 });
	expect(await asked()).toBe(1);
	await expect(page.getByRole('dialog')).toHaveCount(0);
	// Next time it listens straight away.
	await page.reload();
	await expect(page.getByTestId('intro')).toHaveCount(0);
	await expect(page.getByTestId('chord-name')).toHaveText('C', { timeout: 15_000 });
});

test('shows an error when microphone access is denied @chord-uke-c', async ({ page }) => {
	await page.addInitScript(() => {
		navigator.mediaDevices.getUserMedia = () =>
			Promise.reject(new DOMException('Permission denied', 'NotAllowedError'));
	});
	await begin(page);
	await expect(page.getByRole('alert')).toHaveText('Microphone access was denied.');
	await expect(page.getByTestId('denied-help')).toContainText('allow the microphone');
	await expect(page.getByRole('button', { name: 'Try again' })).toBeEnabled();
	// Lookup works without it.
	await page.getByRole('button', { name: 'Look up chords' }).click();
	await expect(page.getByTestId('chords-sheet')).toBeVisible();
});

test('where audio needs a gesture, the first tap anywhere starts it @chord-uke-c', async ({
	page
}) => {
	// Emulate iOS Safari: the context stays suspended until a user gesture.
	await page.addInitScript(() => {
		const Native = window.AudioContext;
		let gesture = false;
		addEventListener('pointerdown', () => (gesture = true), { capture: true });
		window.AudioContext = class extends Native {
			get state() {
				return gesture ? super.state : 'suspended';
			}
			resume() {
				if (!gesture) return new Promise<void>(() => {});
				return super.resume().then(() => {
					this.dispatchEvent(new Event('statechange'));
				});
			}
		};
	});
	await begin(page);
	const wake = page.getByTestId('wake');
	await expect(wake).toHaveText('Tap anywhere to start listening', { timeout: 10_000 });
	await expect(page.getByTestId('level')).toHaveCount(0);
	// The tap also reaches what it lands on.
	await page.getByRole('button', { name: 'Tune', exact: true }).click({ force: true });
	await expect(wake).toHaveCount(0);
	await expect(page.getByTestId('tune-sheet')).toBeVisible();
	await expect(page.getByTestId('level')).toBeVisible({ timeout: 10_000 });
});
