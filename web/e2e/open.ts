import { expect, type Page } from '@playwright/test';

/** Chord: the listening screen; Strum: the Tune sheet; Chords: lookup. */
export type Mode = 'Chord' | 'Strum' | 'Chords';

/**
 * Opens the app in `mode`. Listening starts on load; if the browser holds
 * audio back until a gesture, taps the "tap to start" overlay.
 */
export async function open(page: Page, mode: Mode, path = '/') {
	await begin(page, path);
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
	await expect(page.getByTestId('level')).toBeVisible();
}

/** Loads the app and passes the first-run intro (fresh storage in each test). */
export async function begin(page: Page, path = '/') {
	await page.goto(path);
	// Chrome's fake microphone counts as already allowed, so the intro
	// usually doesn't show; pass it when it does.
	const intro = page.getByTestId('intro');
	const started = page.getByTestId('chord-name').or(page.getByRole('alert'));
	await expect(intro.or(started).first()).toBeVisible({ timeout: 10_000 });
	if (await intro.isVisible()) await page.getByRole('button', { name: 'Start listening' }).click();
}
