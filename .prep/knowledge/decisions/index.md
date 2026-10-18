# decisions

## Entries

* [Stack: SvelteKit + Rust/WASM in an AudioWorklet, static PWA](/decisions/01M4ZPXYG12WVJMAHKVGWTN0ME.md) - Stack: SvelteKit + Rust/WASM in an AudioWorklet, static PWA
* [Package manager pnpm; SvelteKit 3 (latest)](/decisions/01M4ZPXYG3NRMK2REFTMF9D1BZ.md) - Package manager pnpm; SvelteKit 3 (latest)
* [Raw C-ABI wasm, no wasm-bindgen](/decisions/01M4ZPXYG418XMBMDGQ8ZEVG6K.md) - Raw C-ABI wasm, no wasm-bindgen
* [WASM bytes passed via processorOptions; Playwright on system Chrome with fake mic](/decisions/01M4ZPXYG50VM84XY0J2M5TGCX.md) - WASM bytes passed via processorOptions; Playwright on system Chrome with fake mic
* [McLeod Pitch Method (NSDF) for monophonic detection](/decisions/01M4ZPXYG65N49DMPQWGM1B9YJ.md) - McLeod Pitch Method (NSDF) for monophonic detection
* [Low-pass prefilter, least-squares peak fit, realistic noise criterion](/decisions/01M4ZPXYG7MDN8H9CP3K3XF69P.md) - Low-pass prefilter, least-squares peak fit, realistic noise criterion
* [Smoothing in the UI, tuning lookups from Rust on the main thread](/decisions/01M4ZPXYG8VMPQWBFV4R05K26X.md) - Smoothing in the UI, tuning lookups from Rust on the main thread
* [Build before real recordings; validate with them before done](/decisions/01M529ANG18ZRX4P1MZ2YYDCA7.md) - Build before real recordings; validate with them before done
* [Dockerfile build pack, nginx static server](/decisions/01M529ANG2RF9KPRPHF643F1D2.md) - Dockerfile build pack, nginx static server
* [Build images in GitHub Actions, Coolify only pulls](/decisions/01M529ANG3NGHKBESNW3G5J524.md) - Build images in GitHub Actions, Coolify only pulls
* [Strum presence by partial prominence; guitar B3/E4 can't be judged](/decisions/01M573Y44DQDZDGZ0EVQ55833P.md) - A string with clean partials is played when the second-largest prominence of its partials not explained by other strings is >= 20 dB; guitar B3/E4 report unknown
* [WASM size budget: 160 KB raw, 60 KB gzipped](/decisions/01M5743A24B8HDWVGCSDBY0F5Z.md) - The strum analysis makes tuner.wasm 147 KB stripped (55 KB gzipped) instead of the guessed 80 KB; accepted, further cuts need sort/fmt hand-tuning
