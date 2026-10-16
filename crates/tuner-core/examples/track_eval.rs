//! How a single plucked string looks on the tuner display: share of the
//! first two seconds with a reading, error of what is shown, and time to the
//! first reading, for every guitar and ukulele string at several input levels.
//!
//! cargo run --release -p tuner-core --example track_eval

use tuner_core::MonoTuner;
use tuner_core::synth::{PluckedString, Rng, highpass, render};
use tuner_core::tuning::{Instrument, cents_between};

const RATE: f32 = 48_000.0;
const HOP: usize = 1024;

fn main() {
    // (label, pluck peak amplitude, white noise RMS)
    let conditions = [
        ("loud: peak −20 dBFS, noise −60 dBFS", 0.1, 0.001),
        ("quiet: peak −34 dBFS, noise −60 dBFS", 0.02, 0.001),
        ("quiet + noisy room: peak −34, noise −50", 0.02, 0.003),
        ("very quiet: peak −40, noise −50", 0.01, 0.003),
    ];
    for (label, level, noise) in conditions {
        println!("\n{label}  (phone-like 120 Hz low cut)");
        println!("  string  shown 0–2 s   |err| max   first reading");
        for inst in [Instrument::Guitar, Instrument::Ukulele] {
            for t in inst.strings() {
                let f = t.frequency;
                let s = PluckedString {
                    frequency: f,
                    level,
                    onset: 0.1,
                    inharmonicity: if inst == Instrument::Guitar {
                        1e-4
                    } else {
                        2e-5
                    },
                    decay: 1.2 * (110.0 / f).powf(0.4),
                    rolloff: 0.9,
                    pluck_position: 0.15,
                    polarization_cents: 1.5,
                };
                let mut x = render(&[s], RATE, 2.2, noise, &mut Rng::new(t.midi as u64));
                highpass(&mut x, 120.0, RATE);
                let mut tuner = MonoTuner::new(RATE, HOP);
                let n = tuner.frame_size();
                let (mut frames, mut shown, mut worst, mut first) = (0, 0, 0.0f32, None);
                let mut i = 0;
                while i + n <= x.len() {
                    let out = tuner.process(&x[i..i + n]);
                    let end = (i + n) as f32 / RATE;
                    if end > 0.1 {
                        frames += 1;
                        if let Some(hz) = out.frequency {
                            shown += 1;
                            worst = worst.max(cents_between(hz, f).abs());
                            first.get_or_insert(end - 0.1);
                        }
                    }
                    i += HOP;
                }
                println!(
                    "  {:<3} {:<3} {:>8.0}%   {:>9.1}c   {}",
                    if inst == Instrument::Guitar {
                        "gtr"
                    } else {
                        "uke"
                    },
                    t.name,
                    100.0 * shown as f32 / frames as f32,
                    worst,
                    first.map_or("never".into(), |s| format!("{:.0} ms", s * 1e3))
                );
            }
        }
    }
}
