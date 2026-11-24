import { expect, test } from '@playwright/test';

test('chord lookup shows shapes and theory without the microphone @a2', async ({ page }) => {
	await page.addInitScript(() => {
		const w = window as unknown as { tracks: MediaStreamTrack[] };
		w.tracks = [];
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
	await page.goto('/');
	await expect(page.getByTestId('level')).toBeVisible({ timeout: 10_000 });
	await page.getByRole('radio', { name: 'Chords', exact: true }).check({ force: true });
	// Lookup doesn't listen: the microphone is released, with no pause notice.
	await expect.poll(live).toBe(0);
	await expect(page.getByTestId('paused')).toHaveCount(0);

	await page.getByTestId('lookup-roots').getByRole('radio', { name: 'G', exact: true }).click();
	await page.getByTestId('lookup-types').getByRole('radio', { name: '7', exact: true }).click();
	await expect(page.getByTestId('lookup-name')).toHaveText('G7');
	await expect(page.getByTestId('lookup-description')).toHaveText('dominant 7th');
	await expect(page.getByTestId('lookup-tab')).toHaveText('A|-2--|\nE|-1--|\nC|-2--|\nG|-0--|');
	await expect(page.getByTestId('lookup-notes')).toContainText('G R');
	await expect(page.getByTestId('lookup-notes')).toContainText('F b7');
	await expect(page.getByTestId('lookup-keys')).toContainText('C major (V7)');

	// Another shape up the neck becomes the main one.
	const shapes = page.getByTestId('lookup-shapes').getByRole('button');
	expect(await shapes.count()).toBeGreaterThan(1);
	await shapes.nth(1).click();
	await expect(page.getByTestId('lookup-tab')).not.toHaveText('A|-2--|\nE|-1--|\nC|-2--|\nG|-0--|');
	await expect(page.getByTestId('lookup-name')).toHaveText('G7');

	// The mode and the chord are remembered.
	await page.reload();
	await expect(page.getByTestId('lookup-name')).toHaveText('G7');

	// Back to chord mode: listening again.
	await page.getByRole('radio', { name: 'Chord', exact: true }).check({ force: true });
	await expect(page.getByTestId('level')).toBeVisible({ timeout: 10_000 });
	await expect.poll(live).toBe(1);
});
