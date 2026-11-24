// What the player has done, kept on the device: the chords they have
// played, their best chord changes and drills, and how much they played
// each day. No accounts; storage may be cleared by the browser.

export interface ChordRecord {
	/** Symbol as first played, e.g. "Am". */
	symbol: string;
	/** Local day of the first time, "2026-11-24". */
	first: string;
	count: number;
}

export interface Session {
	/** Start and last activity, ms since the epoch. */
	start: number;
	end: number;
	/** Time spent playing (gaps capped), seconds. */
	seconds: number;
	chords: number;
	/** Fastest change in the session, e.g. { pair: "Am → F", seconds: 1.2 }. */
	bestChange: { pair: string; seconds: number } | null;
}

export interface ProgressData {
	version: 1;
	/** Keyed by chordKey(). */
	chords: Record<string, ChordRecord>;
	/** Best change time per pair "Am → F", seconds. */
	changes: Record<string, number>;
	/** Best one-minute-changes count per pair "Am ⇄ F". */
	drills: Record<string, number>;
	/** Seconds played per local day. */
	days: Record<string, number>;
	/** The latest session (the current one while playing). */
	session: Session | null;
}

/** Playing stops counting after this long without a chord. */
const IDLE_SECONDS = 60;
/** A pause this long starts a new session. */
export const SESSION_GAP_MS = 30 * 60 * 1000;
/** Changes slower than this aren't changes, just playing on. */
export const CHANGE_MAX_SECONDS = 8;
/** Days of history kept. */
const KEEP_DAYS = 120;

export const empty = (): ProgressData => ({
	version: 1,
	chords: {},
	changes: {},
	drills: {},
	days: {},
	session: null
});

/** Local calendar day of `ms`, e.g. "2026-11-24". */
export function dayOf(ms: number): string {
	const d = new Date(ms);
	const pad = (n: number) => String(n).padStart(2, '0');
	return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;
}

/** Parses stored progress; anything unreadable starts afresh. */
export function parse(text: string | null): ProgressData {
	try {
		const data = JSON.parse(text ?? 'null');
		if (data?.version !== 1) return empty();
		return { ...empty(), ...data };
	} catch {
		return empty();
	}
}

export interface Played {
	/** Key for the collection (see chordKey). */
	key: string;
	symbol: string;
	/** When it was played, ms since the epoch. */
	at: number;
	/** Onset in the audio stream, seconds (for change times). */
	onset: number;
}

export interface Change {
	pair: string;
	seconds: number;
	/** A new best for this pair. */
	best: boolean;
}

export interface RecordResult {
	data: ProgressData;
	/** First time this chord was played. */
	isNew: boolean;
	/** The change from the previous chord, if it was a quick change. */
	change: Change | null;
}

/**
 * Records a settled chord. `previous` is the last settled chord of this
 * listening stretch (null after a pause), for the change time.
 */
export function record(data: ProgressData, played: Played, previous: Played | null): RecordResult {
	const next: ProgressData = {
		...data,
		chords: { ...data.chords },
		changes: { ...data.changes },
		days: { ...data.days }
	};
	const known = next.chords[played.key];
	const isNew = !known;
	next.chords[played.key] = known
		? { ...known, count: known.count + 1 }
		: { symbol: played.symbol, first: dayOf(played.at), count: 1 };

	let change: Change | null = null;
	if (previous && previous.key !== played.key) {
		const seconds = played.onset - previous.onset;
		if (seconds > 0 && seconds <= CHANGE_MAX_SECONDS) {
			const pair = `${previous.symbol} → ${played.symbol}`;
			const best = next.changes[pair] === undefined || seconds < next.changes[pair];
			if (best) next.changes[pair] = seconds;
			change = { pair, seconds, best };
		}
	}

	// Session and daily time.
	const s = data.session;
	const fresh = !s || played.at - s.end > SESSION_GAP_MS;
	const gap = fresh ? 0 : Math.min(IDLE_SECONDS, Math.max(0, (played.at - s!.end) / 1000));
	const session: Session = fresh
		? { start: played.at, end: played.at, seconds: 0, chords: 1, bestChange: null }
		: { ...s!, end: played.at, seconds: s!.seconds + gap, chords: s!.chords + 1 };
	if (change && (!session.bestChange || change.seconds < session.bestChange.seconds)) {
		session.bestChange = { pair: change.pair, seconds: change.seconds };
	}
	next.session = session;
	const day = dayOf(played.at);
	next.days[day] = (next.days[day] ?? 0) + gap;
	for (const d of Object.keys(next.days)) {
		if (played.at - new Date(d).getTime() > KEEP_DAYS * 86_400_000) delete next.days[d];
	}
	return { data: next, isNew, change };
}

/** Records a finished one-minute-changes drill; says whether it is a best. */
export function recordDrill(
	data: ProgressData,
	pair: string,
	count: number
): { data: ProgressData; best: boolean } {
	const best = count > 0 && count > (data.drills[pair] ?? 0);
	return {
		data: best ? { ...data, drills: { ...data.drills, [pair]: count } } : data,
		best
	};
}

export interface WeekDay {
	day: string;
	/** "M", "T" … */
	label: string;
	seconds: number;
	today: boolean;
}

/** Playing time in words: "under a minute", "1 min", "38 min". */
export function minutesText(seconds: number): string {
	return seconds < 30 ? 'under a minute' : `${Math.round(seconds / 60)} min`;
}

/** The last seven days, oldest first. */
export function week(data: ProgressData, now: number): WeekDay[] {
	const labels = ['S', 'M', 'T', 'W', 'T', 'F', 'S'];
	return Array.from({ length: 7 }, (_, i) => {
		const ms = now - (6 - i) * 86_400_000;
		const day = dayOf(ms);
		return {
			day,
			label: labels[new Date(ms).getDay()],
			seconds: data.days[day] ?? 0,
			today: i === 6
		};
	});
}

/**
 * The previous session, worth summing up when the app opens again: it
 * ended a while ago and had some playing in it.
 */
export function lastSession(data: ProgressData, now: number): Session | null {
	const s = data.session;
	return s && now - s.end > SESSION_GAP_MS && s.chords >= 3 ? s : null;
}

/** Pairs worth drilling: the slowest best changes first. */
export function slowPairs(data: ProgressData, count = 3): string[] {
	return Object.entries(data.changes)
		.sort((a, b) => b[1] - a[1])
		.slice(0, count)
		.map(([pair]) => pair);
}
