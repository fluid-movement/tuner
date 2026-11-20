//! Chord recognition for a strum: which fret each string was played at
//! (a tab) and the chord's name.
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
/// Penalties (in explained-weight fraction) that break ties.
const MUTED_PENALTY: f32 = 0.02;
const FRET_PENALTY: f32 = 0.001;
const MAX_FRET_PENALTY: f32 = 0.002;
const COMMON_SHAPE_BONUS: f32 = 0.01;

/// Common ukulele chord shapes (frets in string order G C E A). The same
/// notes can often be fingered several ways; when the sound can't tell them
/// apart, one of these wins.
pub const COMMON_UKULELE_SHAPES: [[u8; 4]; 47] = [
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
];

/// Chord qualities recognised by [`name_chord`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Quality {
    Major = 0,
    Minor = 1,
    Seventh = 2,
    MajorSeventh = 3,
    MinorSeventh = 4,
    Sus2 = 5,
    Sus4 = 6,
    Diminished = 7,
    Augmented = 8,
}

impl Quality {
    /// In order of preference when a set of notes has several names.
    pub const ALL: [Quality; 9] = [
        Quality::Major,
        Quality::Minor,
        Quality::Seventh,
        Quality::MinorSeventh,
        Quality::MajorSeventh,
        Quality::Sus4,
        Quality::Sus2,
        Quality::Diminished,
        Quality::Augmented,
    ];

    /// Suffix after the root, such as `"m7"`.
    pub fn suffix(self) -> &'static str {
        match self {
            Quality::Major => "",
            Quality::Minor => "m",
            Quality::Seventh => "7",
            Quality::MajorSeventh => "maj7",
            Quality::MinorSeventh => "m7",
            Quality::Sus2 => "sus2",
            Quality::Sus4 => "sus4",
            Quality::Diminished => "dim",
            Quality::Augmented => "aug",
        }
    }

    /// Semitones above the root, and whether the fifth (7) may be left out.
    fn intervals(self) -> (&'static [u8], bool) {
        match self {
            Quality::Major => (&[0, 4, 7], false),
            Quality::Minor => (&[0, 3, 7], false),
            Quality::Seventh => (&[0, 4, 7, 10], true),
            Quality::MajorSeventh => (&[0, 4, 7, 11], true),
            Quality::MinorSeventh => (&[0, 3, 7, 10], true),
            Quality::Sus2 => (&[0, 2, 7], false),
            Quality::Sus4 => (&[0, 5, 7], false),
            Quality::Diminished => (&[0, 3, 6], false),
            Quality::Augmented => (&[0, 4, 8], false),
        }
    }
}

/// Root (pitch class, 0 = C) and quality.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChordName {
    pub root: u8,
    pub quality: Quality,
}

const ROOT_NAMES: [&str; 12] = [
    "C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B",
];

impl std::fmt::Display for ChordName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            ROOT_NAMES[self.root as usize % 12],
            self.quality.suffix()
        )
    }
}

/// Names the chord formed by `midi` notes (any order, doublings allowed).
/// Among several fitting names, one rooted on the lowest note wins, then
/// the order of [`Quality::ALL`]. `None` if no quality fits.
pub fn name_chord(midi: &[i32]) -> Option<ChordName> {
    let bass = midi.iter().copied().min()?.rem_euclid(12) as u8;
    let set: u16 = midi
        .iter()
        .fold(0, |s, &m| s | 1 << m.rem_euclid(12) as u16);
    let mut best: Option<(bool, usize, ChordName)> = None;
    for root in 0..12u8 {
        if set & (1 << root) == 0 {
            continue;
        }
        let rotated = ((set >> root) | (set << (12 - root))) & 0xfff;
        for (rank, &quality) in Quality::ALL.iter().enumerate() {
            let (intervals, fifth_optional) = quality.intervals();
            let full: u16 = intervals.iter().fold(0, |s, &i| s | 1 << i);
            let fits = rotated == full || (fifth_optional && rotated == full & !(1 << 7));
            if !fits {
                continue;
            }
            let key = (root != bass, rank);
            if best.is_none_or(|(b, r, _)| key < (b, r)) {
                best = Some((key.0, key.1, ChordName { root, quality }));
            }
        }
    }
    best.map(|(_, _, name)| name)
}

/// A recognised strum.
#[derive(Debug, Clone, PartialEq)]
pub struct ChordReading {
    /// Fret per string in string order; `None` = muted / not played.
    pub frets: Vec<Option<u8>>,
    /// Sounding MIDI note per string (`None` = muted).
    pub midi: Vec<Option<i32>>,
    /// The chord's name, if the notes form a known chord.
    pub name: Option<ChordName>,
    /// Share of the spectrum's peak weight the tab explains (0..1).
    pub explained: f32,
}

struct Peak {
    hz: f32,
    weight: f32,
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
            // or two lobes, whichever is wider.
            let hz = i as f32 * spectrum.bin_hz;
            let reach = ((hz * (2f32.powf(1.0 / 12.0) - 1.0)) / spectrum.bin_hz)
                .ceil()
                .max(2.0 * lobe_bins as f32) as usize;
            let mut around: Vec<f32> = (i.saturating_sub(reach)..(i + reach + 1).min(db.len()))
                .filter(|&j| j.abs_diff(i) > lobe_bins)
                .map(|j| db[j])
                .collect();
            around.sort_by(f32::total_cmp);
            around
                .get(around.len() / 2)
                .is_none_or(|&median| db[i] - median >= PEAK_PROMINENCE_DB)
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
/// `None` if no string's fundamental is found (silence, noise).
pub fn detect_chord(
    frame: &[f32],
    sample_rate: f32,
    instrument: Instrument,
    inharmonicity: f32,
) -> Option<ChordReading> {
    let spectrum = Spectrum::new(frame, sample_rate, 2);
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
    let mut best: Option<(f32, Vec<Option<u8>>, f32)> = None;
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
            if best.as_ref().is_none_or(|(b, _, _)| score > *b) {
                best = Some((score, chosen.iter().map(|c| c.fret).collect(), share));
            }
        }
        // Next combination.
        let mut s = 0;
        loop {
            if s == index.len() {
                let (_, frets, share) = best?;
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
    let notes: Vec<i32> = midi.iter().flatten().copied().collect();
    ChordReading {
        name: name_chord(&notes),
        frets,
        midi,
        explained,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synth::{Rng, chord_strum, render};

    fn name(midi: &[i32]) -> String {
        name_chord(midi).map_or("?".into(), |n| n.to_string())
    }

    #[test]
    fn names_common_chords() {
        // Ukulele G C E A strings: 67 60 64 69.
        assert_eq!(name(&[67, 60, 64, 72]), "C"); // 0003
        assert_eq!(name(&[69, 60, 64, 69]), "Am"); // 2000
        assert_eq!(name(&[69, 60, 65, 69]), "F"); // 2010
        assert_eq!(name(&[67, 62, 67, 71]), "G"); // 0232
        assert_eq!(name(&[67, 62, 65, 71]), "G7"); // 0212
        assert_eq!(name(&[67, 60, 64, 69]), "Am7"); // 0000
        assert_eq!(name(&[67, 60, 64, 70]), "C7"); // 0001
        assert_eq!(name(&[68, 62, 64, 71]), "E7"); // 1202
        assert_eq!(name(&[70, 62, 65, 70]), "Bb"); // 3211
        assert_eq!(name(&[67, 60, 64, 71]), "Cmaj7"); // 0002
        assert_eq!(name(&[69, 62, 65, 69]), "Dm"); // 2210
        assert_eq!(name(&[67, 62, 69, 74]).as_str(), "Dsus4"); // 0255: G D A D
        assert_eq!(name(&[60, 64, 70]), "C7"); // no fifth
        assert_eq!(name(&[60, 61, 62]), "?");
        assert_eq!(name(&[]), "?");
    }

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
        assert_eq!(r.name.unwrap().to_string(), "C");
    }

    #[test]
    fn silence_has_no_chord() {
        let frame = vec![0.0; 16_384];
        assert!(detect_chord(&frame, 48_000.0, Instrument::Ukulele, 2e-5).is_none());
    }
}
