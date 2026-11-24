import { expect, test } from '@playwright/test';
import { begin, open, openTool } from './open.ts';

test('tuning ends by itself when every string is in tune @strum-uke-tuned', async ({ page }) => {
	await open(page, 'Strum');
	await expect(page.getByTestId('strum-status')).toContainText('All in tune', { timeout: 15_000 });
	// The sheet closes and the listening card invites a strum.
	await expect(page.getByTestId('tune-sheet')).toHaveCount(0, { timeout: 5_000 });
	await expect(page.getByTestId('chord-status')).toHaveText('Ready · strum a chord');
});

test('Play it in the lookup aims at the chord and lands it @chord-uke-c', async ({ page }) => {
	await open(page, 'Chords');
	await page.getByTestId('lookup-roots').getByRole('radio', { name: 'C', exact: true }).click();
	await page.getByTestId('lookup-types').getByRole('radio', { name: 'maj', exact: true }).click();
	await page.getByRole('button', { name: /Play it/ }).click();
	await expect(page.getByTestId('chords-sheet')).toHaveCount(0);
	const target = page.getByTestId('target');
	await expect(target).toContainText('From the lookup');
	await expect(target).toHaveAttribute('data-landed', 'true', { timeout: 15_000 });
	await expect(page.getByTestId('target-text')).toContainText('C ✓');
	// First C ever: marked new, and the lookup now knows it.
	await expect(page.getByTestId('chord-new')).toBeVisible();
	await openTool(page, 'Chords');
	await expect(page.getByTestId('lookup-played')).toBeVisible();
});

test('play along a progression: each chord lights up in turn @chord-uke-progression', async ({
	page
}) => {
	await open(page, 'Chord');
	await page.getByRole('button', { name: 'Practice' }).click();
	const sheet = page.getByTestId('practice-sheet');
	await sheet.getByLabel('Key').selectOption({ label: 'C' });
	await sheet.getByRole('button', { name: /I–vi–IV–V/ }).click();
	const steps = page.getByTestId('progression').locator('li');
	await expect(steps).toHaveText(['C', 'Am', 'F', 'G']);
	// The recording plays C Am F G: the current chord moves along with it.
	const seen = new Set<string>();
	for (let i = 0; i < 60 && seen.size < 3; i++) {
		seen.add((await page.locator('[data-testid="progression"] li.current').textContent())!);
		await page.waitForTimeout(200);
	}
	expect(seen.size).toBeGreaterThanOrEqual(3);
});

test('one-minute changes count clean switches @chord-uke-progression', async ({ page }) => {
	await open(page, 'Chord');
	await page.getByRole('button', { name: 'Practice' }).click();
	await page
		.getByTestId('drill-pairs')
		.getByRole('button', { name: /C ⇄ Am/ })
		.click();
	const target = page.getByTestId('target');
	await expect(target).toHaveAttribute('data-kind', 'drill');
	// C starts the clock, Am counts the first change.
	await expect(page.getByTestId('drill-count')).toHaveText(/[1-9]/, { timeout: 20_000 });
	await expect(target).toContainText('left');
});

test('practice shows the week, the chord of the day and my chords @chord-uke-c', async ({
	page
}) => {
	await open(page, 'Chord');
	await expect(page.getByTestId('chord-name')).toHaveText('C', { timeout: 15_000 });
	// Some playing time today: a second chord a little later.
	await page.waitForTimeout(3500);
	await page.getByRole('button', { name: 'Practice' }).click();
	const sheet = page.getByTestId('practice-sheet');
	await expect(sheet.getByTestId('daily-chord')).not.toBeEmpty();
	await expect(sheet.getByTestId('my-chords')).toContainText('C');
	await sheet.getByTestId('my-chords').getByRole('button', { name: 'C', exact: true }).click();
	await expect(page.getByTestId('chords-sheet')).toBeVisible();
	await expect(page.getByTestId('lookup-name')).toHaveText('C');
});

test('the last session is summed up on return @chord-uke-c', async ({ page }) => {
	await page.addInitScript(() => {
		const end = Date.now() - 2 * 60 * 60 * 1000;
		localStorage.setItem(
			'tuner.progress',
			JSON.stringify({
				version: 1,
				chords: { '0:': { symbol: 'C', first: '2026-11-20', count: 9 } },
				changes: { 'C → Am': 1.3 },
				drills: {},
				days: {},
				session: {
					start: end - 12 * 60 * 1000,
					end,
					seconds: 12 * 60,
					chords: 9,
					bestChange: { pair: 'C → Am', seconds: 1.3 }
				}
			})
		);
	});
	await begin(page);
	await expect(page.getByTestId('session-summary')).toContainText(
		'Last time: 12 min · 9 chords · best change C → Am 1.3 s'
	);
	await page.getByRole('button', { name: 'Dismiss' }).click();
	await expect(page.getByTestId('session-summary')).toHaveCount(0);
});
