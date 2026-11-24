import { expect, test, type Page } from '@playwright/test';
import { CHORDS, PROGRESSION, STRUMS } from '../playwright.config.ts';
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
		// Two seconds on, the settled verdicts are still on screen (early
		// reports may differ; guitar settles 0.8 s after the strum).
		await expect(page.getByTestId('strum-status')).toContainText('Last strum 2 s ago');
		expect(await verdicts()).toEqual(strum.cents.map(expected));
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
		// The looping microphone strums the same chord again every three
		// seconds: repeats collapse into one history entry with a count.
		await expect(page.getByTestId('chord-history').locator('li')).toHaveCount(1);
		await expect(page.getByTestId('chord-history')).toContainText(`${chord.name}×2`, {
			timeout: 10_000
		});
	});
}

test('chord mode explains what a chord is @chord-uke-fadd9', async ({ page }) => {
	await open(page, 'Chord');
	await expect(page.getByTestId('chord-name')).toHaveText('Fadd9', { timeout: 15_000 });
	// Simple (the default) leaves the theory out; Learn shows it.
	await expect(page.getByTestId('chord-description')).toHaveCount(0);
	await page.getByTestId('learn-toggle').click();
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

test('chord mode finds the key and suggests next chords @chord-uke-progression', async ({
	page
}) => {
	await open(page, 'Chord');
	await page.getByTestId('learn-toggle').click();
	const history = page.getByTestId('chord-history').locator('li');
	// C Am F G: one full round, then the key and numerals show.
	await expect(history).toHaveCount(4, { timeout: 20_000 });
	await expect(page.getByTestId('chord-key')).toContainText('C major');
	// Repeat counts (×2) aren't checked: Chrome's fake microphone sometimes
	// clicks (a misaligned 10 ms buffer), which reads as a re-strum of the
	// ringing chord.
	const entries = (await history.allTextContents()).map((t) =>
		t.replace(/×\d+/, '').replace(/\s+/g, ' ').trim()
	);
	expect(entries).toEqual(expect.arrayContaining(['C I', 'Am vi', 'F IV', 'G V']));
	// After G (V) the next ideas start with C (I), with a diagram and why.
	const first = page.getByTestId('chord-next').locator('li').first();
	await expect(first).toContainText(/C\s*I/, { timeout: 5_000 });
	await expect(first.getByTestId('chord-next-reason')).toHaveText(
		'V → I · the dominant resolves home'
	);
	await expect(first.locator('svg')).toBeVisible();
	expect(PROGRESSION.frets).toHaveLength(4);
	// Tapping an idea opens it in the lookup.
	await first.getByRole('button').click();
	await expect(page.getByRole('radio', { name: 'Chords', exact: true })).toBeChecked();
	await expect(page.getByTestId('lookup-name')).toHaveText('C');
	await expect(page.getByTestId('lookup-tab')).toHaveText('A|-3--|\nE|-0--|\nC|-0--|\nG|-0--|');
});
