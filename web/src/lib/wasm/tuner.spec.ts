import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { instantiateTuner } from './tuner';

describe('instantiateTuner', () => {
	it('loads the built module and calls an export', async () => {
		const bytes = readFileSync(new URL('../../../static/wasm/tuner.wasm', import.meta.url));
		const tuner = await instantiateTuner(bytes);
		expect(tuner.tuner_version()).toBe(1);
	});
});
