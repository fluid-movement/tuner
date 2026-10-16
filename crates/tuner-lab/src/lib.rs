//! Offline analysis of guitar and ukulele recordings with tuner-core.
//!
//! A take follows the recording protocol: each string plucked on its own,
//! then a few strums, without retuning in between. The single plucks show how
//! the single-string tuner behaves on real strings and give the ground truth
//! for checking the strum estimates. See `tuner-lab --help`.

pub mod analyze;
pub mod audio;
pub mod demo;
pub mod report;
pub mod segment;

/// Shown when there is nothing to analyse.
pub const PROTOCOL: &str = "\
Recording protocol (one tuning state per file):
  1. Set a tuning state; don't touch the pegs for the rest of the take.
  2. Pluck each string on its own, low to high, letting each ring ~2 s.
  3. Strum all strings 3-4 times, letting each ring ~3 s.
  4. Save as WAV or FLAC (mono or stereo, 44.1/48 kHz, no noise suppression
     or auto gain) named after the instrument and the take, e.g.
     guitar_04_e4-flat.wav or ukulele_02_c-sharp.flac.
Put the files in recordings/ (gitignored) and run `just analyze`.
`tuner-lab demo <dir>` writes synthetic example takes.";
