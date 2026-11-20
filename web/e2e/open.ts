import { expect, type Page } from '@playwright/test';

export type Mode = 'Chord' | 'Strum' | 'String';

/**
 * Opens the app in `mode`. Listening starts on load; if the browser holds
 * audio back until a gesture, taps the "tap to start" overlay.
 */
export async function open(page: Page, mode: Mode, path = '/') {
	await page.goto(path);
	await page.getByRole('radio', { name: mode, exact: true }).check({ force: true });
	const wake = page.getByTestId('wake');
	await expect(page.getByTestId('level').or(wake)).toBeVisible({ timeout: 10_000 });
	if (await wake.isVisible()) await page.mouse.click(5, 5);
	await expect(page.getByTestId('level')).toBeVisible();
}
