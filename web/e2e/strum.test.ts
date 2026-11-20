import { expect, test, type Page } from '@playwright/test';
import { CHORDS, STRUMS } from '../playwright.config.ts';
import { open } from './open.ts';

/** Expected verdict for a string offset (null: muted). */
function expected(cents: number | null) {
	if (cents === null) return 'not-heard';
	if (Math.abs(cents) <= 5) return 'in-tune';
	return cents < 0 ? 'flat' : 'sharp';
}

async function strumCheck(page: Page, instrument: 'Guitar' | 'Ukulele') {
	await open(page, 'Strum');
	await page.getByRole('radio', { name: instrument }).check({ force: true });
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
		const verdicts = () =>
			page
				.locator('[data-testid^="strum-"][data-verdict]')
				.evaluateAll((rows) => rows.map((r) => r.getAttribute('data-verdict')));
		// Settled by now (early reports may still show strings as pending).
		await expect(page.getByTestId('strum-status')).toContainText('Last strum 1 s ago');
		const held = await verdicts();
		await expect(page.getByTestId('strum-status')).toContainText('Last strum 2 s ago');
		expect(await verdicts()).toEqual(held);
		// The looping microphone strums again every three seconds.
		await expect(page.getByTestId('strum-status')).toContainText('Last strum just now');
	});
}

test('strum mode and instrument are remembered @strum-guitar', async ({ page }) => {
	await page.goto('/');
	await page.getByRole('radio', { name: 'Strum', exact: true }).check({ force: true });
	await page.getByRole('radio', { name: 'Ukulele' }).check({ force: true });
	await page.reload();
	await expect(page.getByRole('radio', { name: 'Strum', exact: true })).toBeChecked();
	await expect(page.getByTestId('strum-G4')).toBeVisible();
});

for (const [name, chord] of Object.entries(CHORDS).filter(([n]) => n !== 'chord-uke-c-flat')) {
	test(`chord mode names a strummed ${chord.name} and shows its tab @${name}`, async ({ page }) => {
		await open(page, 'Chord');
		await expect(page.getByTestId('chord-name')).toHaveText(chord.name, { timeout: 15_000 });
		// Highest string on top: A E C G.
		const tab = [...chord.frets]
			.map((f, i) => `${'GCEA'[i]}|-${String(f).padEnd(2, '-')}-|`)
			.reverse()
			.join('\n');
		await expect(page.getByTestId('chord-tab')).toHaveText(tab);
		// The looping microphone strums again every three seconds.
		await expect(page.getByTestId('chord-history').locator('li')).toHaveCount(2, {
			timeout: 10_000
		});
		await expect(page.getByTestId('chord-history')).toContainText(chord.name);
	});
}

test('chord mode explains what a chord is @chord-uke-fadd9', async ({ page }) => {
	await open(page, 'Chord');
	await expect(page.getByTestId('chord-name')).toHaveText('Fadd9', { timeout: 15_000 });
	await expect(page.getByTestId('chord-description')).toHaveText('major triad, added 9th');
	await expect(page.getByTestId('chord-alternatives')).toContainText('also');
	await expect(page.getByTestId('chord-keys')).toHaveText(
		'In F major (Iadd9) · Bb major (Vadd9) · C major (IVadd9)'
	);
	// Each string's note and role under the diagram.
	await expect(page.getByTestId('chord-diagram')).toContainText('G9C5FRA3');
});

test('chord mode points out a detuned string @chord-uke-c-flat', async ({ page }) => {
	await open(page, 'Chord');
	await expect(page.getByTestId('chord-name')).toHaveText('C', { timeout: 15_000 });
	await expect(page.getByTestId('chord-intonation')).toContainText('C string ~22 c flat');
	await page.getByTestId('chord-intonation').getByRole('button', { name: 'Tune' }).click();
	await expect(page.getByRole('radio', { name: 'Strum', exact: true })).toBeChecked();
	await expect(page.getByTestId('strum-C4')).toBeVisible();
});
