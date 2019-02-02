
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