import { expect, test, type Page } from '@playwright/test';
import { STRUMS } from '../playwright.config.ts';

/** Expected verdict for a string offset (null: muted). */
function expected(cents: number | null) {
	if (cents === null) return 'not-heard';
	if (Math.abs(cents) <= 5) return 'in-tune';
	return cents < 0 ? 'flat' : 'sharp';
}

async function strumCheck(page: Page, instrument: 'Guitar' | 'Ukulele') {
	await page.goto('/');
	await page.getByRole('radio', { name: 'Strum' }).check({ force: true });
	await page.getByRole('radio', { name: instrument }).check({ force: true });
	await expect(page.getByTestId('strum-status')).toHaveText('Press Start and strum all strings');
	await page.getByRole('button', { name: 'Start' }).click();
}

for (const [name, strum] of Object.entries(STRUMS)) {
	test(`strum check shows each string's verdict @${name}`, async ({ page }) => {
		await strumCheck(page, strum.instrument);
		await expect(page.getByTestId('strum-status')).toContainText('Last strum', { timeout: 15_000 });
		for (const [i, string] of strum.strings.entries()) {
			const row = page.getByTestId(`strum-${string}`);
			await expect(row, string).toHaveAttribute('data-verdict', expected(strum.cents[i]));
			const c = strum.cents[i];
			if (c !== null) {
				const text = (await row.locator('.cents').textContent())!;
				const shown = parseFloat(text.replace('−', '-').replace('±', ''));
				expect(Math.abs(shown - c), `${string} cents`).toBeLessThanOrEqual(3);
			}
		}
	});
}

test('strum mode and instrument are remembered @strum-guitar', async ({ page }) => {
	await page.goto('/');
	await page.getByRole('radio', { name: 'Strum' }).check({ force: true });
	await page.getByRole('radio', { name: 'Ukulele' }).check({ force: true });
	await page.reload();
	await expect(page.getByRole('radio', { name: 'Strum' })).toBeChecked();
	await expect(page.getByTestId('strum-G4')).toBeVisible();
});
