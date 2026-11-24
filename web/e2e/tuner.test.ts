import { expect, test, type Page } from '@playwright/test';
import { open } from './open.ts';

async function listen(page: Page) {
	await open(page, 'String');
}

test('an in-tune G lights up the G string @g4', async ({ page }) => {
	await listen(page);
	await expect(page.getByTestId('string-G4')).toHaveAttribute('aria-current', 'true', {
		timeout: 10_000
	});
	await expect(page.getByTestId('status')).toHaveText('In tune');
	await expect(page.getByTestId('note')).toHaveText('G4');
});

test('a flat A shows flat and not in tune @a4-flat', async ({ page }) => {
	await listen(page);
	await expect(page.getByTestId('string-A4')).toHaveAttribute('aria-current', 'true', {
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

test('a C4 lights up the C string @c4', async ({ page }) => {
	await listen(page);
	await expect(page.getByTestId('string-C4')).toHaveAttribute('aria-current', 'true', {
		timeout: 10_000
	});
	await expect(page.getByTestId('status')).toHaveText('In tune');
});

test('the tuner choice survives a reload @a4', async ({ page }) => {
	await open(page, 'String');
	const strings = page.getByRole('list', { name: 'Strings' }).getByRole('listitem');
	await expect(strings).toHaveText(['G4', 'C4', 'E4', 'A4']);
	await page.reload();
	// A reload starts on the listening screen; Tune remembers one string.
	await expect(page.getByTestId('tune-sheet')).toHaveCount(0);
	await page.getByRole('button', { name: 'Tune', exact: true }).click();
	await expect(page.getByRole('radio', { name: 'One string' })).toBeChecked();
	await expect(strings).toHaveText(['G4', 'C4', 'E4', 'A4']);
});

test('fits a 375 px phone screen without horizontal scrolling @a4', async ({ page }) => {
	await page.setViewportSize({ width: 375, height: 740 });
	await listen(page);
	await expect(page.getByTestId('note')).toHaveText('A4', { timeout: 10_000 });
	const overflow = await page.evaluate(
		() => document.documentElement.scrollWidth - document.documentElement.clientWidth
	);
	expect(overflow).toBeLessThanOrEqual(0);
	await expect(page.getByTestId('status')).toHaveText('In tune');
	await page.waitForTimeout(500);
	await page.screenshot({ path: 'test-results/phone.png', fullPage: true });
});

test('a quiet decaying pluck stays on screen through its decay @pluck', async ({ page }) => {
	await open(page, 'String', '/?debug');
	const note = page.getByTestId('note');
	await expect(note).toHaveText('A4', { timeout: 10_000 });
	await expect(page.getByTestId('level')).toBeVisible();
	await expect(page.getByTestId('debug')).toContainText('clarity');
	// Sample the display for 2.5 s: the note never disappears (tracked, then
	// held after the string fades) and always reads A4.
	for (let i = 0; i < 10; i++) {
		await expect(note).toHaveText('A4');
		await page.waitForTimeout(250);
	}
});
