//! Chord recognition for a strum: which fret each string was played at
//! (a tab). Naming the chord is music theory, done by the web app
//! (`web/src/lib/theory`).
//!
//! The strum's spectrum is reduced to its peaks. Every string gets
//! candidates at frets `0..=MAX_FRET`, each kept only if a peak sits at its
//! fundamental. Every combination of candidates (or a muted string) with a
//! playable fret span is scored by how much of the peak weight its partials
//! explain; the best wins. A pitch can be played on several strings, and a
//! string doubling another's note explains nothing new, so ties go to the
//! combination with more strings played and lower frets (how chords are
//! usually fingered).
//!
//! Exhaustive search is cheap for the ukulele (at most 14⁴ combinations of
//! precomputed bit masks). Assumes standard tuning.

use crate::poly::Spectrum;
use crate::tuning::{Instrument, cents_between};

/// Highest fret considered.
pub const MAX_FRET: u8 = 12;
/// Largest distance between the lowest and highest fretted (non-open)
/// string.
const MAX_SPAN: u8 = 4;
/// A peak counts as a candidate's partial within this many cents.
const TOLERANCE_CENTS: f32 = 40.0;
/// Peaks are looked for up to this frequency.
const MAX_PEAK_HZ: f32 = 4000.0;
/// Peaks weaker than the strongest by more than this are ignored; weights
/// are dB above that floor.
const DYNAMIC_RANGE_DB: f32 = 60.0;
/// A candidate's fundamental must be a peak at most this far below the
/// strongest one (played fundamentals are strong; noise peaks are not).
const FUNDAMENTAL_RANGE_DB: f32 = 30.0;
/// A peak must stand this far above the median of the spectrum within a
/// semitone around it.
const PEAK_PROMINENCE_DB: f32 = 10.0;
/// At most this many (strongest) peaks are used: one bit each.
const MAX_PEAKS: usize = 64;
/// A spectrum is only searched if its loudest bin stands this far above
/// the median bin (synthetic strums >= 39 dB, noise bursts <= 12 dB).
const MIN_CREST_DB: f32 = 25.0;
/// A reading is only reported if its strings explain this share of the
/// peak weight...
const MIN_EXPLAINED: f32 = 0.75;
/// ...and every played string explains at least this many peaks.
const MIN_HARMONICS: u32 = 2;
/// Penalties (in explained-weight fraction) that break ties.
const MUTED_PENALTY: f32 = 0.02;
const FRET_PENALTY: f32 = 0.001;
const MAX_FRET_PENALTY: f32 = 0.002;
const COMMON_SHAPE_BONUS: f32 = 0.01;

/// Common ukulele chord shapes (frets in string order G C E A). The same
/// notes can often be fingered several ways; when the sound can't tell them
/// apart, one of these wins.
pub const COMMON_UKULELE_SHAPES: &[[u8; 4]] = &[
    [0, 0, 0, 3], // C
    [0, 0, 0, 1], // C7
    [0, 0, 0, 2], // Cmaj7
    [0, 3, 3, 3], // Cm
    [0, 0, 1, 3], // Csus4
    [0, 2, 3, 3], // Csus2
    [1, 0, 0, 3], // Caug
    [2, 2, 2, 0], // D
    [2, 2, 2, 3], // D7
    [2, 2, 1, 0], // Dm
    [2, 2, 1, 3], // Dm7
    [2, 2, 0, 0], // Dsus2
    [0, 2, 3, 0], // Dsus4
    [4, 4, 4, 2], // E
    [1, 4, 0, 2], // E (alternative)
    [1, 2, 0, 2], // E7
    [0, 4, 3, 2], // Em
    [0, 2, 0, 2], // Em7
    [2, 0, 1, 0], // F
    [2, 3, 1, 3], // F7
    [2, 4, 1, 3], // Fmaj7
    [1, 0, 1, 3], // Fm
    [0, 2, 3, 2], // G
    [0, 2, 1, 2], // G7
    [0, 2, 2, 2], // Gmaj7
    [0, 2, 3, 1], // Gm
    [0, 2, 3, 3], // Gsus4
    [2, 1, 0, 0], // A
    [0, 1, 0, 0], // A7
    [2, 0, 0, 0], // Am
    [0, 0, 0, 0], // Am7
    [2, 2, 0, 0], // Asus2 / Dsus2
    [3, 2, 1, 1], // Bb
    [1, 2, 1, 1], // Bb7
    [3, 1, 1, 1], // Bbm
    [4, 3, 2, 2], // B
    [2, 3, 2, 2], // B7
    [4, 2, 2, 2], // Bm
    [2, 2, 2, 2], // Bm7
    [4, 2, 1, 2], // Bdim
    [1, 1, 1, 4], // C#
    [1, 1, 0, 4], // C#m
    [0, 3, 3, 1], // Eb
    [3, 3, 3, 1], // Eb (alternative)
    [2, 1, 2, 0], // F#m
    [3, 1, 2, 1], // F#
    [1, 3, 4, 3], // Ab
    [0, 0, 1, 0], // Fadd9
    [0, 2, 0, 3], // Cadd9
    [0, 0, 0, 5], // Cadd9 (high D)
    [0, 2, 0, 0], // A7sus4
    [0, 0, 1, 1], // C7sus4
    [2, 0, 0, 2], // Amadd9
    [2, 1, 0, 2], // Aadd9
    [1, 2, 1, 2], // dim7 shape (Ddim7 = Fdim7 = G#dim7 = Bdim7)
    [2, 3, 2, 3], // dim7 shape
    [3, 4, 3, 3], // C7 (closed)
    [2, 2, 1, 2], // Bm7b5 / Dm6
    [0, 2, 1, 1], // Gm7
    [0, 2, 0, 1], // Gm6
    [1, 0, 0, 1], // C7#5
];

/// A recognised strum.
#[derive(Debug, Clone, PartialEq)]
pub struct ChordReading {
    /// Fret per string in string order; `None` = muted / not played.
    pub frets: Vec<Option<u8>>,
    /// Sounding MIDI note per string (`None` = muted).
    pub midi: Vec<Option<i32>>,
    /// Share of the spectrum's peak weight the tab explains (0..1).
    pub explained: f32,
}

struct Peak {
    hz: f32,
    weight: f32,
}

/// Loudest bin over the median bin (dB) between 150 Hz and `MAX_PEAK_HZ`:
/// tonal sounds have peaks far above their spectrum's general level.
fn crest_db(spectrum: &Spectrum) -> f32 {
    let lo = (150.0 / spectrum.bin_hz) as usize;
    let hi = ((MAX_PEAK_HZ / spectrum.bin_hz) as usize).min(spectrum.db.len() - 1);
    if lo >= hi {
        return 0.0;
    }
    let mut band = spectrum.db[lo..=hi].to_vec();
    band.sort_by(f32::total_cmp);
    band[band.len() - 1] - band[band.len() / 2]
}

fn find_peaks(spectrum: &Spectrum) -> Vec<Peak> {
    let db = &spectrum.db;
    let lo = (150.0 / spectrum.bin_hz) as usize;
    let hi = ((MAX_PEAK_HZ / spectrum.bin_hz) as usize).min(db.len() - 3);
    if lo + 3 >= hi {
        return vec![];
    }
    let top = db[lo..=hi].iter().copied().fold(f32::MIN, f32::max);
    // Blackman–Harris main lobe: ±4 bins of the unpadded frame.
    let lobe_bins =
        (4.0 * spectrum.sample_rate / spectrum.frame_len as f32 / spectrum.bin_hz).ceil() as usize;
    let floor = top - DYNAMIC_RANGE_DB;
    let mut peaks: Vec<Peak> = (lo.max(2)..=hi)
        .filter(|&i| {
            db[i] > floor
                && db[i] >= db[i - 1]
                && db[i] >= db[i - 2]
                && db[i] > db[i + 1]
                && db[i] > db[i + 2]
        })
        .filter(|&i| {
            // Background: outside the window's main lobe, out to a semitone
            // or two lobes, whichever is wider, on either side.
            let hz = i as f32 * spectrum.bin_hz;
            let reach = ((hz * (2f32.powf(1.0 / 12.0) - 1.0)) / spectrum.bin_hz)
                .ceil()
                .max(2.0 * lobe_bins as f32) as usize;
            // Each side separately, the quieter one counts: a neighbour a
            // semitone away (E4 under F4) fills one side with its main lobe.
            let median = |range: std::ops::Range<usize>| {
                let mut side: Vec<f32> = range.filter(|&j| j < db.len()).map(|j| db[j]).collect();
                side.sort_by(f32::total_cmp);
                side.get(side.len() / 2).copied()
            };
            let left = median(i.saturating_sub(reach)..i.saturating_sub(lobe_bins));
            let right = median(i + lobe_bins + 1..i + reach + 1);
            let background = match (left, right) {
                (Some(l), Some(r)) => l.min(r),
                (l, r) => l.or(r).unwrap_or(f32::MIN),
            };
            db[i] - background >= PEAK_PROMINENCE_DB
        })
        .map(|i| {
            let (l, m, r) = (db[i - 1], db[i], db[i + 1]);
            let denom = l - 2.0 * m + r;
            let shift = if denom < 0.0 {
                0.5 * (l - r) / denom
            } else {
                0.0
            };
            Peak {
                hz: (i as f32 + shift) * spectrum.bin_hz,
                weight: m - floor,
            }
        })
        .collect();
    if peaks.len() > MAX_PEAKS {
        // Keep the strongest. Sorting plain f32s with `total_cmp` reuses the
        // sort code the rest of the crate already pulls into the wasm.
        let mut weights: Vec<f32> = peaks.iter().map(|p| p.weight).collect();
        weights.sort_by(f32::total_cmp);
        let min = weights[weights.len() - MAX_PEAKS];
        peaks.retain(|p| p.weight >= min);
        peaks.truncate(MAX_PEAKS);
    }
    peaks
}

/// One string's option: a fret (or muted) and the peaks its partials explain.
#[derive(Clone, Copy)]
struct Candidate {
    fret: Option<u8>,
    mask: u64,
}

/// Recognises the chord in `frame`, an analysis window after a strum.
/// `None` if no confident reading is found (silence, noise, speech).
pub fn detect_chord(
    frame: &[f32],
    sample_rate: f32,
    instrument: Instrument,
    inharmonicity: f32,
) -> Option<ChordReading> {
    let spectrum = Spectrum::new(frame, sample_rate, 2);
    // Not tonal (noise, knocks, speech hiss): no chord, and no search.
    if crest_db(&spectrum) < MIN_CREST_DB {
        return None;
    }
    let peaks = find_peaks(&spectrum);
    if peaks.is_empty() {
        return None;
    }
    let total: f32 = peaks.iter().map(|p| p.weight).sum();
    // Weights are dB above the floor, so the strongest peak's is the range.
    let strong_weight = peaks.iter().map(|p| p.weight).fold(0.0, f32::max) - FUNDAMENTAL_RANGE_DB;
    let near = |hz: f32| move |p: &Peak| cents_between(p.hz, hz).abs() <= TOLERANCE_CENTS;

    let strings = instrument.strings();
    let candidates: Vec<Vec<Candidate>> = strings
        .iter()
        .map(|s| {
            let mut options = vec![Candidate {
                fret: None,
                mask: 0,
            }];
            for fret in 0..=MAX_FRET {
                let f0 = s.frequency * 2f32.powf(f32::from(fret) / 12.0);
                if !peaks
                    .iter()
                    .any(|p| near(f0)(p) && p.weight >= strong_weight)
                {
                    continue;
                }
                let mut mask = 0u64;
                for n in 1u32.. {
                    let n = n as f32;
                    let hz = n * f0 * (1.0 + inharmonicity * n * n).sqrt();
                    if hz > MAX_PEAK_HZ * 1.03 {
                        break;
                    }
                    for (i, p) in peaks.iter().enumerate() {
                        if near(hz)(p) {
                            mask |= 1 << i;
                        }
                    }
                }
                options.push(Candidate {
                    fret: Some(fret),
                    mask,
                });
            }
            options
        })
        .collect();

    let weights: Vec<f32> = peaks.iter().map(|p| p.weight).collect();
    let explained = |mask: u64| -> f32 {
        let mut m = mask;
        let mut sum = 0.0;
        while m != 0 {
            sum += weights[m.trailing_zeros() as usize];
            m &= m - 1;
        }
        sum
    };

    // Odometer over every string's candidates.
    let mut index = vec![0usize; strings.len()];
    // (score, frets, share, fewest peaks explained by one played string)
    let mut best: Option<(f32, Vec<Option<u8>>, f32, u32)> = None;
    loop {
        let chosen: Vec<Candidate> = index.iter().zip(&candidates).map(|(&i, c)| c[i]).collect();
        let fretted = chosen.iter().filter_map(|c| c.fret).filter(|&f| f > 0);
        let (lo, hi) = fretted.fold((u8::MAX, 0u8), |(lo, hi), f| (lo.min(f), hi.max(f)));
        let played = chosen.iter().filter(|c| c.fret.is_some()).count();
        if played > 0 && (lo == u8::MAX || hi - lo <= MAX_SPAN) {
            let mask = chosen.iter().fold(0, |m, c| m | c.mask);
            let share = explained(mask) / total;
            let frets: u32 = chosen.iter().filter_map(|c| c.fret).map(u32::from).sum();
            let top = chosen.iter().filter_map(|c| c.fret).max().unwrap_or(0);
            let muted = strings.len() - played;
            let common = instrument == Instrument::Ukulele
                && COMMON_UKULELE_SHAPES
                    .iter()
                    .any(|shape| shape.iter().zip(&chosen).all(|(&f, c)| c.fret == Some(f)));
            let score = share
                - MUTED_PENALTY * muted as f32
                - FRET_PENALTY * frets as f32
                - MAX_FRET_PENALTY * f32::from(top)
                + if common { COMMON_SHAPE_BONUS } else { 0.0 };
            if best.as_ref().is_none_or(|(b, ..)| score > *b) {
                let harmonics = chosen
                    .iter()
                    .filter(|c| c.fret.is_some())
                    .map(|c| c.mask.count_ones())
                    .min()
                    .unwrap_or(0);
                best = Some((
                    score,
                    chosen.iter().map(|c| c.fret).collect(),
                    share,
                    harmonics,
                ));
            }
        }
        // Next combination.
        let mut s = 0;
        loop {
            if s == index.len() {
                let (_, frets, share, harmonics) = best?;
                // Confidence: a strum's partials explain nearly all peaks
                // (>= 0.84 on synthetic strums, noise bursts mostly < 0.75),
                // and every played string shows more than its fundamental.
                if share < MIN_EXPLAINED || harmonics < MIN_HARMONICS {
                    return None;
                }
                return Some(reading(instrument, frets, share));
            }
            index[s] += 1;
            if index[s] < candidates[s].len() {
                break;
            }
            index[s] = 0;
            s += 1;
        }
    }
}

fn reading(instrument: Instrument, frets: Vec<Option<u8>>, explained: f32) -> ChordReading {
    let midi: Vec<Option<i32>> = frets
        .iter()
        .zip(instrument.strings())
        .map(|(f, s)| f.map(|f| s.midi + i32::from(f)))
        .collect();
    ChordReading {
        frets,
        midi,
        explained,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synth::{Rng, chord_strum, render};

    #[test]
    fn recognises_a_synthetic_c_major_strum() {
        let rate = 48_000.0;
        let mut rng = Rng::new(3);
        let frets = [Some(0), Some(0), Some(0), Some(3)];
        let strum = chord_strum(Instrument::Ukulele, &frets, 3.0, &mut rng);
        let signal = render(&strum, rate, 0.6, 0.002, &mut rng);
        let start = (0.15 * rate) as usize;
        let r = detect_chord(
            &signal[start..start + 16_384],
            rate,
            Instrument::Ukulele,
            2e-5,
        )
        .expect("chord");
        assert_eq!(r.frets, frets);
        assert_eq!(r.midi, [Some(67), Some(60), Some(64), Some(72)]);
    }

    #[test]
    fn silence_has_no_chord() {
        let frame = vec![0.0; 16_384];
        assert!(detect_chord(&frame, 48_000.0, Instrument::Ukulele, 2e-5).is_none());
    }
}
