// Builds crates/tuner-wasm for wasm32 and copies the module to static/wasm/tuner.wasm.
import { execFileSync } from 'node:child_process';
import { copyFileSync, mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const web = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const root = resolve(web, '..');

execFileSync(
	'cargo',
	['build', '-p', 'tuner-wasm', '--target', 'wasm32-unknown-unknown', '--release'],
	{ cwd: root, stdio: 'inherit' }
);

const out = resolve(web, 'static/wasm/tuner.wasm');
mkdirSync(dirname(out), { recursive: true });
copyFileSync(resolve(root, 'target/wasm32-unknown-unknown/release/tuner_wasm.wasm'), out);
console.log(`wasm: ${out}`);
