import { expect, test } from '@playwright/test';
import { open } from './open.ts';

test('Chrome finds the app installable @a4', async ({ page }) => {
	await page.goto('/');
	const href = await page.locator('link[rel="manifest"]').getAttribute('href');
	const manifest = await (await page.request.get(new URL(href!, page.url()).href)).json();
	expect(manifest).toMatchObject({
		name: 'Four Strings',
		short_name: 'Four Strings',
		display: 'standalone'
	});
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

test('keeps the screen on while listening @a4', async ({ page }) => {
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

test('uses the microphone only while focused and keeps the chord @chord-uke-c', async ({
	page
}) => {
	await page.addInitScript(() => {
		const w = window as unknown as { tracks: MediaStreamTrack[]; focus: boolean };
		w.tracks = [];
		w.focus = true;
		document.hasFocus = () => w.focus;
		const gum = navigator.mediaDevices.getUserMedia.bind(navigator.mediaDevices);
		navigator.mediaDevices.getUserMedia = async (c) => {
			const stream = await gum(c);
			w.tracks.push(...stream.getTracks());
			return stream;
		};
	});
	const live = () =>
		page.evaluate(
			() =>
				(window as unknown as { tracks: MediaStreamTrack[] }).tracks.filter(
					(t) => t.readyState === 'live'
				).length
		);
	const setFocus = (focus: boolean) =>
		page.evaluate((focus) => {
			(window as unknown as { focus: boolean }).focus = focus;
			window.dispatchEvent(new Event(focus ? 'focus' : 'blur'));
		}, focus);

	await open(page, 'Chord');
	await expect(page.getByTestId('chord-name')).toHaveText('C', { timeout: 15_000 });
	expect(await live()).toBe(1);

	await setFocus(false);
	await expect(page.getByTestId('paused')).toBeVisible();
	await expect.poll(live).toBe(0);
	await expect(page.getByTestId('level')).toHaveCount(0);
	// The chord stays on screen while paused.
	await expect(page.getByTestId('chord-name')).toHaveText('C');

	await setFocus(true);
	await expect(page.getByTestId('paused')).toHaveCount(0);
	await expect(page.getByTestId('level')).toBeVisible({ timeout: 10_000 });
	expect(await live()).toBe(1);
	await expect(page.getByTestId('chord-name')).toHaveText('C');
});

test('chord mode keeps the chord on screen between strums @chord-uke-c', async ({ page }) => {
	await open(page, 'Chord');
	await expect(page.getByTestId('chord-name')).toHaveText('C', { timeout: 15_000 });
	// Sample the name through the next strum (every three seconds): never blank.
	const names = new Set<string>();
	for (let i = 0; i < 40; i++) {
		names.add((await page.getByTestId('chord-name').textContent())!.trim());
		await page.waitForTimeout(100);
	}
	expect([...names]).toEqual(['C']);
});
