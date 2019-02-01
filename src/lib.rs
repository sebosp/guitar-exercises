
extern crate pitch_calc;
extern crate chords;

use pitch_calc::{
    Hz,
    Letter,
    LetterOctave,
    ScaledPerc,
    Step,
    Octave,
};

/// `StringMaterials` maintains types that make up a string.
/// https://en.wikipedia.org/wiki/String_(music)
enum FlexibleMaterial {
    Steel,
    Nylon,
    Gut,
    Wound,
    // XXX: Overwound strings not covered yet 
}

/// `StringedElement` contains the gauge and frequency of the open string.
/// Used to calculate when the next "note" in a scale should be length-wise 
struct StringedElement {
    material: FlexibleMaterial,
    /// The gauge of the string, in Millimeters
    gauge: f64,
    /// The note of the open string
    note: pitch_calc::Letter,
    /// The octave of the open string
    octave: pitch_calc::Octave,
    /// The string can be disabled for jumping exercises, broken strings, etc
    enabled: bool,
}

impl StringedElement {
    /// `new_steel` creates a new steel string
    fn new_steel(gauge: f64, letter_octave: pitch_calc::LetterOctave) -> StringedElement {
        StringedElement {
            material: FlexibleMaterial::Steel,
            enabled: true,
            gauge: gauge,
            note: letter_octave.0,
            octave: letter_octave.1,
        }
    }
    /// `new_wound` creates a new wounded string, some exercises may need
    /// special speed adjustment depending on the gauge
    fn new_wound(gauge: f64, letter_octave: pitch_calc::LetterOctave) -> StringedElement {
        StringedElement {
            material: FlexibleMaterial::Wound,
            enabled: true,
            gauge: gauge,
            note: letter_octave.0,
            octave: letter_octave.1,
        }
    }
    /// `to_hz` returns the open string Frequency in Hertz
    /// A vibrating string vibrates with a set of frequencies that depend on
    /// the string's tension. pitch
    fn to_hz(self) -> pitch_calc::Hz {
        LetterOctave(self.note, self.octave).to_hz()
    }
}

/// `Fretboard` a vector of strings to practice on
struct Fretboard {
    strings: Vec<StringedElement>,
    frets: u8,
}

/// `Default` provides a Guitar
impl Default for Fretboard {
    /// Gauge used is Regular Light
    /// from https://en.wikipedia.org/wiki/String_(music)#Electric_guitar
    /// Notes and Frequencies are taken from:
    /// https://en.wikipedia.org/wiki/Guitar_tunings
    fn default() -> Fretboard {
        Fretboard {
            strings: vec![
                StringedElement::new_steel(0.2540, LetterOctave(Letter::E, 4)), // E4
                StringedElement::new_steel(0.3302, LetterOctave(Letter::B, 3)), // B3
                StringedElement::new_steel(0.4318, LetterOctave(Letter::G, 3)), // G3
                StringedElement::new_wound(0.6604, LetterOctave(Letter::D, 3)), // D3
                StringedElement::new_wound(0.9144, LetterOctave(Letter::A, 2)), // A2
                StringedElement::new_wound(1.1684, LetterOctave(Letter::E, 2)), // E2
            ],
            frets: 24, 
        }
    }
}
/// `FingerDigit` maintains the finger types
enum FingerDigit {
    Thumb,
    Index,
    Middle,
    Ring,
    Little,
}

/// `Finger` contains information about each finger
struct Finger {
    // The type of finger
    digit: FingerDigit,
    fret: u8,
    /// 0 being weak, 8 being strong
    strength: u8,
    /// 0 being slow, 8 being fast
    agility: u8,
    /// A finger may be free or already busy
    used: bool,
    /// The finger can be disabled for skipping exercises, broken, etc
    enabled: bool,
}

impl Finger {
    fn new(name: String, strength: u8, agility: u8) -> Finger {
        let finger_digit = match name.as_ref() {
            "Thumb" => FingerDigit::Thumb,
            "Index" => FingerDigit::Index,
            "Middle" => FingerDigit::Middle,
            "Ring" => FingerDigit::Ring,
            "Little" => FingerDigit::Little,
            &_ => FingerDigit::Index, // By default use Index.
        };
        Finger{
                digit: finger_digit,
                fret: 0,
                strength: strength,
                agility: agility,
                used: false,
                enabled: true,
        }
    }
}

/// `HandSide` allows use of different hands for tapping/harmonics
/// Technically other hands could be added by more players joining
/// the same fretboard
enum HandSide {
    Right,
    Left,
}

/// `Hand` keeps track of the position of fingers
struct Hand {
    fingers: Vec<Finger>,
    side: HandSide,
    dexterity: u8,
}

/// `Default` is the left hand setup
impl Default for Hand{
    fn default() -> Hand {
        // These are made-up numbers, maybe check some studies
        Hand{
            fingers: vec![

                Finger::new(
                    "Index".to_owned(),
                    3u8, // Medium Strength
                    4u8, // High Agility
                    ),
                Finger::new(
                    "Middle".to_owned(),
                    4u8, // High Strength
                    5u8, // High Agility
                    ),
                Finger::new(
                    "Ring".to_owned(),
                    3u8, // Medium Strength
                    3u8, // Medium Agility
                    ),
                Finger::new(
                    "Little".to_owned(),
                    2u8, // Low Strength
                    2u8, // Low Agility
                    ),
            ],
            dexterity: 0,
            side: HandSide::Left,
        }
    }
}

impl Hand{
    fn with_thumb(mut self) -> Self {
        self.fingers.insert(
            0usize,
            Finger::new(
                "Thumb".to_owned(),
                5u8, // Maximum Strength
                0u8, // Minimum Agility
            ));
        self
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_gets_strings() {
    }
}
