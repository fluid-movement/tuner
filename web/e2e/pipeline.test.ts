import { expect, test } from '@playwright/test';
import { TONES } from '../playwright.config.ts';

test('detects the fake microphone tone after pressing Start @a2', async ({ page }) => {
	await page.goto('/');
	// Nothing listens until the user asks.
	await expect(page.getByRole('button', { name: 'Start' })).toBeEnabled();
	await expect(page.getByTestId('note')).toHaveCount(0);

	await page.getByRole('button', { name: 'Start' }).click();
	await expect(page.getByTestId('note')).toHaveText('A2', { timeout: 10_000 });
	// The display converges as readings are smoothed.
	await expect
		.poll(async () => parseFloat((await page.getByTestId('frequency').textContent())!))
		.toBeCloseTo(TONES.a2, 0);

	await page.getByRole('button', { name: 'Stop' }).click();
	await expect(page.getByRole('button', { name: 'Start' })).toBeVisible();
	await expect(page.getByTestId('note')).toHaveCount(0);
});

test('shows an error when microphone access is denied @a2', async ({ page }) => {
	await page.addInitScript(() => {
		navigator.mediaDevices.getUserMedia = () =>
			Promise.reject(new DOMException('Permission denied', 'NotAllowedError'));
	});
	await page.goto('/');
	await page.getByRole('button', { name: 'Start' }).click();
	await expect(page.getByRole('alert')).toHaveText('Microphone access was denied.');
	await expect(page.getByRole('button', { name: 'Start' })).toBeEnabled();
});
