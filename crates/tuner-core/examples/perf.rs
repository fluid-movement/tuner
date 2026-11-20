//! CPU cost of the analysis per second of audio, per mode, on continuous
//! strumming (a strum every 1.5 s): what the worker and the audio thread
//! spend. Native timings; wasm in a browser is typically 1.5-2.5x slower.
//!
//! cargo run --release -p tuner-core --example perf

use std::time::Instant;
use tuner_core::MonoTuner;
use tuner_core::strum::{Analysis, StrumListener};
use tuner_core::synth::{Rng, chord_strum, render};
use tuner_core::tuning::Instrument;

const RATE: f32 = 48_000.0;

fn strumming(instrument: Instrument, seconds: f32) -> Vec<f32> {
    let mut rng = Rng::new(3);
    let n = instrument.strings().len();
    let mut notes = vec![];
    let mut t = 0.3;
    while t < seconds - 1.0 {
        let frets: Vec<Option<u8>> = (0..n).map(|i| Some((i % 3) as u8)).collect();
        let mut strum = chord_strum(instrument, &frets, 5.0, &mut rng);
        for s in &mut strum {
            s.onset += t;
        }
        notes.extend(strum);
        t += 1.5;
    }
    render(&notes, RATE, seconds, 0.002, &mut rng)
}

fn main() {
    let seconds = 30.0;
    for (label, instrument, analysis) in [
        ("chord (ukulele)", Instrument::Ukulele, Analysis::Chord),
        (
            "strum check, ukulele",
            Instrument::Ukulele,
            Analysis::Tuning,
        ),
        ("strum check, guitar", Instrument::Guitar, Analysis::Tuning),
    ] {
        let x = strumming(instrument, seconds);
        let mut listener = StrumListener::new(RATE, instrument);
        listener.set_analysis(analysis);
        let t = Instant::now();
        let mut reports = 0;
        for c in x.chunks(1024) {
            reports += listener.push(c).is_some() as usize;
        }
        let ms = t.elapsed().as_secs_f64() * 1e3 / f64::from(seconds);
        println!("worker  {label:<22} {ms:6.2} ms per audio second ({reports} reports)");
    }
    let x = strumming(Instrument::Guitar, seconds);
    let mut tuner = MonoTuner::new(RATE, 1024);
    let frame = tuner.frame_size();
    let t = Instant::now();
    for start in (0..x.len() - frame).step_by(1024) {
        tuner.process(&x[start..start + frame]);
    }
    let ms = t.elapsed().as_secs_f64() * 1e3 / f64::from(seconds);
    println!("audio   single-note tracker    {ms:6.2} ms per audio second");
}
