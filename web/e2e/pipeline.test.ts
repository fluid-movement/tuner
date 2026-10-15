import { expect, test } from '@playwright/test';
import { FAKE_MIC_FREQUENCY } from '../playwright.config.ts';

test('detects the fake microphone tone after pressing Start', async ({ page }) => {
	await page.goto('/');
	// Nothing listens until the user asks.
	await expect(page.getByRole('button', { name: 'Start' })).toBeVisible();
	await expect(page.getByTestId('reading')).toHaveCount(0);

	await page.getByRole('button', { name: 'Start' }).click();
	await expect(page.getByTestId('note')).toHaveText('A2', { timeout: 10_000 });
	const text = await page.getByTestId('frequency').textContent();
	expect(parseFloat(text!)).toBeCloseTo(FAKE_MIC_FREQUENCY, 0);

	await page.getByRole('button', { name: 'Stop' }).click();
	await expect(page.getByRole('button', { name: 'Start' })).toBeVisible();
});

test('shows an error when microphone access is denied', async ({ page }) => {
	await page.addInitScript(() => {
		navigator.mediaDevices.getUserMedia = () =>
			Promise.reject(new DOMException('Permission denied', 'NotAllowedError'));
	});
	await page.goto('/');
	await page.getByRole('button', { name: 'Start' }).click();
	await expect(page.getByRole('alert')).toHaveText('Microphone access was denied.');
	await expect(page.getByRole('button', { name: 'Start' })).toBeEnabled();
});
