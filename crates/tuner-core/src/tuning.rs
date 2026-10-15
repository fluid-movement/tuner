//! Notes, cents and instrument tunings in 12-tone equal temperament with
//! A4 = 440 Hz.

pub const A4_FREQUENCY: f32 = 440.0;
const A4_MIDI: i32 = 69;
const NOTE_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

/// Frequency of a MIDI note number.
pub fn midi_to_frequency(midi: i32) -> f32 {
    A4_FREQUENCY * 2f32.powf((midi - A4_MIDI) as f32 / 12.0)
}

/// Distance from `reference` to `frequency` in cents (positive is sharp).
pub fn cents_between(frequency: f32, reference: f32) -> f32 {
    1200.0 * (frequency / reference).log2()
}

/// The equal-tempered note nearest to a frequency.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Note {
    pub midi: i32,
    /// Offset of the frequency from the note, from -50 to +50 cents.
    pub cents: f32,
}

impl Note {
    pub fn from_frequency(frequency: f32) -> Self {
        let semitones = 12.0 * (frequency / A4_FREQUENCY).log2();
        let midi = A4_MIDI + semitones.round() as i32;
        Self {
            midi,
            cents: cents_between(frequency, midi_to_frequency(midi)),
        }
    }

    /// Pitch class name using sharps, such as `"C#"`.
    pub fn name(&self) -> &'static str {
        NOTE_NAMES[self.midi.rem_euclid(12) as usize]
    }

    /// Scientific pitch notation octave (middle C is C4).
    pub fn octave(&self) -> i32 {
        self.midi.div_euclid(12) - 1
    }
}

/// One string of a tuning.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StringTarget {
    /// Note name with octave, such as `"E2"`.
    pub name: &'static str,
    pub midi: i32,
    pub frequency: f32,
}

const fn target(name: &'static str, midi: i32, frequency: f32) -> StringTarget {
    StringTarget {
        name,
        midi,
        frequency,
    }
}

/// Guitar standard tuning, low to high.
const GUITAR: [StringTarget; 6] = [
    target("E2", 40, 82.406_89),
    target("A2", 45, 110.0),
    target("D3", 50, 146.832_38),
    target("G3", 55, 195.997_72),
    target("B3", 59, 246.941_65),
    target("E4", 64, 329.627_56),
];

/// Ukulele standard re-entrant tuning, in string order (G is the high G4).
const UKULELE: [StringTarget; 4] = [
    target("G4", 67, 391.995_42),
    target("C4", 60, 261.625_57),
    target("E4", 64, 329.627_56),
    target("A4", 69, 440.0),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instrument {
    Guitar,
    Ukulele,
}

impl Instrument {
    /// Standard tuning in string order.
    pub fn strings(self) -> &'static [StringTarget] {
        match self {
            Self::Guitar => &GUITAR,
            Self::Ukulele => &UKULELE,
        }
    }

    /// The string whose target is nearest to `frequency` (by pitch, not Hz),
    /// with its index in [`strings`](Self::strings) and the offset in cents.
    pub fn nearest_string(self, frequency: f32) -> (usize, &'static StringTarget, f32) {
        self.strings()
            .iter()
            .enumerate()
            .map(|(i, s)| (i, s, cents_between(frequency, s.frequency)))
            .min_by(|a, b| a.2.abs().total_cmp(&b.2.abs()))
            .expect("tunings are not empty")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() <= eps
    }

    #[test]
    fn string_frequencies_match_their_midi_notes() {
        for inst in [Instrument::Guitar, Instrument::Ukulele] {
            for s in inst.strings() {
                assert!(
                    close(midi_to_frequency(s.midi), s.frequency, 0.01),
                    "{}",
                    s.name
                );
                let note = Note::from_frequency(s.frequency);
                assert_eq!(format!("{}{}", note.name(), note.octave()), s.name);
                assert!(close(note.cents, 0.0, 0.01));
            }
        }
    }

    #[test]
    fn notes_and_cents() {
        let n = Note::from_frequency(440.0);
        assert_eq!((n.name(), n.octave(), n.midi), ("A", 4, 69));

        let n = Note::from_frequency(midi_to_frequency(61) * 2f32.powf(20.0 / 1200.0));
        assert_eq!((n.name(), n.octave()), ("C#", 4));
        assert!(close(n.cents, 20.0, 0.05));

        let n = Note::from_frequency(midi_to_frequency(40) * 2f32.powf(-30.0 / 1200.0));
        assert_eq!((n.name(), n.octave()), ("E", 2));
        assert!(close(n.cents, -30.0, 0.05));

        assert!(close(cents_between(880.0, 440.0), 1200.0, 0.01));
        assert_eq!(Note::from_frequency(16.35).octave(), 0);
        assert_eq!(Note::from_frequency(15.0).name(), "B");
        assert_eq!(Note::from_frequency(15.0).octave(), -1);
    }

    #[test]
    fn nearest_guitar_string() {
        let (i, s, c) = Instrument::Guitar.nearest_string(84.0);
        assert_eq!((i, s.name), (0, "E2"));
        assert!(close(c, cents_between(84.0, 82.406_89), 0.001) && c > 0.0);

        let (i, s, c) = Instrument::Guitar.nearest_string(240.0);
        assert_eq!((i, s.name), (4, "B3"));
        assert!(c < 0.0);

        assert_eq!(Instrument::Guitar.nearest_string(330.5).1.name, "E4");
    }

    #[test]
    fn nearest_ukulele_string_respects_reentrant_order() {
        assert_eq!(Instrument::Ukulele.nearest_string(392.0).0, 0); // G4 is string 0
        assert_eq!(Instrument::Ukulele.nearest_string(262.0).0, 1);
        assert_eq!(Instrument::Ukulele.nearest_string(331.0).0, 2);
        let (i, s, c) = Instrument::Ukulele.nearest_string(436.0);
        assert_eq!((i, s.name), (3, "A4"));
        assert!(close(c, cents_between(436.0, 440.0), 0.001));
    }
}
