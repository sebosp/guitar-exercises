/// `fretboard` contains functionality that maps a scale to a fretboard
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
#[derive(Debug)]
enum FlexibleMaterial {
    Steel,
    Nylon,
    Gut,
    Wound,
    // XXX: Overwound strings not covered yet 
}

/// `StringedElement` contains the gauge and frequency of the open string.
/// Used to calculate when the next "note" in a scale should be length-wise 
#[derive(Debug)]
pub struct StringedElement {
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
    pub fn new_steel(gauge: f64, letter_octave: pitch_calc::LetterOctave) -> StringedElement {
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
    pub fn new_wound(gauge: f64, letter_octave: pitch_calc::LetterOctave) -> StringedElement {
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
    pub fn to_hz(self) -> pitch_calc::Hz {
        LetterOctave(self.note, self.octave).to_hz()
    }
}

/// `Fretboard` a vector of strings to practice on
#[derive(Debug)]
struct Fretboard {
    strings: Vec<StringedElement>,
    frets: u8,
}

/// `Default` provides a 6 string standard tuning Guitar
/// with 24 frets
impl Default for Fretboard {
    /// Gauge used is Regular Light
    /// from https://en.wikipedia.org/wiki/String_(music)#Electric_guitar
    /// Notes are taken from:
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
impl Fretboard {
    pub fn default_7() -> Fretboard {
        let mut guitar = Fretboard::default();
        guitar.strings.push(
          StringedElement::new_wound(1.4224, LetterOctave(Letter::B, 1)), // B1
        );
        guitar
    }
}
