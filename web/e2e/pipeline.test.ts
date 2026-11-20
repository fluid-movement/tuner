import { expect, test } from '@playwright/test';
import { TONES } from '../playwright.config.ts';
import { open } from './open.ts';

test('listens as soon as the page opens, without a Start button @a2', async ({ page }) => {
	await open(page, 'String');
	await expect(page.getByRole('button', { name: 'Start' })).toHaveCount(0);
	await expect(page.getByTestId('note')).toHaveText('A2', { timeout: 10_000 });
	// The display converges as readings are smoothed.
	await expect
		.poll(async () => parseFloat((await page.getByTestId('frequency').textContent())!))
		.toBeCloseTo(TONES.a2, 0);
});

test('opens in chord mode on a first visit @a2', async ({ page }) => {
	await page.goto('/');
	await expect(page.getByRole('radio', { name: 'Chord', exact: true })).toBeChecked();
	await expect(page.getByTestId('chord-name')).toBeVisible();
});

test('shows an error when microphone access is denied @a2', async ({ page }) => {
	await page.addInitScript(() => {
		navigator.mediaDevices.getUserMedia = () =>
			Promise.reject(new DOMException('Permission denied', 'NotAllowedError'));
	});
	await page.goto('/');
	await expect(page.getByRole('alert')).toHaveText('Microphone access was denied.');
	await expect(page.getByRole('button', { name: 'Try again' })).toBeEnabled();
});

test('where audio needs a gesture, the first tap anywhere starts it @a2', async ({ page }) => {
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
	await page.goto('/');
	const wake = page.getByTestId('wake');
	await expect(wake).toHaveText('Tap anywhere to start listening', { timeout: 10_000 });
	await expect(page.getByTestId('level')).toHaveCount(0);
	// The tap also reaches what it lands on.
	await page.getByRole('radio', { name: 'String', exact: true }).click({ force: true });
	await expect(wake).toHaveCount(0);
	await expect(page.getByRole('radio', { name: 'String', exact: true })).toBeChecked();
	await expect(page.getByTestId('note')).toHaveText('A2', { timeout: 10_000 });
});
