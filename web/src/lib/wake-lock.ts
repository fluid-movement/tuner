/**
 * Keeps the screen on while `active()` holds (hands are on the instrument).
 * Browsers drop the lock when the page is hidden, so it is re-requested when
 * the page becomes visible again. Returns a function that releases it. Does
 * nothing where the Screen Wake Lock API is missing or refuses.
 */
export function keepScreenOn(): () => void {
	if (typeof navigator === 'undefined' || !('wakeLock' in navigator)) return () => {};
	let lock: WakeLockSentinel | null = null;
	let stopped = false;
	const request = async () => {
		if (stopped || document.visibilityState !== 'visible' || (lock && !lock.released)) return;
		try {
			lock = await navigator.wakeLock.request('screen');
			if (stopped) await lock.release();
		} catch {
			// Denied (battery saver, permissions policy): the screen may sleep.
		}
	};
	const onVisible = () => void request();
	document.addEventListener('visibilitychange', onVisible);
	void request();
	return () => {
		stopped = true;
		document.removeEventListener('visibilitychange', onVisible);
		void lock?.release().catch(() => {});
		lock = null;
	};
}
