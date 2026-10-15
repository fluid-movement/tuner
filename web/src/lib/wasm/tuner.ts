/** Exports of crates/tuner-wasm. Keep in sync with its `extern "C"` functions. */
export interface TunerExports {
	memory: WebAssembly.Memory;
	tuner_version(): number;
}

/** Path of the module under static/, built by `pnpm build:wasm`. */
export const TUNER_WASM_PATH = 'wasm/tuner.wasm' as const;

/**
 * Instantiates the tuner module from its bytes. Takes bytes rather than a URL
 * so the same code works inside an AudioWorklet, which cannot fetch.
 */
export async function instantiateTuner(bytes: BufferSource): Promise<TunerExports> {
	const { instance } = await WebAssembly.instantiate(bytes, {});
	return instance.exports as unknown as TunerExports;
}
