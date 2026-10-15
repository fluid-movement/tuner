import { expect, test, type Page } from '@playwright/test';

async function listen(page: Page, instrument: 'Guitar' | 'Ukulele' = 'Guitar') {
	await page.goto('/');
	await page.getByRole('radio', { name: instrument }).check({ force: true });
	await page.getByRole('button', { name: 'Start' }).click();
}

test('an in-tune low E lights up E2 @e2', async ({ page }) => {
	await listen(page);
	await expect(page.getByTestId('string-E2')).toHaveAttribute('aria-current', 'true', {
		timeout: 10_000
	});
	await expect(page.getByTestId('status')).toHaveText('In tune');
	await expect(page.getByTestId('note')).toHaveText('E2');
});

test('a flat A2 shows flat and not in tune @a2-flat', async ({ page }) => {
	await listen(page);
	await expect(page.getByTestId('string-A2')).toHaveAttribute('aria-current', 'true', {
		timeout: 10_000
	});
	await expect(page.getByTestId('status')).toHaveText('Flat · tune up');
	// The display converges on −10 cents as readings are smoothed.
	await expect
		.poll(async () =>
			parseFloat((await page.getByTestId('cents').textContent())!.replace('−', '-'))
		)
		.toBeCloseTo(-10, 0);
});

test('a C4 on ukulele lights up its C string @c4', async ({ page }) => {
	await listen(page, 'Ukulele');
	await expect(page.getByTestId('string-C4')).toHaveAttribute('aria-current', 'true', {
		timeout: 10_000
	});
	await expect(page.getByTestId('status')).toHaveText('In tune');
});

test('the instrument choice survives a reload @a2', async ({ page }) => {
	await page.goto('/');
	const strings = page.getByRole('list', { name: 'Strings' }).getByRole('listitem');
	await expect(strings).toHaveText(['E2', 'A2', 'D3', 'G3', 'B3', 'E4']);
	await page.getByRole('radio', { name: 'Ukulele' }).check({ force: true });
	await expect(strings).toHaveText(['G4', 'C4', 'E4', 'A4']);
	await page.reload();
	await expect(page.getByRole('radio', { name: 'Ukulele' })).toBeChecked();
	await expect(strings).toHaveText(['G4', 'C4', 'E4', 'A4']);
});

test('fits a 375 px phone screen without horizontal scrolling @a2', async ({ page }) => {
	await page.setViewportSize({ width: 375, height: 740 });
	await listen(page);
	await expect(page.getByTestId('note')).toHaveText('A2', { timeout: 10_000 });
	const overflow = await page.evaluate(
		() => document.documentElement.scrollWidth - document.documentElement.clientWidth
	);
	expect(overflow).toBeLessThanOrEqual(0);
	await expect(page.getByTestId('status')).toHaveText('In tune');
	await page.waitForTimeout(500);
	await page.screenshot({ path: 'test-results/phone.png', fullPage: true });
});
