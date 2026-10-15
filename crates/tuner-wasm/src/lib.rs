//! Raw `extern "C"` exports of `tuner-core`. No wasm-bindgen: the module runs
//! inside an AudioWorkletGlobalScope, which lacks the APIs its JS glue needs,
//! so the worklet instantiates the bytes directly and calls these functions.
//!
//! Usage from JS: `tuner_new(sampleRate)` returns a handle; write
//! `tuner_frame_size(handle)` samples as `f32` at `tuner_input(handle)` in
//! linear memory, call `tuner_detect(handle)`, and if it returns 1 read
//! `tuner_frequency(handle)` and `tuner_clarity(handle)`.

use tuner_core::tuning::cents_between;
use tuner_core::{Instrument, Note, PitchConfig, PitchDetector, PitchEstimate};

/// Detector state behind a handle owned by JS.
pub struct Tuner {
    detector: PitchDetector,
    input: Vec<f32>,
    last: Option<PitchEstimate>,
}

/// Returns [`tuner_core::VERSION`].
#[unsafe(no_mangle)]
pub extern "C" fn tuner_version() -> u32 {
    tuner_core::VERSION
}

/// MIDI number of the equal-tempered note nearest to `frequency`.
#[unsafe(no_mangle)]
pub extern "C" fn tuner_note_midi(frequency: f32) -> i32 {
    Note::from_frequency(frequency).midi
}

/// Offset in cents (-50..50) of `frequency` from its nearest note.
#[unsafe(no_mangle)]
pub extern "C" fn tuner_note_cents(frequency: f32) -> f32 {
    Note::from_frequency(frequency).cents
}

/// Instrument ids used by the tuning exports: 0 is guitar, 1 is ukulele.
fn instrument(id: u32) -> Option<Instrument> {
    match id {
        0 => Some(Instrument::Guitar),
        1 => Some(Instrument::Ukulele),
        _ => None,
    }
}

/// Number of strings of instrument `id` in standard tuning (0 if unknown).
#[unsafe(no_mangle)]
pub extern "C" fn tuner_string_count(id: u32) -> u32 {
    instrument(id).map_or(0, |i| i.strings().len() as u32)
}

/// MIDI number of string `index` of instrument `id`, or -1 if out of range.
#[unsafe(no_mangle)]
pub extern "C" fn tuner_string_midi(id: u32, index: u32) -> i32 {
    string(id, index).map_or(-1, |s| s.midi)
}

/// Target frequency in Hz of string `index` of instrument `id`, or 0.
#[unsafe(no_mangle)]
pub extern "C" fn tuner_string_frequency(id: u32, index: u32) -> f32 {
    string(id, index).map_or(0.0, |s| s.frequency)
}

fn string(id: u32, index: u32) -> Option<&'static tuner_core::StringTarget> {
    instrument(id)?.strings().get(index as usize)
}

/// Index of the string of instrument `id` nearest to `frequency` (by pitch),
/// or -1 if the instrument is unknown.
#[unsafe(no_mangle)]
pub extern "C" fn tuner_nearest_string(id: u32, frequency: f32) -> i32 {
    instrument(id).map_or(-1, |i| i.nearest_string(frequency).0 as i32)
}

/// Distance from `reference` to `frequency` in cents (positive is sharp).
#[unsafe(no_mangle)]
pub extern "C" fn tuner_cents_between(frequency: f32, reference: f32) -> f32 {
    cents_between(frequency, reference)
}

/// Creates a detector with the default configuration for `sample_rate`.
/// Release it with [`tuner_free`].
#[unsafe(no_mangle)]
pub extern "C" fn tuner_new(sample_rate: f32) -> *mut Tuner {
    let detector = PitchDetector::new(PitchConfig::new(sample_rate));
    let input = vec![0.0; detector.config().frame_size];
    Box::into_raw(Box::new(Tuner {
        detector,
        input,
        last: None,
    }))
}

/// # Safety
///
/// `tuner` must come from [`tuner_new`] and not be used afterwards.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tuner_free(tuner: *mut Tuner) {
    if !tuner.is_null() {
        drop(unsafe { Box::from_raw(tuner) });
    }
}

/// Number of samples [`tuner_detect`] analyses.
///
/// # Safety
///
/// `tuner` must be a live handle from [`tuner_new`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tuner_frame_size(tuner: *const Tuner) -> u32 {
    unsafe { &*tuner }.input.len() as u32
}

/// Address of the input frame in linear memory.
///
/// # Safety
///
/// `tuner` must be a live handle from [`tuner_new`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tuner_input(tuner: *mut Tuner) -> *mut f32 {
    unsafe { &mut *tuner }.input.as_mut_ptr()
}

/// Analyses the input frame. Returns 1 if a pitch was found, else 0.
///
/// # Safety
///
/// `tuner` must be a live handle from [`tuner_new`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tuner_detect(tuner: *mut Tuner) -> u32 {
    let tuner = unsafe { &mut *tuner };
    tuner.last = tuner.detector.detect(&tuner.input);
    tuner.last.is_some() as u32
}

/// Frequency in Hz of the last detection, or 0 if none.
///
/// # Safety
///
/// `tuner` must be a live handle from [`tuner_new`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tuner_frequency(tuner: *const Tuner) -> f32 {
    unsafe { &*tuner }.last.map_or(0.0, |e| e.frequency)
}

/// Clarity (0..1) of the last detection, or 0 if none.
///
/// # Safety
///
/// `tuner` must be a live handle from [`tuner_new`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tuner_clarity(tuner: *const Tuner) -> f32 {
    unsafe { &*tuner }.last.map_or(0.0, |e| e.clarity)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::TAU;

    #[test]
    fn exports_core_version() {
        assert_eq!(tuner_version(), tuner_core::VERSION);
    }

    #[test]
    fn maps_frequencies_to_notes() {
        assert_eq!(tuner_note_midi(440.0), 69);
        assert_eq!(tuner_note_midi(82.41), 40);
        assert!((tuner_note_cents(446.0) - 23.45).abs() < 0.05);
    }

    #[test]
    fn exposes_tunings() {
        assert_eq!(tuner_string_count(0), 6);
        assert_eq!(tuner_string_count(1), 4);
        assert_eq!(tuner_string_count(2), 0);
        assert_eq!(tuner_string_midi(0, 0), 40);
        assert_eq!(tuner_string_midi(1, 0), 67);
        assert_eq!(tuner_string_midi(1, 4), -1);
        assert!((tuner_string_frequency(0, 1) - 110.0).abs() < 1e-4);
        assert_eq!(tuner_nearest_string(0, 84.0), 0);
        assert_eq!(tuner_nearest_string(1, 262.0), 1);
        assert_eq!(tuner_nearest_string(9, 262.0), -1);
        assert!((tuner_cents_between(880.0, 440.0) - 1200.0).abs() < 0.01);
    }

    #[test]
    fn detects_through_the_c_abi() {
        let rate = 48_000.0;
        unsafe {
            let t = tuner_new(rate);
            let n = tuner_frame_size(t) as usize;
            assert_eq!(n, 2048);
            let input = std::slice::from_raw_parts_mut(tuner_input(t), n);
            for (i, x) in input.iter_mut().enumerate() {
                *x = 0.4 * (TAU * 110.0 * i as f32 / rate).sin();
            }
            assert_eq!(tuner_detect(t), 1);
            assert!((tuner_frequency(t) - 110.0).abs() < 0.1);
            assert!(tuner_clarity(t) > 0.9);

            input.fill(0.0);
            assert_eq!(tuner_detect(t), 0);
            assert_eq!(tuner_frequency(t), 0.0);
            assert_eq!(tuner_clarity(t), 0.0);
            tuner_free(t);
        }
    }
}
