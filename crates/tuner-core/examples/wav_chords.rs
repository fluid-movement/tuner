//! Prints the chord reports for a 16-bit PCM mono WAV (looped `loops`
//! times), e.g. the browser tests' fake-microphone files.
//!
//! cargo run --release -p tuner-core --example wav_chords <file.wav> [loops]

use tuner_core::strum::{Analysis, StrumListener};
use tuner_core::tuning::Instrument;

fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("usage: wav_chords <file.wav> [loops]");
    let loops: usize = args.next().and_then(|s| s.parse().ok()).unwrap_or(2);
    let bytes = std::fs::read(&path).expect("read wav");
    let rate = u32::from_le_bytes(bytes[24..28].try_into().unwrap()) as f32;
    let data = bytes
        .windows(4)
        .position(|w| w == b"data")
        .expect("data chunk")
        + 8;
    let samples: Vec<f32> = bytes[data..]
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&b| f32::from(i16::from_le_bytes(b)) / 32768.0)
        .collect();
    let x: Vec<f32> = (0..loops).flat_map(|_| samples.iter().copied()).collect();
    let mut listener = StrumListener::new(rate, Instrument::Ukulele);
    listener.set_analysis(Analysis::Chord);
    for c in x.chunks(1024) {
        if let Some(r) = listener.push(c) {
            println!(
                "{:7.3} s settled {:5} {:?}",
                r.onset_seconds,
                r.settled,
                r.chord.map(|c| c.frets)
            );
        }
    }
}
