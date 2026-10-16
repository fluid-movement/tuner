use std::path::{Path, PathBuf};
use std::process::ExitCode;
use tuner_core::tuning::Instrument;
use tuner_lab::analyze::{TakeReport, analyze_file, instrument_name};
use tuner_lab::{PROTOCOL, audio, demo, report};

const USAGE: &str = "\
Usage:
  tuner-lab [--json] [DIR|FILE]...   analyse recordings (default: recordings/)
  tuner-lab demo DIR                 write synthetic example takes to DIR";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        println!("{USAGE}\n\n{PROTOCOL}");
        return ExitCode::SUCCESS;
    }
    if args.first().map(String::as_str) == Some("demo") {
        return match args.get(1) {
            Some(dir) => write_demo(Path::new(dir)),
            None => {
                eprintln!("{USAGE}");
                ExitCode::FAILURE
            }
        };
    }
    let json = args.iter().any(|a| a == "--json");
    let mut inputs: Vec<PathBuf> = args
        .iter()
        .filter(|a| !a.starts_with("--"))
        .map(PathBuf::from)
        .collect();
    if inputs.is_empty() {
        inputs.push(PathBuf::from("recordings"));
    }

    let files = collect(&inputs);
    if files.is_empty() {
        eprintln!("No WAV or FLAC files in {}.\n\n{PROTOCOL}", show(&inputs));
        return ExitCode::FAILURE;
    }

    let mut reports: Vec<TakeReport> = Vec::new();
    let mut failed = false;
    for f in &files {
        match analyze_file(f) {
            Ok(r) => reports.push(r),
            Err(e) => {
                eprintln!("skipped {e}");
                failed = true;
            }
        }
    }
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&reports).expect("serializable")
        );
    } else {
        println!("# Recording analysis\n");
        for r in &reports {
            println!("{}", report::take_markdown(r));
        }
        println!("{}", report::summary_markdown(&reports));
    }
    if failed && reports.is_empty() {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn collect(inputs: &[PathBuf]) -> Vec<PathBuf> {
    let is_audio = |p: &Path| {
        p.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| matches!(e.to_ascii_lowercase().as_str(), "wav" | "flac"))
    };
    let mut files: Vec<PathBuf> = inputs
        .iter()
        .flat_map(|p| {
            if p.is_dir() {
                std::fs::read_dir(p)
                    .map(|rd| {
                        rd.filter_map(|e| e.ok().map(|e| e.path()))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
            } else {
                vec![p.clone()]
            }
        })
        .filter(|p| p.is_file() && is_audio(p))
        .collect();
    files.sort();
    files
}

fn show(inputs: &[PathBuf]) -> String {
    inputs
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn write_demo(dir: &Path) -> ExitCode {
    if let Err(e) = std::fs::create_dir_all(dir) {
        eprintln!("{}: {e}", dir.display());
        return ExitCode::FAILURE;
    }
    let takes: [(Instrument, &str, &[f32]); 3] = [
        (
            Instrument::Guitar,
            "01_in-tune",
            &[1.0, -2.0, 0.5, 2.0, -1.0, 1.5],
        ),
        (
            Instrument::Guitar,
            "02_e4-flat",
            &[0.5, 1.0, -1.0, 0.0, 2.0, -12.0],
        ),
        (Instrument::Ukulele, "01_mixed", &[-15.0, 3.0, 9.0, -1.0]),
    ];
    for (k, (instrument, name, detunes)) in takes.iter().enumerate() {
        let take = demo::synthetic_take(*instrument, detunes, 3, k as u64 + 1);
        let path = dir.join(format!("{}_{name}.wav", instrument_name(*instrument)));
        if let Err(e) = audio::write_wav(&path, &take.audio) {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
        println!("wrote {}", path.display());
    }
    ExitCode::SUCCESS
}
