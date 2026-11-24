import { expect, type Page } from '@playwright/test';

/** Chord: the listening screen; Strum and String: the Tune sheet; Chords: lookup. */
export type Mode = 'Chord' | 'Strum' | 'String' | 'Chords';

/**
 * Opens the app in `mode`. Listening starts on load; if the browser holds
 * audio back until a gesture, taps the "tap to start" overlay.
 */
export async function open(page: Page, mode: Mode, path = '/') {
	await page.goto(path);
	const wake = page.getByTestId('wake');
	await expect(page.getByTestId('level').or(wake)).toBeVisible({ timeout: 10_000 });
	if (await wake.isVisible()) await page.mouse.click(5, 5);
	await expect(page.getByTestId('level')).toBeVisible();
	await openTool(page, mode);
}

/** Opens the sheet for `mode` from the listening screen. */
export async function openTool(page: Page, mode: Mode) {
	if (mode === 'Chord') return;
	if (mode === 'Chords') {
		await page.getByRole('button', { name: 'Chords', exact: true }).click();
		await expect(page.getByTestId('chords-sheet')).toBeVisible();
		return;
	}
	await page.getByRole('button', { name: 'Tune', exact: true }).click();
	await expect(page.getByTestId('tune-sheet')).toBeVisible();
	await page.locator('label', { hasText: mode === 'Strum' ? 'Whole strum' : 'One string' }).click();
	await expect(page.getByTestId('level')).toBeVisible();
}
