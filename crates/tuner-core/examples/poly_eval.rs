//! Evaluates polyphonic estimators on synthetic strums.
//!
//! cargo run --release -p tuner-core --example poly_eval [method|all] [count] [scenario-filter]
//!
//! Set POLY_DEBUG=1 to print strums whose error exceeds 5 cents.

use std::time::Instant;
use tuner_core::poly::{PolyConfig, StringEstimate, estimate_comb, estimate_resolved};
use tuner_core::synth::{PluckedString, Rng, StrumOptions, random_strum, render};
use tuner_core::tuning::{Instrument, cents_between};

const RATE: f32 = 48_000.0;

type Estimator = fn(&[f32], Instrument, &PolyConfig) -> Vec<Option<StringEstimate>>;

const METHODS: [(&str, Estimator); 2] = [("comb", estimate_comb), ("resolved", estimate_resolved)];

struct Scenario {
    name: &'static str,
    frame: usize,
    noise_rms: f32,
    /// Detuning range in cents applied to every string.
    detune: f32,
    polarization: bool,
    /// Mute one random string.
    mute_one: bool,
}

const SCENARIOS: [Scenario; 6] = [
    Scenario {
        name: "random ±50",
        frame: 32_768,
        noise_rms: 0.002,
        detune: 50.0,
        polarization: false,
        mute_one: false,
    },
    Scenario {
        name: "near ±10",
        frame: 32_768,
        noise_rms: 0.002,
        detune: 10.0,
        polarization: false,
        mute_one: false,
    },
    Scenario {
        name: "near ±10, noisy",
        frame: 32_768,
        noise_rms: 0.012,
        detune: 10.0,
        polarization: false,
        mute_one: false,
    },
    Scenario {
        name: "near ±10, short window",
        frame: 16_384,
        noise_rms: 0.002,
        detune: 10.0,
        polarization: false,
        mute_one: false,
    },
    Scenario {
        name: "near ±10, beating partials",
        frame: 32_768,
        noise_rms: 0.002,
        detune: 10.0,
        polarization: true,
        mute_one: false,
    },
    Scenario {
        name: "near ±10, one string muted",
        frame: 32_768,
        noise_rms: 0.002,
        detune: 10.0,
        polarization: false,
        mute_one: true,
    },
];

/// In tune within 5 cents, else flat/sharp.
fn verdict(c: f32) -> i8 {
    if c.abs() <= 5.0 { 0 } else { c.signum() as i8 }
}

#[derive(Default)]
struct Row {
    errors: Vec<f32>,
    misses: usize,
    wrong: usize,
    judged: usize,
}

struct Outcome {
    rows: Vec<Row>,
    ms: f64,
    /// Reported levels of muted strings and of played strings.
    muted_levels: Vec<f32>,
    played_levels: Vec<f32>,
}

fn run(instrument: Instrument, method: Estimator, count: usize, sc: &Scenario) -> Outcome {
    let n = instrument.strings().len();
    let mut out = Outcome {
        rows: (0..n).map(|_| Row::default()).collect(),
        ms: 0.0,
        muted_levels: vec![],
        played_levels: vec![],
    };
    let config = PolyConfig::new(RATE, instrument);
    let opts = StrumOptions {
        max_detune_cents: sc.detune,
        ..StrumOptions::for_instrument(instrument)
    };
    let mut rng = Rng::new(sc.name.len() as u64 * 7919 + instrument as u64);
    let debug = std::env::var("POLY_DEBUG").is_ok();
    let mut elapsed = 0.0;
    for _ in 0..count {
        let mut strum: Vec<PluckedString> = random_strum(instrument, &opts, &mut rng);
        if sc.polarization {
            for s in &mut strum {
                s.polarization_cents = rng.uniform(0.5, 4.0);
            }
        }
        let muted = sc.mute_one.then(|| (rng.next_u32() as usize) % n);
        if let Some(m) = muted {
            strum[m].level = 0.0;
        }
        let last = strum.iter().map(|s| s.onset).fold(0.0, f32::max);
        let start = ((last + 0.08) * RATE) as usize;
        let seconds = (start + sc.frame) as f32 / RATE + 0.01;
        let signal = render(&strum, RATE, seconds, sc.noise_rms, &mut rng);
        let t = Instant::now();
        let est = method(&signal[start..start + sc.frame], instrument, &config);
        elapsed += t.elapsed().as_secs_f64();

        for (i, (e, s)) in est.iter().zip(&strum).enumerate() {
            let row = &mut out.rows[i];
            if Some(i) == muted {
                out.muted_levels
                    .push(e.map_or(f32::NEG_INFINITY, |e| e.level_db));
                continue;
            }
            let truth = cents_between(s.frequency, instrument.strings()[i].frequency);
            let Some(e) = e else {
                row.misses += 1;
                continue;
            };
            out.played_levels.push(e.level_db);
            let err = cents_between(e.frequency, s.frequency);
            if debug && err.abs() > 5.0 {
                let truths: Vec<String> = strum
                    .iter()
                    .zip(instrument.strings())
                    .map(|(s, t)| {
                        format!(
                            "{}{:+.1}@{:.3}",
                            t.name,
                            cents_between(s.frequency, t.frequency),
                            s.level
                        )
                    })
                    .collect();
                eprintln!(
                    "  outlier {} est {:+.1} truth {truth:+.1} | {}",
                    instrument.strings()[i].name,
                    e.cents,
                    truths.join(" ")
                );
            }
            row.errors.push(err);
            // Truths within 1.5 cents of the in-tune boundary are ambiguous.
            if (truth.abs() - 5.0).abs() > 1.5 {
                row.judged += 1;
                row.wrong += (verdict(e.cents) != verdict(truth)) as usize;
            }
        }
    }
    out.ms = elapsed * 1e3 / count as f64;
    out
}

fn quantile(v: &mut [f32], p: f32) -> f32 {
    if v.is_empty() {
        return f32::NAN;
    }
    v.sort_by(|a, b| a.total_cmp(b));
    v[((v.len() - 1) as f32 * p).round() as usize]
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let method = args.get(1).map(String::as_str).unwrap_or("all");
    let count: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(200);
    let filter = args.get(3).map(String::as_str).unwrap_or("");

    for instrument in [Instrument::Guitar, Instrument::Ukulele] {
        for sc in SCENARIOS.iter().filter(|s| s.name.contains(filter)) {
            for (name, est) in METHODS
                .iter()
                .filter(|(n, _)| method == "all" || *n == method)
            {
                let mut o = run(instrument, *est, count, sc);
                println!(
                    "\n[{name}] {instrument:?} — {}  ({:.2} ms/analysis, {} strums)",
                    sc.name, o.ms, count
                );
                println!("  string  |err| p50    p90    max  miss  wrong verdict");
                for (t, row) in instrument.strings().iter().zip(&mut o.rows) {
                    let mut abs: Vec<f32> = row.errors.iter().map(|e| e.abs()).collect();
                    println!(
                        "  {:<6} {:>8.2} {:>6.2} {:>6.1} {:>5} {:>6}/{}",
                        t.name,
                        quantile(&mut abs, 0.5),
                        quantile(&mut abs, 0.9),
                        quantile(&mut abs, 1.0),
                        row.misses,
                        row.wrong,
                        row.judged
                    );
                }
                if sc.mute_one {
                    println!(
                        "  level dB: muted strings p50 {:.1} max {:.1}; played strings min {:.1} p05 {:.1}",
                        quantile(&mut o.muted_levels, 0.5),
                        quantile(&mut o.muted_levels, 1.0),
                        quantile(&mut o.played_levels, 0.0),
                        quantile(&mut o.played_levels, 0.05)
                    );
                }
            }
        }
    }
}
