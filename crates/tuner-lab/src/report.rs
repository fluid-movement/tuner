//! Markdown reports for one take and across takes.

use crate::analyze::{TakeReport, median};
use crate::segment::Kind;
use std::fmt::Write;

fn opt(v: Option<f32>, digits: usize) -> String {
    v.map_or("–".into(), |v| format!("{v:+.digits$}"))
}

pub fn take_markdown(r: &TakeReport) -> String {
    let mut s = String::new();
    let plucks = r.events.iter().filter(|e| e.kind == Kind::Pluck).count();
    let _ = writeln!(
        s,
        "## {} ({}, {:.1} s, {} Hz): {} events, {} plucks, {} strums\n",
        r.file,
        r.instrument,
        r.seconds,
        r.sample_rate,
        r.events.len(),
        plucks,
        r.events.len() - plucks
    );
    let _ = writeln!(
        s,
        "| event | onset | kind | sounding strings | clarity |\n|---|---|---|---|---|"
    );
    for (i, e) in r.events.iter().enumerate() {
        let _ = writeln!(
            s,
            "| {} | {:.2} s | {:?} | {} | {:.2} |",
            i + 1,
            e.onset,
            e.kind,
            e.sounding_strings,
            e.clarity
        );
    }

    if !r.plucks.is_empty() {
        let _ = writeln!(
            s,
            "\n**Single strings** (what the tuner shows)\n\n| string | truth | vs target | first reading | shown in first 2 s | spread | drift |\n|---|---|---|---|---|---|---|"
        );
        for p in &r.plucks {
            let _ = writeln!(
                s,
                "| {} | {:.2} Hz | {:+.1} c | {} | {:.0}% | {:.1} c | {} c |",
                p.string,
                p.frequency,
                p.cents,
                p.first_reading_ms
                    .map_or("never".into(), |ms| format!("{ms:.0} ms")),
                p.shown_share * 100.0,
                p.spread_cents,
                opt(p.drift_cents, 1)
            );
        }
    }

    for (k, strum) in r.strums.iter().enumerate() {
        let _ = writeln!(
            s,
            "\n**Strum {}** at {:.2} s ({:.2} s window)\n\n| string | truth | estimate | error | verdicts |\n|---|---|---|---|---|",
            k + 1,
            strum.onset,
            strum.window_seconds
        );
        for st in &strum.strings {
            let agree = match st.agree {
                Some(true) => "agree",
                Some(false) => "**differ**",
                None => "–",
            };
            let _ = writeln!(
                s,
                "| {} | {} c | {} c | {} c | {} |",
                st.string,
                opt(st.truth_cents, 1),
                opt(st.estimate_cents, 1),
                opt(st.error_cents, 2),
                agree
            );
        }
    }
    for n in &r.notes {
        let _ = writeln!(s, "\n> {n}");
    }
    s
}

pub fn summary_markdown(reports: &[TakeReport]) -> String {
    let mut s = String::from("## Summary\n\n");
    for instrument in ["guitar", "ukulele"] {
        let takes: Vec<&TakeReport> = reports
            .iter()
            .filter(|r| r.instrument == instrument)
            .collect();
        if takes.is_empty() {
            continue;
        }
        let plucks: Vec<_> = takes.iter().flat_map(|r| &r.plucks).collect();
        let strums: Vec<_> = takes.iter().flat_map(|r| &r.strums).collect();
        let _ = writeln!(
            s,
            "### {instrument}: {} takes, {} plucks, {} strums\n",
            takes.len(),
            plucks.len(),
            strums.len()
        );
        if !plucks.is_empty() {
            let latency: Vec<f32> = plucks.iter().filter_map(|p| p.first_reading_ms).collect();
            let never = plucks.len() - latency.len();
            let share: Vec<f32> = plucks.iter().map(|p| p.shown_share * 100.0).collect();
            let spread: Vec<f32> = plucks.iter().map(|p| p.spread_cents).collect();
            let drift: Vec<f32> = plucks
                .iter()
                .filter_map(|p| p.drift_cents.map(f32::abs))
                .collect();
            let _ = writeln!(
                s,
                "Single strings: first reading median {} ms (worst {} ms, never {never}); shown in first 2 s median {:.0}% (worst {:.0}%); spread median {:.1} c (worst {:.1} c); |drift| median {:.1} c (worst {:.1} c)\n",
                median(latency.clone()).map_or("–".into(), |v| format!("{v:.0}")),
                latency
                    .iter()
                    .copied()
                    .fold(f32::MIN, f32::max)
                    .max(0.0)
                    .round(),
                median(share.clone()).unwrap_or(0.0),
                share.iter().copied().fold(f32::MAX, f32::min),
                median(spread.clone()).unwrap_or(0.0),
                spread.iter().copied().fold(0.0, f32::max),
                median(drift.clone()).unwrap_or(0.0),
                drift.iter().copied().fold(0.0, f32::max),
            );
        }
        if !strums.is_empty() {
            let _ = writeln!(
                s,
                "| string | strum error p50 | p90 | max | verdicts agree |\n|---|---|---|---|---|"
            );
            let names: Vec<&str> = strums[0].strings.iter().map(|st| st.string).collect();
            for (i, name) in names.iter().enumerate() {
                let mut errors: Vec<f32> = strums
                    .iter()
                    .filter_map(|st| st.strings[i].error_cents.map(f32::abs))
                    .collect();
                errors.sort_by(f32::total_cmp);
                let q = |p: f32| {
                    errors
                        .get(((errors.len().max(1) - 1) as f32 * p).round() as usize)
                        .map_or("–".into(), |v| format!("{v:.2}"))
                };
                let judged: Vec<bool> =
                    strums.iter().filter_map(|st| st.strings[i].agree).collect();
                let agree = judged.iter().filter(|&&a| a).count();
                let _ = writeln!(
                    s,
                    "| {name} | {} c | {} c | {} c | {agree}/{} |",
                    q(0.5),
                    q(0.9),
                    q(1.0),
                    judged.len()
                );
            }
            s.push('\n');
        }
    }
    s
}
