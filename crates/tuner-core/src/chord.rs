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
/// A candidate's fundamental must also stand this far above its local
/// background: a string, not a noise peak (strings measure 40 dB and more).
const FUNDAMENTAL_PROMINENCE_DB: f32 = 20.0;
/// A peak must stand this far above the median of the spectrum within a
/// semitone around it.
const PEAK_PROMINENCE_DB: f32 = 10.0;
/// At most this many (strongest) peaks are used: one bit each.
const MAX_PEAKS: usize = 64;
/// A spectrum is only searched if its loudest bin stands this far above
/// the median bin (synthetic strums >= 39 dB, noise bursts <= 12 dB).
const MIN_CREST_DB: f32 = 25.0;
/// Median -6 dB width (bins of the unpadded frame) of the played strings'
/// fundamentals above which the sound isn't steady enough to be strings.
const MAX_PEAK_WIDTH_BINS: f32 = 5.0;
/// Semitones above a note where its own overtones sound (unison, octave,
/// octave + fifth, two octaves, + major third, + fifth, + minor seventh,
/// three octaves).
const OVERTONES: [i32; 8] = [0, 12, 19, 24, 28, 31, 34, 36];
/// No chord if a peak below the instrument's lowest note is within this
/// many dB of the strongest peak.
const BELOW_RANGE_DB: f32 = 15.0;
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
    /// Per string: offset in cents of the sounding note from the fretted
    /// pitch in standard tuning (median over its first partials that no
    /// other string shares); `None` if muted or every partial is shared.
    pub cents: Vec<Option<f32>>,
}

struct Peak {
    hz: f32,
    weight: f32,
    /// dB above the quieter side's median (see `find_peaks`).
    prominence: f32,
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

/// Width at -6 dB, in bins of the unpadded frame, of the strongest bin
/// within ±40 padded bins of `hz`.
fn peak_width_bins(spectrum: &Spectrum, hz: f32) -> Option<f32> {
    let db = &spectrum.db;
    let c = (hz / spectrum.bin_hz).round() as usize;
    let (lo, hi) = (c.saturating_sub(40), (c + 40).min(db.len() - 1));
    let i = (lo..=hi).max_by(|&a, &b| db[a].total_cmp(&db[b]))?;
    let level = db[i] - 6.0;
    let (mut l, mut r) = (i, i);
    while l > 0 && db[l] > level {
        l -= 1;
    }
    while r + 1 < db.len() && db[r] > level {
        r += 1;
    }
    let unpadded = spectrum.sample_rate / spectrum.frame_len as f32;
    Some((r - l) as f32 * spectrum.bin_hz / unpadded)
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
        .filter_map(|i| {
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
            let prominence = db[i] - background;
            if prominence < PEAK_PROMINENCE_DB {
                return None;
            }
            let (l, m, r) = (db[i - 1], db[i], db[i + 1]);
            let denom = l - 2.0 * m + r;
            let shift = if denom < 0.0 {
                0.5 * (l - r) / denom
            } else {
                0.0
            };
            Some(Peak {
                hz: (i as f32 + shift) * spectrum.bin_hz,
                weight: m - floor,
                prominence,
            })
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
    let top_weight = peaks.iter().map(|p| p.weight).fold(0.0, f32::max);
    // A strong peak below the lowest note the instrument can play (voices
    // talking or humming, 90-250 Hz) means it isn't (only) the instrument.
    let lowest = instrument
        .strings()
        .iter()
        .map(|s| s.frequency)
        .fold(f32::MAX, f32::min)
        * 2f32.powf(-TOLERANCE_CENTS / 1200.0);
    let band_max = |lo: f32, hi: f32| {
        let (lo, hi) = (
            (lo / spectrum.bin_hz) as usize,
            (hi / spectrum.bin_hz) as usize,
        );
        spectrum.db[lo..hi.min(spectrum.db.len())]
            .iter()
            .copied()
            .fold(f32::MIN, f32::max)
    };
    // Below the lowest note's window main lobe (±4 bins of the frame).
    let lobe_hz = 4.0 * sample_rate / frame.len() as f32;
    if band_max(70.0, lowest - lobe_hz) >= band_max(lowest, MAX_PEAK_HZ) - BELOW_RANGE_DB {
        return None;
    }
    // Weights are dB above the floor, so the strongest peak's is the range.
    let strong_weight = top_weight - FUNDAMENTAL_RANGE_DB;
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
                if !peaks.iter().any(|p| {
                    near(f0)(p)
                        && p.weight >= strong_weight
                        && p.prominence >= FUNDAMENTAL_PROMINENCE_DB
                }) {
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
                // A plucked string holds its pitch: its peak is as narrow as
                // the window allows (~3.5 bins at -6 dB; 99% <= 4 on
                // synthetic strums). Gliding voices and whistles smear
                // (>= 6.5).
                let mut widths: Vec<f32> = frets
                    .iter()
                    .zip(strings)
                    .filter_map(|(f, s)| {
                        let f0 = s.frequency * 2f32.powf(f32::from((*f)?) / 12.0);
                        peak_width_bins(&spectrum, f0)
                    })
                    .collect();
                widths.sort_by(f32::total_cmp);
                if widths
                    .get(widths.len() / 2)
                    .is_some_and(|&w| w > MAX_PEAK_WIDTH_BINS)
                {
                    return None;
                }
                // One pitch and its overtones (a hummed or sung note, a
                // single plucked string): not a chord.
                let notes: Vec<i32> = frets
                    .iter()
                    .zip(strings)
                    .filter_map(|(f, s)| f.map(|f| s.midi + i32::from(f)))
                    .collect();
                let low = notes.iter().copied().min().unwrap_or(0);
                if notes.iter().all(|n| OVERTONES.contains(&(n - low))) {
                    return None;
                }
                return Some(reading(instrument, frets, share, &peaks, inharmonicity));
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

/// Partials used for a string's intonation.
const CENTS_PARTIALS: u32 = 4;

fn reading(
    instrument: Instrument,
    frets: Vec<Option<u8>>,
    explained: f32,
    peaks: &[Peak],
    inharmonicity: f32,
) -> ChordReading {
    let strings = instrument.strings();
    let midi: Vec<Option<i32>> = frets
        .iter()
        .zip(strings)
        .map(|(f, s)| f.map(|f| s.midi + i32::from(f)))
        .collect();
    let f0: Vec<Option<f32>> = frets
        .iter()
        .zip(strings)
        .map(|(f, s)| f.map(|f| s.frequency * 2f32.powf(f32::from(f) / 12.0)))
        .collect();
    let partial = |f: f32, n: u32| {
        let n = n as f32;
        n * f * (1.0 + inharmonicity * n * n).sqrt()
    };
    let cents = f0
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let f = (*f)?;
            let mut offsets: Vec<f32> = (1..=CENTS_PARTIALS)
                .map(|n| partial(f, n))
                // Skip partials another sounding string also produces.
                .filter(|&hz| {
                    !f0.iter().enumerate().any(|(j, g)| {
                        j != i
                            && g.is_some_and(|g| {
                                (1..=8).any(|m| cents_between(partial(g, m), hz).abs() < 60.0)
                            })
                    })
                })
                .filter_map(|hz| {
                    peaks
                        .iter()
                        .map(|p| cents_between(p.hz, hz))
                        .filter(|c| c.abs() <= TOLERANCE_CENTS)
                        .min_by(|a, b| a.abs().total_cmp(&b.abs()))
                })
                .collect();
            offsets.sort_by(f32::total_cmp);
            offsets.get(offsets.len() / 2).copied()
        })
        .collect();
    ChordReading {
        frets,
        midi,
        explained,
        cents,
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

    /// A sung or hummed tone: `f0` with decaying harmonics, optionally
    /// gliding to `f1`, with light vibrato.
    fn voice(f0: f32, f1: f32, harmonics: usize) -> Vec<f32> {
        let rate = 48_000.0;
        let tau = std::f32::consts::TAU;
        let mut phase = vec![0.0f32; harmonics];
        (0..16_384)
            .map(|k| {
                let u = k as f32 / 16_384.0;
                let f = (f0 + (f1 - f0) * u) * (1.0 + 0.004 * (tau * 5.5 * k as f32 / rate).sin());
                phase
                    .iter_mut()
                    .enumerate()
                    .map(|(h, ph)| {
                        *ph = (*ph + tau * f * (h + 1) as f32 / rate) % tau;
                        0.1 * ph.sin() / ((h + 1) * (h + 1)) as f32
                    })
                    .sum()
            })
            .collect()
    }

    #[test]
    fn voices_and_whistles_are_not_chords() {
        let cases = [
            ("hummed D4 (in range, one pitch)", voice(293.7, 293.7, 8)),
            ("hummed G3 (below the ukulele)", voice(196.0, 196.0, 8)),
            ("talking pitch glide", voice(140.0, 180.0, 20)),
            ("whistle glide", voice(900.0, 1300.0, 1)),
        ];
        for (name, x) in cases {
            let r = detect_chord(&x, 48_000.0, Instrument::Ukulele, 2e-5);
            assert!(r.is_none(), "{name}: {:?}", r.map(|r| r.frets));
        }
    }
}
