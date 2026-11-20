// Offline support: precaches the app shell, the built JS/CSS (including the
// audio worklet and strum worker), the WASM and the icons. Each build has
// its own cache (`version`); the new worker installs in the background and
// takes over on the next launch (no skipWaiting: an open page keeps the
// files it started with), then drops older caches.

import { version } from '$app/env';
import { assets, immutable, prerendered } from '$app/manifest';
import { self as sw } from '$app/service-worker';

const CACHE = `tuner-${version}`;
/** The app's root page (the scope), the shell for every navigation. */
const ROOT = sw.registration.scope;
// Manifest paths are relative to the base path, which is the scope.
const ASSETS = [
	...new Set([
		ROOT,
		...[...immutable, ...assets, ...prerendered].map((a) => new URL(a.path, ROOT).href)
	])
];

sw.addEventListener('install', (event) => {
	event.waitUntil(caches.open(CACHE).then((cache) => cache.addAll(ASSETS)));
});

sw.addEventListener('activate', (event) => {
	event.waitUntil(
		caches
			.keys()
			.then((keys) => Promise.all(keys.filter((k) => k !== CACHE).map((k) => caches.delete(k))))
	);
});

sw.addEventListener('fetch', (event) => {
	const request = event.request;
	if (request.method !== 'GET') return;
	const url = new URL(request.url);
	if (url.origin !== sw.location.origin) return;

	event.respondWith(
		(async () => {
			const cache = await caches.open(CACHE);
			// Precached files never change within a version: cache first. Pages
			// match without their query (`/?debug`). Servers send `Vary: Origin`
			// (and nginx `Accept-Encoding`); module imports carry an Origin header
			// the precache requests didn't, so Vary must not cause a miss.
			const cached =
				request.mode === 'navigate'
					? await cache.match(ROOT, { ignoreVary: true })
					: await cache.match(request, { ignoreSearch: true, ignoreVary: true });
			if (cached) return cached;
			return fetch(request);
		})()
	);
});
