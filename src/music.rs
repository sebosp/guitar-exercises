//! Notes, instruments, tunings and the staff: the music theory of the game, without Bevy.
//!
//! Pitches are MIDI numbers (E2, the open low string of a guitar, is 40). Guitar music is written
//! one octave above its sound in the treble clef, bass music one octave above its sound in the
//! bass clef, so the staff shows the written pitch: `sounding + 12`.
use serde::{Deserialize, Serialize};

/// The letters of the notes, C is 0, B is 6.
pub const LETTERS: [char; 7] = ['C', 'D', 'E', 'F', 'G', 'A', 'B'];
/// The semitones of the letters from C.
const LETTER_SEMITONES: [i32; 7] = [0, 2, 4, 5, 7, 9, 11];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Accidental {
    Natural,
    Sharp,
    Flat,
}

/// A note with its spelling: the letter, the accidental and the octave of the scientific pitch
/// notation (middle C is C4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Spelled {
    /// 0 is C, 6 is B.
    pub letter: u8,
    pub accidental: Accidental,
    pub octave: i32,
}

impl Spelled {
    pub fn natural(letter: u8, octave: i32) -> Self {
        Spelled { letter, accidental: Accidental::Natural, octave }
    }

    pub fn midi(self) -> i32 {
        let alter = match self.accidental {
            Accidental::Natural => 0,
            Accidental::Sharp => 1,
            Accidental::Flat => -1,
        };
        (self.octave + 1) * 12 + LETTER_SEMITONES[self.letter as usize] + alter
    }

    /// The diatonic step from C0: the vertical position on a staff.
    pub fn step(self) -> i32 {
        self.octave * 7 + self.letter as i32
    }

    /// The spelling of a MIDI note, with sharps or with flats for the black keys.
    pub fn from_midi(midi: i32, flats: bool) -> Self {
        let octave = midi.div_euclid(12) - 1;
        let pc = midi.rem_euclid(12);
        if let Some(letter) = LETTER_SEMITONES.iter().position(|&s| s == pc) {
            return Spelled::natural(letter as u8, octave);
        }
        if flats {
            let letter = LETTER_SEMITONES.iter().position(|&s| s == pc + 1).unwrap_or(0);
            Spelled { letter: letter as u8, accidental: Accidental::Flat, octave }
        } else {
            let letter = LETTER_SEMITONES.iter().position(|&s| s == pc - 1).unwrap_or(0);
            Spelled { letter: letter as u8, accidental: Accidental::Sharp, octave }
        }
    }

    /// The written note of a sounding one: one octave higher for the guitar and the bass.
    pub fn written(self) -> Self {
        Spelled { octave: self.octave + 1, ..self }
    }

    /// The name without the octave, `F#`: plain ASCII, the UI font has no ♯ nor ♭.
    pub fn name(self) -> String {
        let mut name = LETTERS[self.letter as usize].to_string();
        match self.accidental {
            Accidental::Natural => {}
            Accidental::Sharp => name.push('#'),
            Accidental::Flat => name.push('b'),
        }
        name
    }

    /// The name with the octave, `F#3`.
    pub fn full_name(self) -> String {
        format!("{}{}", self.name(), self.octave)
    }

    /// The accidental on the staff, in the music font.
    pub fn accidental_glyph(self) -> Option<&'static str> {
        match self.accidental {
            Accidental::Natural => None,
            Accidental::Sharp => Some("\u{266F}"),
            Accidental::Flat => Some("\u{266D}"),
        }
    }

    pub fn is_natural(self) -> bool {
        self.accidental == Accidental::Natural
    }
}

/// The name of a MIDI note with sharps, `C#4`.
pub fn midi_name(midi: i32) -> String {
    Spelled::from_midi(midi, false).full_name()
}

pub fn midi_to_hz(midi: f32) -> f32 {
    440.0 * 2f32.powf((midi - 69.0) / 12.0)
}

pub fn hz_to_midi(hz: f32) -> f32 {
    69.0 + 12.0 * (hz / 440.0).log2()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum InstrumentKind {
    #[default]
    Guitar,
    Bass,
}

impl InstrumentKind {
    pub const ALL: [InstrumentKind; 2] = [InstrumentKind::Guitar, InstrumentKind::Bass];

    pub fn name(self) -> &'static str {
        match self {
            InstrumentKind::Guitar => "Guitar",
            InstrumentKind::Bass => "Bass",
        }
    }

    /// The numbers of strings offered in the settings.
    pub fn string_counts(self) -> std::ops::RangeInclusive<usize> {
        match self {
            InstrumentKind::Guitar => 6..=8,
            InstrumentKind::Bass => 4..=6,
        }
    }

    pub fn default_strings(self) -> usize {
        match self {
            InstrumentKind::Guitar => 6,
            InstrumentKind::Bass => 4,
        }
    }

    /// The open strings in standard tuning, from the lowest to the highest.
    pub fn standard_tuning(self, strings: usize) -> Vec<i32> {
        match (self, strings) {
            // F♯1 B1 E2 A2 D3 G3 B3 E4
            (InstrumentKind::Guitar, 8) => vec![30, 35, 40, 45, 50, 55, 59, 64],
            (InstrumentKind::Guitar, 7) => vec![35, 40, 45, 50, 55, 59, 64],
            (InstrumentKind::Guitar, _) => vec![40, 45, 50, 55, 59, 64],
            // B0 E1 A1 D2 G2 C3
            (InstrumentKind::Bass, 6) => vec![23, 28, 33, 38, 43, 48],
            (InstrumentKind::Bass, 5) => vec![23, 28, 33, 38, 43],
            (InstrumentKind::Bass, _) => vec![28, 33, 38, 43],
        }
    }

    pub fn clef(self) -> Clef {
        match self {
            InstrumentKind::Guitar => Clef::Treble,
            InstrumentKind::Bass => Clef::Bass,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum TuningPreset {
    #[default]
    Standard,
    /// The lowest string a whole tone down.
    Drop,
    /// Every string a semitone down.
    HalfStepDown,
}

impl TuningPreset {
    pub const ALL: [TuningPreset; 3] = [TuningPreset::Standard, TuningPreset::Drop, TuningPreset::HalfStepDown];

    pub fn name(self) -> &'static str {
        match self {
            TuningPreset::Standard => "Standard",
            TuningPreset::Drop => "Drop",
            TuningPreset::HalfStepDown => "Half step down",
        }
    }

    pub fn apply(self, mut open: Vec<i32>) -> Vec<i32> {
        match self {
            TuningPreset::Standard => {}
            TuningPreset::Drop => {
                if let Some(low) = open.first_mut() {
                    *low -= 2;
                }
            }
            TuningPreset::HalfStepDown => open.iter_mut().for_each(|s| *s -= 1),
        }
        open
    }
}

/// A place on the fretboard. String 0 is the lowest one; on the screen and in the tablature the
/// highest string is on top.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub string: usize,
    pub fret: u8,
}

/// An instrument as the player set it up: the open strings and the frets shown.
#[derive(Debug, Clone, PartialEq)]
pub struct Instrument {
    pub kind: InstrumentKind,
    /// The open strings, from the lowest to the highest.
    pub open: Vec<i32>,
    pub frets: u8,
}

impl Instrument {
    pub fn new(kind: InstrumentKind, strings: usize, tuning: TuningPreset, frets: u8) -> Self {
        Instrument { kind, open: tuning.apply(kind.standard_tuning(strings)), frets }
    }

    pub fn strings(&self) -> usize {
        self.open.len()
    }

    pub fn midi_at(&self, pos: Position) -> i32 {
        self.open[pos.string] + pos.fret as i32
    }

    /// Every place where a note can be played, the lowest string first.
    pub fn positions_of(&self, midi: i32) -> Vec<Position> {
        self.open
            .iter()
            .enumerate()
            .filter_map(|(string, &open)| {
                let fret = midi - open;
                (0..=self.frets as i32).contains(&fret).then_some(Position { string, fret: fret as u8 })
            })
            .collect()
    }

    pub fn lowest(&self) -> i32 {
        self.open.iter().copied().min().unwrap_or(40)
    }

    pub fn highest(&self) -> i32 {
        self.open.iter().copied().max().unwrap_or(64) + self.frets as i32
    }

    /// The string names for the tablature, `e B G D A E` from the top: the highest string in
    /// lowercase when it has the letter of the lowest one, as usual for guitar.
    pub fn string_label(&self, string: usize) -> String {
        let name = Spelled::from_midi(self.open[string], false).name();
        let lowest = Spelled::from_midi(self.open[0], false).name();
        if string == self.strings() - 1 && string > 0 && name == lowest { name.to_lowercase() } else { name }
    }

    /// The key of the per instrument progress, `guitar-6`.
    pub fn key(&self) -> String {
        format!("{}-{}", self.kind.name().to_lowercase(), self.strings())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clef {
    Treble,
    Bass,
}

impl Clef {
    /// The diatonic step of the bottom line: E4 for the treble clef, G2 for the bass clef.
    pub fn bottom_line(self) -> i32 {
        match self {
            Clef::Treble => Spelled::natural(2, 4).step(),
            Clef::Bass => Spelled::natural(4, 2).step(),
        }
    }

    /// The glyph of the clef in the Musical Symbols block.
    pub fn glyph(self) -> &'static str {
        match self {
            Clef::Treble => "\u{1D11E}",
            Clef::Bass => "\u{1D122}",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Clef::Treble => "treble clef",
            Clef::Bass => "bass clef",
        }
    }

    /// The mnemonics of the lines and the spaces, from the bottom to the top.
    pub fn mnemonics(self) -> (Mnemonic, Mnemonic) {
        match self {
            Clef::Treble => {
                (Mnemonic { letters: "EGBDF", phrase: "Every Good Boy Does Fine" }, Mnemonic { letters: "FACE", phrase: "FACE" })
            }
            Clef::Bass => (
                Mnemonic { letters: "GBDFA", phrase: "Good Boys Do Fine Always" },
                Mnemonic { letters: "ACEG", phrase: "All Cows Eat Grass" },
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mnemonic {
    pub letters: &'static str,
    pub phrase: &'static str,
}

/// Where a written note sits on a staff: `position` 0 is the bottom line, 1 the first space, 8
/// the top line; negative below the staff, above 8 over it.
pub fn staff_position(clef: Clef, written: Spelled) -> i32 {
    written.step() - clef.bottom_line()
}

/// The number of ledger lines a note needs, below (negative) or above (positive) the staff.
pub fn ledger_lines(position: i32) -> i32 {
    if position < -1 {
        position / 2
    } else if position > 9 {
        (position - 8) / 2
    } else {
        0
    }
}

/// How to find a written note on the staff: the mnemonic of the lines or the spaces, or the way
/// from the nearest note of the staff with ledger lines.
pub fn staff_hint(clef: Clef, written: Spelled) -> String {
    let position = staff_position(clef, written);
    let (lines, spaces) = clef.mnemonics();
    let letter = LETTERS[written.letter as usize];
    match position {
        0..=8 if position % 2 == 0 => {
            let n = position / 2 + 1;
            format!("{letter} is line {n} from the bottom: {} ({})", lines.phrase, lines.letters)
        }
        1..=7 => {
            let n = (position + 1) / 2;
            format!("{letter} is space {n} from the bottom: {} ({})", spaces.phrase, spaces.letters)
        }
        -1 => format!("{letter} hangs just below the bottom line, {}", lines.letters.chars().next().unwrap_or(' ')),
        9 => format!("{letter} sits just above the top line, {}", lines.letters.chars().last().unwrap_or(' ')),
        p if p < 0 => {
            let ledgers = -ledger_lines(p);
            let on = if p % 2 == 0 { "on" } else { "under" };
            format!("{letter} is {on} ledger line {ledgers} below the staff, {} steps under the bottom line", -p)
        }
        p => {
            let ledgers = ledger_lines(p);
            let on = if p % 2 == 0 { "on" } else { "over" };
            format!("{letter} is {on} ledger line {ledgers} above the staff, {} steps over the top line", p - 8)
        }
    }
}

/// The natural notes of the instrument in the order they are learned: from the middle of the
/// staff outwards (the lines and the spaces first, the ledger lines later), the sharps and the
/// flats last when `accidentals` is on.
pub fn learning_order(instrument: &Instrument, accidentals: bool) -> Vec<Spelled> {
    let clef = instrument.kind.clef();
    let middle = 4;
    let mut naturals: Vec<Spelled> =
        (instrument.lowest()..=instrument.highest()).map(|m| Spelled::from_midi(m, false)).filter(|s| s.is_natural()).collect();
    naturals.sort_by_key(|s| {
        let p = staff_position(clef, s.written());
        ((p - middle).abs(), p < middle)
    });
    if accidentals {
        let mut sharps: Vec<Spelled> =
            (instrument.lowest()..=instrument.highest()).map(|m| Spelled::from_midi(m, false)).filter(|s| !s.is_natural()).collect();
        sharps.sort_by_key(|s| {
            let p = staff_position(clef, s.written());
            ((p - middle).abs(), p < middle)
        });
        let flats: Vec<Spelled> = sharps.iter().map(|s| Spelled::from_midi(s.midi(), true)).collect();
        // A sharp, then the same key as a flat.
        for (sharp, flat) in sharps.into_iter().zip(flats) {
            naturals.push(sharp);
            naturals.push(flat);
        }
    }
    naturals
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spelling() {
        assert_eq!(Spelled::from_midi(60, false).full_name(), "C4");
        assert_eq!(Spelled::from_midi(61, false).full_name(), "C#4");
        assert_eq!(Spelled::from_midi(61, true).full_name(), "Db4");
        assert_eq!(Spelled::from_midi(40, false).full_name(), "E2");
        for m in 20..100 {
            assert_eq!(Spelled::from_midi(m, false).midi(), m);
            assert_eq!(Spelled::from_midi(m, true).midi(), m);
        }
        assert!((midi_to_hz(69.0) - 440.0).abs() < 1e-3);
        assert!((hz_to_midi(82.41) - 40.0).abs() < 0.01);
    }

    #[test]
    fn tunings() {
        let guitar = Instrument::new(InstrumentKind::Guitar, 6, TuningPreset::Standard, 12);
        let labels: Vec<String> = (0..6).rev().map(|s| guitar.string_label(s)).collect();
        assert_eq!(labels, ["e", "B", "G", "D", "A", "E"]);
        let drop = Instrument::new(InstrumentKind::Guitar, 6, TuningPreset::Drop, 12);
        assert_eq!(drop.open[0], 38);
        let bass = Instrument::new(InstrumentKind::Bass, 5, TuningPreset::Standard, 12);
        assert_eq!(bass.open, [23, 28, 33, 38, 43]);
    }

    #[test]
    fn positions() {
        let guitar = Instrument::new(InstrumentKind::Guitar, 6, TuningPreset::Standard, 12);
        // E4: open high e, B string fret 5, G fret 9, D fret 14 is off the 12 frets.
        let e4 = guitar.positions_of(64);
        assert_eq!(e4, [Position { string: 3, fret: 9 }, Position { string: 4, fret: 5 }, Position { string: 5, fret: 0 }]);
        assert_eq!(guitar.positions_of(40), [Position { string: 0, fret: 0 }]);
    }

    #[test]
    fn staff() {
        // The open low E of a guitar is written E3, three ledger lines below the treble staff.
        let low_e = Spelled::from_midi(40, false).written();
        assert_eq!(staff_position(Clef::Treble, low_e), -7);
        assert_eq!(ledger_lines(-7), -3);
        // The open high e is written E5, the fourth space.
        let high_e = Spelled::from_midi(64, false).written();
        assert_eq!(staff_position(Clef::Treble, high_e), 7);
        assert!(staff_hint(Clef::Treble, high_e).contains("FACE"));
        // The open G string is written G4, the second line: Every Good.
        let g = Spelled::from_midi(55, false).written();
        assert_eq!(staff_position(Clef::Treble, g), 2);
        assert!(staff_hint(Clef::Treble, g).contains("line 2"));
        // The open E of a bass sounds E1, is written E2: under the bass staff.
        let bass_e = Spelled::from_midi(28, false).written();
        assert_eq!(staff_position(Clef::Bass, bass_e), -2);
        assert_eq!(ledger_lines(-2), -1);
        assert_eq!(ledger_lines(-1), 0);
        assert_eq!(ledger_lines(10), 1);
    }

    #[test]
    fn order() {
        let guitar = Instrument::new(InstrumentKind::Guitar, 6, TuningPreset::Standard, 12);
        let order = learning_order(&guitar, false);
        // B4 written, the middle line: the open B string.
        assert_eq!(order[0].written().full_name(), "B4");
        assert!(order.iter().all(|s| s.is_natural()));
        // E2 to E5 (high e fret 12): 3 octaves of naturals plus one.
        assert_eq!(order.len(), 22);
        let with = learning_order(&guitar, true);
        assert!(with.len() > order.len());
        assert!(with[..order.len()].iter().all(|s| s.is_natural()));
    }
}
