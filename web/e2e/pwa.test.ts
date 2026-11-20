import { expect, test } from '@playwright/test';
import { open } from './open.ts';

test('Chrome finds the app installable @a2', async ({ page }) => {
	await page.goto('/');
	const href = await page.locator('link[rel="manifest"]').getAttribute('href');
	const manifest = await (await page.request.get(new URL(href!, page.url()).href)).json();
	expect(manifest).toMatchObject({ name: 'Tuner', short_name: 'Tuner', display: 'standalone' });
	const purposes = new Set<string>();
	for (const icon of manifest.icons) {
		const response = await page.request.get(new URL(icon.src, page.url()).href);
		expect(response.ok(), icon.src).toBe(true);
		purposes.add(`${icon.sizes} ${icon.purpose}`);
	}
	expect([...purposes]).toEqual(
		expect.arrayContaining(['192x192 any', '512x512 any', '512x512 maskable'])
	);
	// Chrome's own verdict, as used for the install prompt.
	await page.evaluate(() => navigator.serviceWorker.ready);
	const cdp = await page.context().newCDPSession(page);
	const { installabilityErrors } = await cdp.send('Page.getInstallabilityErrors');
	// Playwright contexts are incognito, where Chrome never offers to install.
	expect(installabilityErrors.filter((e) => e.errorId !== 'in-incognito')).toEqual([]);
});

test('works offline once loaded @chord-uke-c', async ({ page, context }) => {
	await page.goto('/');
	await page.evaluate(() => navigator.serviceWorker.ready);
	await page.reload();
	await expect.poll(() => page.evaluate(() => !!navigator.serviceWorker.controller)).toBe(true);
	await context.setOffline(true);
	const errors: string[] = [];
	page.on('pageerror', (e) => errors.push(e.message));
	await open(page, 'Chord');
	await expect(page.getByTestId('chord-name')).toHaveText('C', { timeout: 15_000 });
	expect(errors).toEqual([]);
});

test('keeps the screen on while listening @a2', async ({ page }) => {
	await page.addInitScript(() => {
		const w = window as unknown as { wakeLocks: number };
		w.wakeLocks = 0;
		Object.defineProperty(navigator, 'wakeLock', {
			value: {
				request: async () => {
					w.wakeLocks++;
					return { released: false, release: async () => {} };
				}
			}
		});
	});
	await open(page, 'String');
	await expect
		.poll(() => page.evaluate(() => (window as unknown as { wakeLocks: number }).wakeLocks))
		.toBeGreaterThan(0);
});
