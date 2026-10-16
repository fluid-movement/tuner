// Builds crates/tuner-wasm for wasm32 and copies the module to static/wasm/tuner.wasm.
import { execFileSync } from 'node:child_process';
import { copyFileSync, mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const web = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const root = resolve(web, '..');

const out = resolve(web, 'static/wasm/tuner.wasm');
mkdirSync(dirname(out), { recursive: true });

// The Docker build compiles the wasm in a Rust stage and passes it in here.
const prebuilt = process.env.TUNER_WASM_PREBUILT;
if (prebuilt) {
	copyFileSync(resolve(prebuilt), out);
	console.log(`wasm: ${out} (prebuilt from ${prebuilt})`);
	process.exit(0);
}

execFileSync(
	'cargo',
	['build', '-p', 'tuner-wasm', '--target', 'wasm32-unknown-unknown', '--release'],
	{ cwd: root, stdio: 'inherit' }
);

copyFileSync(resolve(root, 'target/wasm32-unknown-unknown/release/tuner_wasm.wasm'), out);
console.log(`wasm: ${out}`);
