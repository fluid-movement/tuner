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
