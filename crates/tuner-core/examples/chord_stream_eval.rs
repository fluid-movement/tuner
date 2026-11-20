//! Chord mode on streams that look like real playing: strumming patterns,
//! chord changes, detuned strings, muted strings and non-musical noise, fed
//! through `StrumListener` like the app does.
//!
//! cargo run --release -p tuner-core --example chord_stream_eval [trials]

use tuner_core::strum::{Analysis, StrumListener};
use tuner_core::synth::{PluckedString, Rng, chord_strum, render};
use tuner_core::tuning::Instrument;

const RATE: f32 = 48_000.0;
const PROGRESSION: [[u8; 4]; 4] = [[0, 0, 0, 3], [0, 0, 0, 0], [2, 0, 1, 0], [0, 2, 3, 2]];

struct Outcome {
    strums: usize,
    settled: usize,
    correct: usize,
    false_chords: usize,
}

/// Strums `per_chord` times per chord of the progression, `gap` s apart.
fn run(
    trials: usize,
    gap: f32,
    per_chord: usize,
    detune: f32,
    mute: bool,
    gain: f32,
    seed: u64,
) -> Outcome {
    let mut rng = Rng::new(seed);
    let mut out = Outcome {
        strums: 0,
        settled: 0,
        correct: 0,
        false_chords: 0,
    };
    for _ in 0..trials {
        // (string, note): each note ends when its string is plucked again
        // or muted at the next strum.
        let mut notes: Vec<(usize, PluckedString)> = vec![];
        let mut stops: Vec<f32> = vec![];
        let mut truth: Vec<(f32, Vec<Option<u8>>)> = vec![];
        let mut t = 0.5;
        for chord in PROGRESSION {
            let mut frets: Vec<Option<u8>> = chord.iter().map(|&f| Some(f)).collect();
            if mute {
                frets[(rng.next_u32() % 4) as usize] = None;
            }
            for _ in 0..per_chord {
                let mut strum = chord_strum(Instrument::Ukulele, &frets, detune, &mut rng);
                for (i, s) in strum.iter_mut().enumerate() {
                    s.onset += t;
                    s.level *= gain;
                    // The previous note on this string stops here.
                    for (k, (string, _)) in notes.iter().enumerate() {
                        if *string == i && stops[k] > s.onset {
                            stops[k] = s.onset;
                        }
                    }
                    if s.level > 0.0 {
                        notes.push((i, *s));
                        stops.push(f32::MAX);
                    }
                }
                truth.push((t, frets.clone()));
                t += gap;
            }
        }
        let seconds = t + 1.0;
        let mut x = render(&[], RATE, seconds, 0.002, &mut rng);
        for ((_, note), &stop) in notes.iter().zip(&stops) {
            let y = render(&[*note], RATE, seconds, 0.0, &mut rng);
            let end = ((stop * RATE) as usize).min(x.len());
            let fade = (0.01 * RATE) as usize;
            for i in 0..end.min(x.len()) {
                x[i] += y[i];
            }
            for i in end..(end + fade).min(x.len()) {
                x[i] += y[i] * (1.0 - (i - end) as f32 / fade as f32);
            }
        }
        let mut listener = StrumListener::new(RATE, Instrument::Ukulele);
        listener.set_analysis(Analysis::Chord);
        // Last settled report per onset.
        let mut settled: Vec<(f64, Vec<Option<u8>>)> = vec![];
        for c in x.chunks(1024) {
            if let Some(r) = listener.push(c)
                && r.settled
            {
                if std::env::var("ONSET_DEBUG").is_ok() && r.chord.is_none() {
                    eprintln!("no chord at {:.2}", r.onset_seconds);
                }
                if let Some(chord) = r.chord {
                    settled.push((r.onset_seconds, chord.frets));
                }
            }
        }
        if std::env::var("ONSET_DEBUG").is_ok() {
            eprintln!(
                "truth {:?}",
                truth
                    .iter()
                    .map(|t| (t.0 * 100.0).round() / 100.0)
                    .collect::<Vec<_>>()
            );
            eprintln!(
                "got   {:?}",
                settled
                    .iter()
                    .map(|t| (t.0 * 100.0).round() / 100.0)
                    .collect::<Vec<_>>()
            );
        }
        out.strums += truth.len();
        for (onset, frets) in &truth {
            if let Some((_, got)) = settled
                .iter()
                .find(|(o, _)| (*o as f32 - onset).abs() < 0.06)
            {
                out.settled += 1;
                let notes = |f: &[Option<u8>]| {
                    let mut v: Vec<i32> = f
                        .iter()
                        .zip(Instrument::Ukulele.strings())
                        .filter_map(|(f, s)| f.map(|f| s.midi + i32::from(f)))
                        .collect();
                    v.sort();
                    v.dedup();
                    v
                };
                let ok = notes(got) == notes(frets);
                if !ok && std::env::var("CHORD_DEBUG").is_ok() {
                    eprintln!("  at {onset:.2}: {frets:?} -> {got:?}");
                }
                out.correct += ok as usize;
            }
        }
        out.false_chords += settled
            .iter()
            .filter(|(o, _)| !truth.iter().any(|(t, _)| (*o as f32 - t).abs() < 0.06))
            .count();
    }
    out
}

fn noise_reports(seconds: f32, seed: u64) -> usize {
    // Bursts of noise (claps, knocks, speech-like hiss) every 0.7 s.
    let mut rng = Rng::new(seed);
    let n = (seconds * RATE) as usize;
    let mut x = render(&[], RATE, seconds, 0.001, &mut rng);
    let mut t = 0.4;
    while t < seconds - 0.5 {
        let start = (t * RATE) as usize;
        let len = (rng.uniform(0.05, 0.3) * RATE) as usize;
        let amp = rng.uniform(0.05, 0.3);
        for (k, y) in x[start..(start + len).min(n)].iter_mut().enumerate() {
            let env = (-(k as f32) / (0.3 * len as f32)).exp();
            *y += amp * env * rng.normal();
        }
        t += 0.7;
    }
    let mut listener = StrumListener::new(RATE, Instrument::Ukulele);
    listener.set_analysis(Analysis::Chord);
    let chords: Vec<_> = x
        .chunks(1024)
        .filter_map(|c| listener.push(c))
        .filter(|r| r.settled)
        .filter_map(|r| r.chord)
        .collect();
    if std::env::var("CHORD_DEBUG").is_ok() {
        let mut e: Vec<f32> = chords.iter().map(|c| c.explained).collect();
        e.sort_by(f32::total_cmp);
        eprintln!("noise explained: {e:?}");
        let strings: Vec<usize> = chords
            .iter()
            .map(|c| c.frets.iter().flatten().count())
            .collect();
        eprintln!("noise strings: {strings:?}");
    }
    chords.len()
}

/// Chords reported on `x` (settled), for sounds that aren't ukulele.
fn chords_in(x: &[f32]) -> Vec<Vec<Option<u8>>> {
    let mut listener = StrumListener::new(RATE, Instrument::Ukulele);
    listener.set_analysis(Analysis::Chord);
    x.chunks(1024)
        .filter_map(|c| listener.push(c))
        .filter(|r| r.settled)
        .filter_map(|r| r.chord.map(|c| c.frets))
        .collect()
}

/// Everyday sounds while the ukulele isn't played.
fn everyday(kind: &str, seconds: f32, seed: u64) -> Vec<f32> {
    let mut rng = Rng::new(seed);
    let n = (seconds * RATE) as usize;
    let mut x = render(&[], RATE, seconds, 0.0005, &mut rng);
    let tau = std::f32::consts::TAU;
    match kind {
        // Voiced syllables: gliding f0 with harmonics shaped by two
        // formants, separated by pauses and hiss (fricatives).
        "talking" => {
            let mut t = 0.3;
            while t < seconds - 0.8 {
                let len = rng.uniform(0.15, 0.6);
                let f_start = rng.uniform(90.0, 260.0);
                let f_end = f_start * rng.uniform(0.75, 1.3);
                let (f1, f2) = (rng.uniform(300.0, 850.0), rng.uniform(900.0, 2300.0));
                let amp = rng.uniform(0.03, 0.2);
                let start = (t * RATE) as usize;
                let m = (len * RATE) as usize;
                let mut phase = [0.0f32; 30];
                for k in 0..m.min(n - start) {
                    let u = k as f32 / m as f32;
                    let f0 =
                        f_start + (f_end - f_start) * u + 3.0 * (tau * 5.5 * k as f32 / RATE).sin();
                    let env = (u * 12.0).min(1.0) * ((1.0 - u) * 8.0).min(1.0);
                    let mut y = 0.0;
                    for (h, ph) in phase.iter_mut().enumerate() {
                        let hz = f0 * (h + 1) as f32;
                        if hz > 5000.0 {
                            break;
                        }
                        *ph = (*ph + tau * hz / RATE) % tau;
                        let formant = (-((hz - f1) / 150.0).powi(2)).exp()
                            + 0.6 * (-((hz - f2) / 250.0).powi(2)).exp()
                            + 0.05;
                        y += formant * ph.sin() / (h + 1) as f32;
                    }
                    x[start + k] += amp * env * y;
                }
                t += len;
                if rng.uniform(0.0, 1.0) < 0.4 {
                    let s0 = (t * RATE) as usize;
                    let hiss = (rng.uniform(0.05, 0.15) * RATE) as usize;
                    for y in x[s0..(s0 + hiss).min(n)].iter_mut() {
                        *y += 0.03 * rng.normal();
                    }
                }
                t += rng.uniform(0.08, 0.5);
            }
        }
        "whistling" => {
            let mut t = 0.3;
            let mut phase = 0.0f32;
            while t < seconds - 1.0 {
                let len = rng.uniform(0.2, 0.8);
                let (a, b) = (rng.uniform(700.0, 1800.0), rng.uniform(700.0, 1800.0));
                let start = (t * RATE) as usize;
                let m = (len * RATE) as usize;
                for k in 0..m.min(n - start) {
                    let u = k as f32 / m as f32;
                    phase = (phase + tau * (a + (b - a) * u) / RATE) % tau;
                    let env = (u * 20.0).min(1.0) * ((1.0 - u) * 20.0).min(1.0);
                    x[start + k] += 0.08 * env * phase.sin();
                }
                t += len + rng.uniform(0.05, 0.4);
            }
        }
        // Mains hum with harmonics, a fan (steady noise), switching on and off.
        "hum and fan" => {
            for (i, y) in x.iter_mut().enumerate() {
                let t = i as f32 / RATE;
                let on = (t / 3.0) as usize % 2 == 1;
                let hum: f32 = (1..8)
                    .map(|h| (tau * 50.0 * h as f32 * t).sin() / h as f32)
                    .sum();
                *y += if on { 0.02 * hum } else { 0.004 * hum } + 0.004 * rng.normal();
            }
        }
        // Quiet room: taps, clicks, a knock now and then.
        "quiet room taps" => {
            let mut t = 0.3;
            while t < seconds - 0.2 {
                let start = (t * RATE) as usize;
                let len = (rng.uniform(0.001, 0.02) * RATE) as usize;
                let amp = rng.uniform(0.05, 0.5);
                let ring = rng.uniform(200.0, 3000.0);
                for k in 0..(len * 8).min(n - start) {
                    let env = (-(k as f32) / len as f32).exp();
                    x[start + k] +=
                        amp * env * (0.5 * rng.normal() + (tau * ring * k as f32 / RATE).sin());
                }
                t += rng.uniform(0.2, 1.5);
            }
        }
        // Humming one sustained note now and then (a single pitch).
        "humming" => {
            let mut t = 0.3;
            while t < seconds - 2.0 {
                let f0 = rng.uniform(110.0, 330.0);
                let len = rng.uniform(0.6, 1.5);
                let start = (t * RATE) as usize;
                let m = (len * RATE) as usize;
                for k in 0..m.min(n - start) {
                    let u = k as f32 / m as f32;
                    let env = (u * 10.0).min(1.0) * ((1.0 - u) * 10.0).min(1.0);
                    let ph = tau * f0 * k as f32 / RATE;
                    let y: f32 = (1..10)
                        .map(|h| (ph * h as f32).sin() / (h * h) as f32)
                        .sum();
                    x[start + k] += 0.1 * env * y;
                }
                t += len + rng.uniform(0.3, 1.0);
            }
        }
        _ => unreachable!(),
    }
    x
}

fn main() {
    let trials: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);
    for (label, gap, per, detune, mute) in [
        ("slow strums (2 s), in tune", 2.0, 1, 3.0, false),
        ("strumming 0.5 s", 0.5, 4, 3.0, false),
        ("strumming 0.3 s", 0.3, 4, 3.0, false),
        ("detuned ±20 c, 1 s", 1.0, 2, 20.0, false),
        ("one string muted, 1 s", 1.0, 2, 3.0, true),
        ("quiet strums (-20 dB), 1 s", 1.0, 2, 3.0, false),
        ("very quiet (-30 dB), 1 s", 1.0, 2, 3.0, false),
    ] {
        let gain = if label.starts_with("quiet") {
            0.1
        } else if label.starts_with("very quiet") {
            0.032
        } else {
            1.0
        };
        // CASE=<text> runs only the scenarios whose label contains it.
        if std::env::var("CASE").is_ok_and(|c| !label.contains(c.as_str())) {
            continue;
        }
        let o = run(trials, gap, per, detune, mute, gain, 11);
        println!(
            "{label:<28} settled {:5.1}%  notes right {:5.1}% of strums  false chords {}",
            100.0 * o.settled as f64 / o.strums as f64,
            100.0 * o.correct as f64 / o.strums as f64,
            o.false_chords
        );
    }
    println!(
        "noise bursts (60 s): {} chords reported",
        noise_reports(60.0, 5)
    );
    for kind in [
        "talking",
        "whistling",
        "hum and fan",
        "quiet room taps",
        "humming",
    ] {
        // Five independent minutes of each.
        let chords: Vec<_> = (21..26)
            .flat_map(|seed| chords_in(&everyday(kind, 60.0, seed)))
            .collect();
        println!(
            "{kind} (5 min): {} chords reported {:?}",
            chords.len(),
            chords.iter().take(4).collect::<Vec<_>>()
        );
    }
}
