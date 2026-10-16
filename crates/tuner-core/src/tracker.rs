//! Turns per-frame pitch estimates into a steady reading for a tuner display.
//!
//! A decaying string gets quieter and less periodic within a second, so a
//! detector with fixed thresholds shows the pluck only briefly. The tracker
//! instead:
//!
//! - gates relative to an adaptive noise floor instead of an absolute level;
//! - locks onto a note only on a confident reading clearly above the floor,
//!   then *follows* it with much weaker readings as long as they stay near the
//!   tracked pitch (octave slips are folded back);
//! - retargets quickly when a different note is played confidently;
//! - holds the last reading for a while after the string fades.
//!
//! Feed it every frame's estimate from a permissive detector
//! ([`PitchConfig::permissive`](crate::PitchConfig::permissive)). It never
//! allocates, so it can run on the audio thread.

use crate::PitchEstimate;

/// Settings for a [`NoteTracker`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrackerConfig {
    /// Time between successive frames in seconds (hop / sample rate).
    pub frame_seconds: f32,
    /// Clarity needed to lock onto a new note.
    pub onset_clarity: f32,
    /// Level above the noise floor (dB) needed to lock onto a new note.
    pub onset_margin_db: f32,
    /// Clarity needed for a frame to keep following the tracked note.
    pub keep_clarity: f32,
    /// Level above the noise floor (dB) needed to keep following.
    pub keep_margin_db: f32,
    /// A following frame must be within this many cents of the tracked pitch.
    pub keep_cents: f32,
    /// Consecutive confident frames needed to lock onto a note (a new one, or
    /// another one while tracking).
    pub retarget_frames: u32,
    /// An onset also needs an attack: the level must have risen by this many
    /// dB above its minimum over the last `attack_seconds`. Without it, the
    /// periodicity of two strings ringing together (B3 + E4 repeat at E2's
    /// period) would look like a new note as they decay.
    pub attack_db: f32,
    pub attack_seconds: f32,
    /// Seconds without a usable frame before the note is released.
    pub release_seconds: f32,
    /// Seconds a released reading stays on display.
    pub hold_seconds: f32,
    /// A note without an onset (already sounding, bowed, a reference tone)
    /// locks after `steady_seconds` of frames at this clarity or more...
    pub steady_clarity: f32,
    /// ...lasting this long at a stable pitch...
    pub steady_seconds: f32,
    /// ...and at least this level (dBFS).
    pub steady_min_db: f32,
    /// Time constant (s) with which the noise floor rises on pitched frames
    /// while no note is tracked (unpitched frames rise faster). It falls
    /// quickly whenever the level drops below it.
    pub floor_rise_seconds: f32,
    /// Number of recent frames whose median is smoothed (1..=MAX_MEDIAN).
    pub median: usize,
    /// Time constant (s) of the exponential smoothing after the median.
    pub smoothing_seconds: f32,
}

/// Largest supported [`TrackerConfig::median`] window.
pub const MAX_MEDIAN: usize = 9;
/// Frames of level history kept for attack detection (bounds
/// `attack_seconds / frame_seconds`).
const MAX_ATTACK_FRAMES: usize = 16;

impl TrackerConfig {
    /// Defaults for frames every `frame_seconds` (about 21 ms in the app).
    pub fn new(frame_seconds: f32) -> Self {
        Self {
            frame_seconds,
            onset_clarity: 0.85,
            onset_margin_db: 12.0,
            keep_clarity: 0.6,
            keep_margin_db: 10.0,
            keep_cents: 35.0,
            retarget_frames: 2,
            attack_db: 6.0,
            attack_seconds: 0.1,
            release_seconds: 0.25,
            hold_seconds: 3.0,
            steady_clarity: 0.95,
            steady_seconds: 0.15,
            steady_min_db: -60.0,
            floor_rise_seconds: 4.0,
            median: 5,
            smoothing_seconds: 0.12,
        }
    }
}

/// What the display should show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TrackState {
    /// Nothing to show.
    Idle = 0,
    /// Following a ringing note.
    Tracking = 1,
    /// The note has faded; showing its last reading.
    Holding = 2,
}

/// One frame's result.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrackerOutput {
    pub state: TrackState,
    /// True on the frame where a note was locked onto (new or retargeted).
    pub locked: bool,
    /// Smoothed frequency of the tracked or held note (Hz).
    pub frequency: Option<f32>,
    /// This frame's raw detector frequency, if any.
    pub raw_frequency: Option<f32>,
    /// This frame's clarity (0 when the detector found nothing).
    pub clarity: f32,
    /// This frame's level in dBFS.
    pub level_db: f32,
    /// Current noise floor estimate in dBFS.
    pub floor_db: f32,
}

#[derive(Debug, Clone)]
pub struct NoteTracker {
    config: TrackerConfig,
    state: TrackState,
    floor_db: Option<f32>,
    /// Smoothed pitch in cents relative to A4.
    smoothed: f32,
    history: [f32; MAX_MEDIAN],
    history_len: usize,
    history_next: usize,
    misses: u32,
    held_seconds: f32,
    /// Candidate for retargeting: (cents, consecutive frames).
    candidate: Option<(f32, u32)>,
    /// Steady-pitch candidate without an onset: (cents, consecutive frames).
    steady: Option<(f32, u32)>,
    /// Levels (dB) of the most recent frames, for attack detection.
    recent_levels: [f32; MAX_ATTACK_FRAMES],
    recent_next: usize,
    just_locked: bool,
}

const SILENCE_DB: f32 = -120.0;
/// When the very first frame is already a confident note, the floor starts
/// this far below it (so that note still counts as an onset).
const ASSUMED_SNR_DB: f32 = 30.0;
/// Rise time constant of the floor on unpitched frames while no note is
/// tracked.
const NOISE_RISE_SECONDS: f32 = 0.5;

fn to_cents(hz: f32) -> f32 {
    1200.0 * (hz / 440.0).log2()
}

fn to_hz(cents: f32) -> f32 {
    440.0 * 2f32.powf(cents / 1200.0)
}

impl NoteTracker {
    pub fn new(config: TrackerConfig) -> Self {
        assert!(
            (1..=MAX_MEDIAN).contains(&config.median),
            "median window out of range"
        );
        Self {
            config,
            state: TrackState::Idle,
            floor_db: None,
            smoothed: 0.0,
            history: [0.0; MAX_MEDIAN],
            history_len: 0,
            history_next: 0,
            misses: 0,
            held_seconds: 0.0,
            candidate: None,
            steady: None,
            recent_levels: [SILENCE_DB; MAX_ATTACK_FRAMES],
            recent_next: 0,
            just_locked: false,
        }
    }

    pub fn config(&self) -> &TrackerConfig {
        &self.config
    }

    pub fn state(&self) -> TrackState {
        self.state
    }

    /// Forgets everything, including the noise floor.
    pub fn reset(&mut self) {
        *self = Self::new(self.config);
    }

    /// Processes one frame: the detector's estimate (if it found a pitch) and
    /// the frame's RMS level ([`PitchDetector::level`](crate::PitchDetector::level)).
    pub fn push(&mut self, estimate: Option<PitchEstimate>, level: f32) -> TrackerOutput {
        let c = self.config;
        self.just_locked = false;
        let level_db = 20.0 * level.max(1e-6).log10().max(SILENCE_DB / 20.0);
        let clarity = estimate.map_or(0.0, |e| e.clarity);
        // The first frame sets the floor if it sounds like noise; if it is
        // already a confident note, start low so that note still counts.
        let floor = *self.floor_db.get_or_insert(if clarity >= c.onset_clarity {
            level_db - ASSUMED_SNR_DB
        } else {
            level_db
        });
        let attack_frames =
            ((c.attack_seconds / c.frame_seconds).round() as usize).clamp(1, MAX_ATTACK_FRAMES);
        let recent_min = (1..=attack_frames)
            .map(|k| {
                self.recent_levels[(self.recent_next + MAX_ATTACK_FRAMES - k) % MAX_ATTACK_FRAMES]
            })
            .fold(f32::MAX, f32::min);
        let attack = level_db - recent_min >= c.attack_db;
        self.recent_levels[self.recent_next] = level_db;
        self.recent_next = (self.recent_next + 1) % MAX_ATTACK_FRAMES;
        let raw = estimate.map(|e| to_cents(e.frequency));

        // Onset quality. The attack is required on the first frame of a
        // candidate; the confirming frame only needs to agree in pitch.
        let candidate = self.candidate;
        let onset = |cents: Option<f32>| {
            cents.filter(|&x| {
                let confirms = candidate.is_some_and(|(prev, _)| (x - prev).abs() <= c.keep_cents);
                clarity >= c.onset_clarity
                    && level_db >= floor + c.onset_margin_db
                    && (attack || confirms)
            })
        };

        match self.state {
            TrackState::Idle | TrackState::Holding => {
                // A new note must be confirmed by consecutive confident frames;
                // the first one usually straddles the attack.
                if let Some(x) = onset(raw) {
                    if self.confirm(x) {
                        self.lock(x, level_db);
                    }
                } else {
                    self.candidate = None;
                }
                // No onset, but a clean, stable pitch that keeps sounding. Not
                // while holding: that is the tail of the note just released,
                // and replaying it brings a real onset.
                let steady =
                    raw.filter(|_| clarity >= c.steady_clarity && level_db >= c.steady_min_db);
                if self.state == TrackState::Idle {
                    self.steady = steady.map(|x| match self.steady {
                        Some((prev, n)) if (x - prev).abs() <= c.keep_cents => (x, n + 1),
                        _ => (x, 1),
                    });
                    if let Some((x, n)) = self.steady
                        && n as f32 * c.frame_seconds >= c.steady_seconds
                    {
                        self.lock(x, level_db);
                    }
                }
                if self.state == TrackState::Holding {
                    self.held_seconds += c.frame_seconds;
                    if self.held_seconds >= c.hold_seconds {
                        self.state = TrackState::Idle;
                    }
                }
            }
            TrackState::Tracking => {
                let folded = raw.map(|x| self.fold_octave(x));
                let keep = folded.filter(|&x| {
                    clarity >= c.keep_clarity
                        && level_db >= floor + c.keep_margin_db
                        && (x - self.smoothed).abs() <= c.keep_cents
                });
                if let Some(x) = keep {
                    self.candidate = None;
                    self.misses = 0;
                    self.follow(x);
                } else if let Some(x) = onset(raw) {
                    // Confident and clearly elsewhere: another string?
                    if self.confirm(x) {
                        self.lock(x, level_db);
                    } else {
                        self.miss();
                    }
                } else {
                    self.candidate = None;
                    self.miss();
                }
            }
        }

        // The floor drops quickly to quieter levels; it only rises (slowly)
        // while no note is ringing, so a sustained note doesn't raise it.
        let floor = self.floor_db.as_mut().expect("initialised above");
        if level_db < *floor {
            *floor += 0.5 * (level_db - *floor);
        } else if self.state != TrackState::Tracking {
            // Unpitched sound is background: adapt quickly to a noisier room.
            let tau = if clarity < c.keep_clarity {
                NOISE_RISE_SECONDS
            } else {
                c.floor_rise_seconds
            };
            *floor += (1.0 - (-c.frame_seconds / tau).exp()) * (level_db - *floor);
        }

        TrackerOutput {
            state: self.state,
            locked: self.just_locked,
            frequency: (self.state != TrackState::Idle).then(|| to_hz(self.smoothed)),
            raw_frequency: estimate.map(|e| e.frequency),
            clarity,
            level_db,
            floor_db: *floor,
        }
    }

    /// Counts consecutive confident frames near `x`; true once there are
    /// `retarget_frames` of them.
    fn confirm(&mut self, x: f32) -> bool {
        let count = match self.candidate {
            Some((prev, n)) if (x - prev).abs() <= self.config.keep_cents => n + 1,
            _ => 1,
        };
        self.candidate = Some((x, count));
        count >= self.config.retarget_frames
    }

    /// Moves `x` by one or two octaves if that puts it within the follow range
    /// of the tracked pitch (decaying strings often slip an octave; two strings
    /// ringing together can read two octaves low).
    fn fold_octave(&self, x: f32) -> f32 {
        let diff = x - self.smoothed;
        for shift in [1200.0, -1200.0, 2400.0, -2400.0] {
            if (diff - shift).abs() <= self.config.keep_cents {
                return x - shift;
            }
        }
        x
    }

    /// Replaces the reading of a note locked on this frame, e.g. with a
    /// re-analysis of the same frame in a narrower band. Ignored unless the
    /// tracker has just locked and `frequency` is within the follow range.
    pub fn reseed(&mut self, frequency: f32) -> Option<f32> {
        let cents = to_cents(frequency);
        if self.state != TrackState::Tracking
            || self.history_len != 1
            || (cents - self.smoothed).abs() > self.config.keep_cents
        {
            return None;
        }
        self.history_len = 0;
        self.history_next = 0;
        self.follow(cents);
        Some(to_hz(self.smoothed))
    }

    fn lock(&mut self, cents: f32, level_db: f32) {
        // A note this loud means the background is at least the onset margin
        // below it (the floor may have learned a note that was already
        // sounding when listening started).
        if let Some(floor) = self.floor_db.as_mut() {
            *floor = floor.min(level_db - self.config.onset_margin_db);
        }
        self.just_locked = true;
        self.state = TrackState::Tracking;
        self.history_len = 0;
        self.history_next = 0;
        self.smoothed = cents;
        self.misses = 0;
        self.held_seconds = 0.0;
        self.candidate = None;
        self.steady = None;
        self.follow(cents);
    }

    fn miss(&mut self) {
        self.misses += 1;
        if self.misses as f32 * self.config.frame_seconds >= self.config.release_seconds {
            self.state = TrackState::Holding;
            self.held_seconds = 0.0;
        }
    }

    /// Median of recent frames, then exponential smoothing.
    fn follow(&mut self, cents: f32) {
        let n = self.config.median;
        self.history[self.history_next] = cents;
        self.history_next = (self.history_next + 1) % n;
        self.history_len = (self.history_len + 1).min(n);
        let mut sorted = [0.0f32; MAX_MEDIAN];
        let window = &mut sorted[..self.history_len];
        window.copy_from_slice(&self.history[..self.history_len]);
        window.sort_by(f32::total_cmp);
        let mid = window.len() / 2;
        let median = if window.len() % 2 == 1 {
            window[mid]
        } else {
            0.5 * (window[mid - 1] + window[mid])
        };
        let alpha = 1.0 - (-self.config.frame_seconds / self.config.smoothing_seconds).exp();
        if self.history_len == 1 {
            self.smoothed = median;
        } else {
            self.smoothed += alpha * (median - self.smoothed);
        }
    }
}

/// Single-string tuning: a permissive [`PitchDetector`](crate::PitchDetector)
/// feeding a [`NoteTracker`], with the detector's analysis band narrowed to
/// the low partials of the tracked note.
#[derive(Debug, Clone)]
pub struct MonoTuner {
    detector: crate::PitchDetector,
    tracker: NoteTracker,
    default_lowpass: Option<f32>,
}

/// While tracking, the detector analyses up to this many times the tracked
/// fundamental (within [`MIN_TRACKING_LOWPASS`] and the default cutoff).
pub const TRACKING_LOWPASS_HARMONICS: f32 = 4.5;
/// Lowest low-pass cutoff used while tracking, in Hz.
pub const MIN_TRACKING_LOWPASS: f32 = 350.0;

impl MonoTuner {
    /// Analyses frames of `PitchConfig::permissive(sample_rate)` size, one
    /// every `hop` samples.
    pub fn new(sample_rate: f32, hop: usize) -> Self {
        let config = crate::PitchConfig::permissive(sample_rate);
        Self {
            default_lowpass: config.lowpass_frequency,
            detector: crate::PitchDetector::new(config),
            tracker: NoteTracker::new(TrackerConfig::new(hop as f32 / sample_rate)),
        }
    }

    pub fn frame_size(&self) -> usize {
        self.detector.config().frame_size
    }

    pub fn reset(&mut self) {
        self.tracker.reset();
        self.detector.set_lowpass(self.default_lowpass);
    }

    /// Analyses one frame of `frame_size()` samples.
    pub fn process(&mut self, frame: &[f32]) -> TrackerOutput {
        let estimate = self.detector.detect(frame);
        let mut out = self.tracker.push(estimate, self.detector.level());
        self.detector
            .set_lowpass(self.cutoff_for(out.frequency, out.state));
        if out.locked {
            // The lock was decided in the wide band; measure the same frame
            // again in the narrow one so the first reading isn't biased sharp
            // by upper partials.
            if let Some(f) = self
                .detector
                .detect(frame)
                .and_then(|e| self.tracker.reseed(e.frequency))
            {
                out.frequency = Some(f);
                self.detector
                    .set_lowpass(self.cutoff_for(out.frequency, out.state));
            }
        }
        out
    }

    fn cutoff_for(&self, frequency: Option<f32>, state: TrackState) -> Option<f32> {
        match (state, frequency, self.default_lowpass) {
            (TrackState::Tracking | TrackState::Holding, Some(f), Some(default)) => Some(
                (f * TRACKING_LOWPASS_HARMONICS).clamp(MIN_TRACKING_LOWPASS.min(default), default),
            ),
            _ => self.default_lowpass,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synth::{PluckedString, Rng, highpass, render};
    use crate::tuning::{Instrument, cents_between};

    const RATE: f32 = 48_000.0;
    const FRAME: usize = 2048;
    const HOP: usize = 1024;

    fn string(frequency: f32, level: f32, onset: f32) -> PluckedString {
        PluckedString {
            frequency,
            level,
            onset,
            inharmonicity: 1e-4,
            decay: 1.2 * (110.0 / frequency).powf(0.4),
            rolloff: 0.9,
            pluck_position: 0.15,
            polarization_cents: 1.5,
        }
    }

    /// Renders through a phone-like mic (120 Hz low cut) with noise and runs
    /// detector + tracker. Returns (time of frame end, output) per frame.
    fn track(
        strings: &[PluckedString],
        seconds: f32,
        noise: f32,
        seed: u64,
    ) -> Vec<(f32, TrackerOutput)> {
        let mut x = render(strings, RATE, seconds, noise, &mut Rng::new(seed));
        highpass(&mut x, 120.0, RATE);
        run(&x)
    }

    fn run(x: &[f32]) -> Vec<(f32, TrackerOutput)> {
        let mut tuner = MonoTuner::new(RATE, HOP);
        assert_eq!(tuner.frame_size(), FRAME);
        (0..)
            .map(|k| k * HOP)
            .take_while(|&i| i + FRAME <= x.len())
            .map(|i| ((i + FRAME) as f32 / RATE, tuner.process(&x[i..i + FRAME])))
            .collect()
    }

    fn all_strings() -> impl Iterator<Item = f32> {
        [Instrument::Guitar, Instrument::Ukulele]
            .into_iter()
            .flat_map(|i| i.strings().iter().map(|s| s.frequency))
    }

    #[test]
    fn quiet_plucks_stay_on_display_for_two_seconds() {
        // Peak ≈ −34 dBFS, noise ≈ −50 dBFS: where the stateless detector
        // showed a reading for only 25–75% of this window.
        for (k, f) in all_strings().enumerate() {
            let out = track(&[string(f, 0.02, 0.1)], 2.3, 0.003, k as u64);
            let window: Vec<_> = out
                .iter()
                .filter(|(t, _)| *t > 0.15 && *t <= 2.15)
                .collect();
            let shown = window.iter().filter(|(_, o)| o.frequency.is_some()).count();
            assert!(
                shown as f32 >= 0.95 * window.len() as f32,
                "{f} Hz shown in {shown}/{} frames",
                window.len()
            );
            for (t, o) in &window {
                if let Some(hz) = o.frequency {
                    let cents = cents_between(hz, f);
                    assert!(
                        cents.abs() <= 3.0,
                        "{f} Hz shown as {hz} ({cents:+.1} c) at {t:.2}s"
                    );
                }
            }
        }
    }

    #[test]
    fn a_note_already_sounding_at_start_is_picked_up() {
        let rate = RATE;
        let tone: Vec<f32> = (0..FRAME * 20)
            .map(|i| 0.05 * (std::f32::consts::TAU * 196.0 * i as f32 / rate).sin())
            .collect();
        let out = run(&tone);
        assert!(
            out.iter()
                .skip(2)
                .all(|(_, o)| o.state == TrackState::Tracking)
        );
    }

    #[test]
    fn a_steady_tone_without_onset_locks() {
        // The floor learns the tone's level from an imperfect first frame,
        // so there is never an onset; the steady pitch still locks.
        let mut t = NoteTracker::new(TrackerConfig::new(HOP as f32 / RATE));
        let est = |clarity: f32| {
            Some(PitchEstimate {
                frequency: 110.0,
                clarity,
            })
        };
        assert_eq!(t.push(est(0.8), 0.2).state, TrackState::Idle);
        let frames_needed = (0.15 / (HOP as f32 / RATE)).ceil() as usize;
        for _ in 1..frames_needed {
            assert_eq!(t.push(est(0.99), 0.2).state, TrackState::Idle);
        }
        let out = t.push(est(0.99), 0.2);
        assert_eq!(out.state, TrackState::Tracking);
        assert!(out.locked);
        // ...and keeps tracking it rather than releasing it as "background".
        for _ in 0..100 {
            assert_eq!(t.push(est(0.99), 0.2).state, TrackState::Tracking);
        }
    }

    #[test]
    fn noise_alone_never_locks() {
        for seed in 0..5 {
            let mut x = render(&[], RATE, 5.0, 0.003, &mut Rng::new(seed));
            highpass(&mut x, 120.0, RATE);
            assert!(
                run(&x).iter().all(|(_, o)| o.state == TrackState::Idle),
                "seed {seed}"
            );
        }
    }

    #[test]
    fn a_pluck_shows_within_100_ms() {
        for (k, f) in all_strings().enumerate() {
            let out = track(&[string(f, 0.02, 0.5)], 1.0, 0.003, 100 + k as u64);
            let first = out
                .iter()
                .find(|(_, o)| o.state == TrackState::Tracking)
                .expect("locks");
            assert!(
                first.0 - 0.5 <= 0.1,
                "{f} Hz locked {:.0} ms after the pluck",
                (first.0 - 0.5) * 1e3
            );
        }
    }

    #[test]
    fn switching_strings_retargets_within_150_ms() {
        let (a2, d3) = (110.0, 146.832);
        // A2 still ringing when D3 is plucked.
        let out = track(
            &[string(a2, 0.02, 0.1), string(d3, 0.02, 1.5)],
            2.5,
            0.002,
            7,
        );
        let before = out.iter().filter(|(t, _)| (1.2..1.5).contains(t));
        assert!(before.clone().all(|(_, o)| {
            o.frequency
                .is_some_and(|hz| cents_between(hz, a2).abs() < 3.0)
        }));
        let switched = out
            .iter()
            .find(|(t, o)| {
                *t > 1.5
                    && o.frequency
                        .is_some_and(|hz| cents_between(hz, d3).abs() < 3.0)
            })
            .expect("retargets");
        assert!(
            switched.0 - 1.5 <= 0.15,
            "retargeted after {:.0} ms",
            (switched.0 - 1.5) * 1e3
        );
    }

    #[test]
    fn faded_reading_is_held_then_cleared() {
        let mut x = render(
            &[string(196.0, 0.03, 0.1)],
            RATE,
            6.0,
            0.002,
            &mut Rng::new(11),
        );
        // The string is damped abruptly at 1.0 s; only noise remains.
        let mut rng = Rng::new(12);
        for v in &mut x[(1.0 * RATE) as usize..] {
            *v = 0.002 * rng.normal();
        }
        highpass(&mut x, 120.0, RATE);
        let out = run(&x);
        let state_at = |t: f32| out.iter().find(|(ft, _)| *ft >= t).unwrap().1;
        assert_eq!(state_at(0.8).state, TrackState::Tracking);
        let held = state_at(1.5);
        assert_eq!(held.state, TrackState::Holding);
        assert!(cents_between(held.frequency.unwrap(), 196.0).abs() < 3.0);
        assert_eq!(state_at(3.8).state, TrackState::Holding);
        assert_eq!(state_at(4.8).state, TrackState::Idle);
        assert_eq!(state_at(4.8).frequency, None);
    }

    #[test]
    fn two_strings_ringing_together_do_not_fake_a_new_note() {
        // B3 still rings when a flat E4 is plucked; as both decay, the mixture
        // repeats at E2's period (B3:E4 = 3:4). Without an attack that must
        // not replace the E4 reading.
        let e4 = 329.628 * 2f32.powf(-12.0 / 1200.0);
        let out = track(
            &[string(246.94, 0.08, 0.1), string(e4, 0.08, 2.1)],
            4.3,
            0.002,
            21,
        );
        for (t, o) in out.iter().filter(|(t, _)| (2.2..4.1).contains(t)) {
            let hz = o.frequency.expect("shown");
            assert!(cents_between(hz, e4).abs() < 5.0, "{t:.2}s shows {hz} Hz");
        }
    }

    #[test]
    fn octave_slips_are_folded_back() {
        let config = TrackerConfig::new(HOP as f32 / RATE);
        let mut t = NoteTracker::new(config);
        let est = |hz: f32, clarity: f32| {
            Some(PitchEstimate {
                frequency: hz,
                clarity,
            })
        };
        // Establish a floor, then lock on 110 Hz.
        for _ in 0..20 {
            t.push(None, 0.001);
        }
        assert_eq!(
            t.push(est(110.0, 0.95), 0.05).state,
            TrackState::Idle,
            "needs confirming"
        );
        assert_eq!(t.push(est(110.0, 0.95), 0.05).state, TrackState::Tracking);
        for hz in [220.0, 55.0, 440.0, 27.5] {
            for _ in 0..10 {
                let o = t.push(est(hz, 0.6), 0.05);
                assert_eq!(o.state, TrackState::Tracking);
                assert!(
                    cents_between(o.frequency.unwrap(), 110.0).abs() < 0.5,
                    "{hz}"
                );
            }
        }
    }
}
