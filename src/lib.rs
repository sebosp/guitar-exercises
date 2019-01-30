
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
    /// A vibrating string vibrates with a set of frequencies that depend on
    /// the string's tension. These frequencies can be derived from Newton's
    /// laws of motion. https://en.wikipedia.org/wiki/Tension_(physics)
    /// The frequency is stored in Hertz
    frequency: f64,
    enabled: bool,
}

impl StringedElement {
    fn new_steel(gauge: f64, frequency: f64) -> StringedElement {
        StringedElement {
            material: FlexibleMaterial::Steel,
            enabled: true,
            gauge: gauge,
            frequency: frequency,
        }
    }
    fn new_wound(gauge: f64, frequency: f64) -> StringedElement {
        StringedElement {
            material: FlexibleMaterial::Wound,
            enabled: true,
            gauge: gauge,
            frequency: frequency,
        }
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
                StringedElement::new_steel(0.2540, 329.63), // E4
                StringedElement::new_steel(0.3302, 246.94), // B3
                StringedElement::new_steel(0.4318, 196.00), // G3
                StringedElement::new_wound(0.6604, 146.83), // D3
                StringedElement::new_wound(0.9144, 110.00), // A2
                StringedElement::new_wound(1.1684, 82.41),  // E2
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
}

impl Finger {
    fn new(name: String) -> Finger {
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
                strength: 0,
                agility: 0,
                used: false,
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
        Hand{
            fingers: vec![
                Finger::new("Thumb".to_owned()),
                Finger::new("Index".to_owned()),
                Finger::new("Middle".to_owned()),
                Finger::new("Ring".to_owned()),
                Finger::new("Little".to_owned()),
            ],
            dexterity: 0,
            side: HandSide::Left,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_gets_strings() {
    }
}
