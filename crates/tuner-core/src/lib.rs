//! Pitch analysis for the tuner. Pure Rust with no web dependencies, so it
//! can be tested natively and wrapped for WASM by `tuner-wasm`.

pub mod pitch;
pub mod poly;
pub mod synth;
pub mod tuning;

pub use pitch::{PitchConfig, PitchDetector, PitchEstimate};
pub use tuning::{Instrument, Note, StringTarget};

/// Version of the analysis core, exposed so the web app can confirm the
/// module it loaded.
pub const VERSION: u32 = 1;
