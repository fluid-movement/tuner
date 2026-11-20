//! Evaluates ukulele chord recognition on synthetic strums of common chords.
//!
//! cargo run --release -p tuner-core --example chord_eval [count-per-chord] [window...]
//!
//! For each window length (samples at 48 kHz, default 4096 8192 16384) prints
//! how often the tab and the name are right. Set CHORD_DEBUG=1 to print
//! every miss.

use std::time::Instant;
use tuner_core::chord::{detect_chord, name_chord};
use tuner_core::synth::{Rng, chord_strum, highpass, render};
use tuner_core::tuning::Instrument;

const RATE: f32 = 48_000.0;

/// Common ukulele chords, frets in string order G C E A.
const CHORDS: [(&str, [u8; 4]); 30] = [
    ("C", [0, 0, 0, 3]),
    ("Am", [2, 0, 0, 0]),
    ("F", [2, 0, 1, 0]),
    ("G", [0, 2, 3, 2]),
    ("G7", [0, 2, 1, 2]),
    ("Am7", [0, 0, 0, 0]),
    ("C7", [0, 0, 0, 1]),
    ("Cmaj7", [0, 0, 0, 2]),
    ("Dm", [2, 2, 1, 0]),
    ("D", [2, 2, 2, 0]),
    ("D7", [2, 2, 2, 3]),
    ("Em", [0, 4, 3, 2]),
    ("E7", [1, 2, 0, 2]),
    ("A", [2, 1, 0, 0]),
    ("A7", [0, 1, 0, 0]),
    ("Bb", [3, 2, 1, 1]),
    ("Bm", [4, 2, 2, 2]),
    ("B7", [2, 3, 2, 2]),
    ("Dm7", [2, 2, 1, 3]),
    ("Fmaj7", [2, 4, 1, 3]),
    ("Gm", [0, 2, 3, 1]),
    ("Gmaj7", [0, 2, 2, 2]),
    ("Csus4", [0, 0, 1, 3]),
    ("Dsus2", [2, 2, 0, 0]),
    ("Eb", [0, 3, 3, 1]),
    ("E", [4, 4, 4, 2]),
    ("F#m", [2, 1, 2, 0]),
    ("Cm", [0, 3, 3, 3]),
    ("Bdim", [4, 2, 1, 2]),
    ("Caug", [1, 0, 0, 3]),
];

fn main() {
    let mut args = std::env::args().skip(1);
    let count: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(20);
    let mut windows: Vec<usize> = args.filter_map(|s| s.parse().ok()).collect();
    if windows.is_empty() {
        windows = vec![4096, 8192, 16_384];
    }
    let debug = std::env::var("CHORD_DEBUG").is_ok();
    for &window in &windows {
        for (label, noise, low_cut) in [
            ("clean", 0.002f32, false),
            ("noisy", 0.01, false),
            ("phone mic", 0.004, true),
        ] {
            let mut rng = Rng::new(window as u64 + noise.to_bits() as u64);
            let (mut tabs, mut sets, mut names, mut total) = (0, 0, 0, 0);
            let mut elapsed = 0.0;
            for (name, frets) in CHORDS {
                assert_eq!(
                    name_chord(
                        &frets
                            .iter()
                            .zip(Instrument::Ukulele.strings())
                            .map(|(&f, s)| s.midi + i32::from(f))
                            .collect::<Vec<_>>()
                    )
                    .map(|n| n.to_string())
                    .as_deref(),
                    Some(name),
                    "reference name"
                );
                let frets: Vec<Option<u8>> = frets.iter().map(|&f| Some(f)).collect();
                for _ in 0..count {
                    let strum = chord_strum(Instrument::Ukulele, &frets, 8.0, &mut rng);
                    let last = strum.iter().map(|s| s.onset).fold(0.0, f32::max);
                    let start = ((last + 0.06) * RATE) as usize;
                    let seconds = (start + window) as f32 / RATE + 0.01;
                    let mut signal = render(&strum, RATE, seconds, noise, &mut rng);
                    if low_cut {
                        highpass(&mut signal, 150.0, RATE);
                    }
                    let t = Instant::now();
                    let r = detect_chord(
                        &signal[start..start + window],
                        RATE,
                        Instrument::Ukulele,
                        2e-5,
                    );
                    elapsed += t.elapsed().as_secs_f64();
                    total += 1;
                    let got_name = r.as_ref().and_then(|r| r.name).map(|n| n.to_string());
                    let tab_ok = r.as_ref().is_some_and(|r| r.frets == frets);
                    tabs += tab_ok as usize;
                    let notes = |m: &[Option<i32>]| {
                        let mut v: Vec<i32> = m.iter().flatten().copied().collect();
                        v.sort();
                        v.dedup();
                        v
                    };
                    let truth: Vec<Option<i32>> = frets
                        .iter()
                        .zip(Instrument::Ukulele.strings())
                        .map(|(f, s)| f.map(|f| s.midi + i32::from(f)))
                        .collect();
                    sets += r.as_ref().is_some_and(|r| notes(&r.midi) == notes(&truth)) as usize;
                    names += (got_name.as_deref() == Some(name)) as usize;
                    if debug && !tab_ok {
                        eprintln!(
                            "  {window} {label}: {name} {:?} -> {:?} {:?}",
                            frets,
                            r.as_ref().map(|r| &r.frets),
                            got_name
                        );
                    }
                }
            }
            println!(
                "window {window:>5} {label:<9}: tab {:5.1}%  notes {:5.1}%  name {:5.1}%  ({:.2} ms/analysis, n={total})",
                100.0 * tabs as f64 / total as f64,
                100.0 * sets as f64 / total as f64,
                100.0 * names as f64 / total as f64,
                elapsed * 1e3 / total as f64
            );
        }
    }
}
