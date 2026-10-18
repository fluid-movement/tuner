//! Can a strum tell played strings from muted ones? Prints the presence cue
//! (`prominence_db`) for played and muted strings with clean partials, and
//! how often `present` gets it right. Guitar B3 and E4 have no clean partials
//! and are not judged.
//!
//! cargo run --release -p tuner-core --example presence_eval [count]

use tuner_core::poly::{PolyConfig, estimate_strum};
use tuner_core::synth::{PluckedString, Rng, StrumOptions, highpass, random_strum, render};
use tuner_core::tuning::Instrument;

const RATE: f32 = 48_000.0;
const POST: usize = 32_768;

fn q(v: &mut [f32], p: f32) -> f32 {
    if v.is_empty() {
        return f32::NAN;
    }
    v.sort_by(f32::total_cmp);
    v[((v.len() - 1) as f32 * p).round() as usize]
}

fn main() {
    let count: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(200);
    for instrument in [Instrument::Guitar, Instrument::Ukulele] {
        for previous in [true, false] {
            let n = instrument.strings().len();
            // [string][played?] -> cue values
            let mut cues: Vec<[Vec<f32>; 2]> = (0..n).map(|_| [vec![], vec![]]).collect();
            let mut rng = Rng::new(42 + previous as u64);
            let opts = StrumOptions {
                max_detune_cents: 10.0,
                ..StrumOptions::for_instrument(instrument)
            };
            for _ in 0..count {
                let muted = rng.next_u32() as usize % n;
                let mut notes: Vec<PluckedString> = Vec::new();
                let t0 = if previous { 3.0 } else { 0.5 };
                if previous {
                    // A full strum 2.5 s earlier, still ringing.
                    let mut prev = random_strum(instrument, &opts, &mut rng);
                    for s in &mut prev {
                        s.onset += 0.5;
                    }
                    notes.extend(prev);
                }
                let mut strum = random_strum(instrument, &opts, &mut rng);
                for s in &mut strum {
                    s.onset += t0;
                }
                strum[muted].level = 0.0;
                // A muted string is also silenced if it was ringing: drop the
                // previous strum's note for it from the onset on (approximated
                // by not rendering it at all when there was a previous strum).
                if previous {
                    notes.remove(muted);
                }
                let last = strum.iter().map(|s| s.onset).fold(0.0, f32::max);
                notes.extend(strum);
                let mut x = render(&notes, RATE, last + 1.0, 0.002, &mut rng);
                highpass(&mut x, 100.0, RATE);
                let start = ((last + 0.08) * RATE) as usize;
                let post = &x[start..start + POST];
                let est = estimate_strum(post, instrument, &PolyConfig::new(RATE, instrument));
                for (i, e) in est.iter().enumerate() {
                    let Some(e) = e else { continue };
                    if let Some(v) = e.prominence_db {
                        cues[i][(i != muted) as usize].push(v);
                    }
                }
            }
            println!(
                "\n{instrument:?}, {}:",
                if previous {
                    "previous strum still ringing"
                } else {
                    "from silence"
                }
            );
            println!(
                "  string cue      played min / p05      muted p95 / max     at 20 dB: muted caught, played missed"
            );
            for (t, [muted, played]) in instrument.strings().iter().zip(&mut cues) {
                let cue = if t.name == "B3" || (t.name == "E4" && instrument == Instrument::Guitar)
                {
                    "unique"
                } else {
                    "clean"
                };
                println!(
                    "  {:<4} {:<6} {:>8.1} / {:>6.1}       {:>6.1} / {:>6.1}   (n={}/{})",
                    t.name,
                    cue,
                    q(played, 0.0),
                    q(played, 0.05),
                    q(muted, 0.95),
                    q(muted, 1.0),
                    played.len(),
                    muted.len()
                );
                if cue == "clean" {
                    let caught = muted.iter().filter(|&&v| v < 20.0).count();
                    let missed = played.iter().filter(|&&v| v < 20.0).count();
                    println!(
                        "         -> muted caught {caught}/{}, played missed {missed}/{}",
                        muted.len(),
                        played.len()
                    );
                }
            }
        }
    }
}
