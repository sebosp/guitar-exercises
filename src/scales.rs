//! Scales and their modes, and how to play them three notes per string (two for the
//! pentatonics): one position that climbs the neck a little from string to string. Without Bevy.
use serde::{Deserialize, Serialize};

use crate::music::{Accidental, Instrument, LETTERS, Position, Spelled};

/// The roots offered, by pitch class from C: the usual spelling of each key.
pub const ROOT_NAMES: [&str; 12] = ["C", "Db", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum ScaleFamily {
    #[default]
    Major,
    HarmonicMinor,
    MelodicMinor,
    Pentatonic,
}

impl ScaleFamily {
    pub const ALL: [ScaleFamily; 4] = [ScaleFamily::Major, ScaleFamily::HarmonicMinor, ScaleFamily::MelodicMinor, ScaleFamily::Pentatonic];

    pub fn name(self) -> &'static str {
        match self {
            ScaleFamily::Major => "Major",
            ScaleFamily::HarmonicMinor => "Harmonic minor",
            ScaleFamily::MelodicMinor => "Melodic minor",
            ScaleFamily::Pentatonic => "Pentatonic",
        }
    }

    /// The semitones between the degrees of the first mode.
    fn steps(self) -> &'static [i32] {
        match self {
            ScaleFamily::Major => &[2, 2, 1, 2, 2, 2, 1],
            ScaleFamily::HarmonicMinor => &[2, 1, 2, 2, 1, 3, 1],
            ScaleFamily::MelodicMinor => &[2, 1, 2, 2, 2, 2, 1],
            ScaleFamily::Pentatonic => &[2, 2, 3, 2, 3],
        }
    }

    /// The modes, from the one on the first degree; plain ASCII, the UI font has no sharps.
    pub fn modes(self) -> &'static [&'static str] {
        match self {
            ScaleFamily::Major => &["Ionian (major)", "Dorian", "Phrygian", "Lydian", "Mixolydian", "Aeolian (natural minor)", "Locrian"],
            ScaleFamily::HarmonicMinor => {
                &["Harmonic minor", "Locrian nat6", "Ionian #5", "Dorian #4", "Phrygian dominant", "Lydian #2", "Altered bb7"]
            }
            ScaleFamily::MelodicMinor => {
                &["Melodic minor", "Dorian b2", "Lydian augmented", "Lydian dominant", "Mixolydian b6", "Locrian nat2", "Altered"]
            }
            ScaleFamily::Pentatonic => &["Major pentatonic", "Suspended", "Blues minor", "Blues major", "Minor pentatonic"],
        }
    }

    /// The notes of a position on each string.
    pub fn notes_per_string(self) -> usize {
        if self == ScaleFamily::Pentatonic { 2 } else { 3 }
    }
}

/// A scale to practice: its root, its family and its mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Scale {
    /// The pitch class of the root, 0 is C.
    pub root: u8,
    pub family: ScaleFamily,
    /// From 0, the mode on the first degree.
    pub mode: u8,
}

impl Default for Scale {
    fn default() -> Self {
        // G: the first position is low on the neck of a guitar and a bass.
        Scale { root: 7, family: ScaleFamily::Major, mode: 0 }
    }
}

impl Scale {
    /// Back in the ranges after a load.
    pub fn sanitized(self) -> Self {
        let modes = self.family.modes().len() as u8;
        Scale { root: self.root % 12, mode: self.mode % modes, ..self }
    }

    /// The semitones between the degrees of the mode.
    pub fn steps(&self) -> Vec<i32> {
        let steps = self.family.steps();
        let mode = self.mode as usize % steps.len();
        steps[mode..].iter().chain(&steps[..mode]).copied().collect()
    }

    /// The pitch classes of the degrees, the root first.
    pub fn pitch_classes(&self) -> Vec<i32> {
        let mut pc = self.root as i32;
        self.steps()
            .iter()
            .map(|step| {
                let degree = pc;
                pc = (pc + step) % 12;
                degree
            })
            .collect()
    }

    pub fn contains(&self, midi: i32) -> bool {
        self.pitch_classes().contains(&midi.rem_euclid(12))
    }

    pub fn mode_name(&self) -> &'static str {
        let modes = self.family.modes();
        modes[self.mode as usize % modes.len()]
    }

    /// `G Dorian`.
    pub fn name(&self) -> String {
        format!("{} {}", ROOT_NAMES[self.root as usize % 12], self.mode_name())
    }

    /// The key of the progress, `G-major-1`.
    pub fn key(&self) -> String {
        let family = match self.family {
            ScaleFamily::Major => "major",
            ScaleFamily::HarmonicMinor => "harmonic-minor",
            ScaleFamily::MelodicMinor => "melodic-minor",
            ScaleFamily::Pentatonic => "pentatonic",
        };
        format!("{}-{family}-{}", ROOT_NAMES[self.root as usize % 12], self.mode + 1)
    }

    /// The spelling of a note of the scale: the seven-note scales take one letter per degree
    /// from the root's (Bb in F major, not A#); the others, and the double sharps or flats, the
    /// sharps or the flats of the root.
    pub fn spell(&self, midi: i32) -> Spelled {
        let root_name = ROOT_NAMES[self.root as usize % 12];
        let flats = root_name.contains('b') || root_name == "F";
        let fallback = Spelled::from_midi(midi, flats);
        let classes = self.pitch_classes();
        if classes.len() != 7 {
            return fallback;
        }
        let Some(degree) = classes.iter().position(|&pc| pc == midi.rem_euclid(12)) else {
            return fallback;
        };
        let root_letter = LETTERS.iter().position(|&l| root_name.starts_with(l)).unwrap_or(0);
        let letter = (root_letter + degree) % 7;
        let natural = Spelled::natural(letter as u8, 0).midi().rem_euclid(12);
        let alter = (midi.rem_euclid(12) - natural + 6).rem_euclid(12) - 6;
        let accidental = match alter {
            0 => Accidental::Natural,
            1 => Accidental::Sharp,
            -1 => Accidental::Flat,
            _ => return fallback,
        };
        let octave = (midi - alter - natural).div_euclid(12) - 1;
        Spelled { letter: letter as u8, accidental, octave }
    }

    /// The names of the degrees, `G A Bb C D E F`.
    pub fn note_names(&self) -> Vec<String> {
        self.pitch_classes().iter().map(|&pc| self.spell(60 + pc).name()).collect()
    }
}

/// A note of a scale position on the fretboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScaleNote {
    pub position: Position,
    /// 0 for an open string, 1 the index to 4 the pinky.
    pub finger: u8,
}

/// The position of a scale from its root on the lowest string, three notes per string (two for
/// the pentatonics), the lowest note first. A string takes more notes when the next one is tuned
/// far above it (a drop tuning). When the position from the root does not fit on the frets, it
/// starts on a lower note of the scale on the lowest string. `None` when nothing fits.
pub fn position(scale: &Scale, instrument: &Instrument) -> Option<Vec<ScaleNote>> {
    let open = instrument.open.first().copied()?;
    let root_fret = (scale.root as i32 - open).rem_euclid(12);
    let starts = std::iter::once(root_fret).chain((0..root_fret).rev().filter(|f| scale.contains(open + f)));
    starts.into_iter().find_map(|fret| layout(scale, instrument, open + fret))
}

fn layout(scale: &Scale, instrument: &Instrument, start: i32) -> Option<Vec<ScaleNote>> {
    let per_string = scale.family.notes_per_string();
    let mut notes = (start..).filter(|m| scale.contains(*m));
    let mut out = Vec::new();
    let mut next = notes.next()?;
    let strings = instrument.strings();
    for string in 0..strings {
        let mut on_string = Vec::new();
        let above = instrument.open.get(string + 1).copied();
        // The usual count, and every note under the next open string.
        while on_string.len() < per_string || above.is_some_and(|a| next < a) {
            let fret = next - instrument.open[string];
            if fret < 0 || fret > instrument.frets as i32 {
                return None;
            }
            on_string.push(fret as u8);
            next = notes.next()?;
        }
        for (fret, finger) in on_string.iter().zip(fingers(&on_string)) {
            out.push(ScaleNote { position: Position { string, fret: *fret }, finger });
        }
    }
    Some(out)
}

/// The fingers of the notes on a string: the index on the lowest fret, the pinky on the highest
/// when it is 3 frets away or more; one finger per fret from the nut on the open strings.
fn fingers(frets: &[u8]) -> Vec<u8> {
    let Some(&low) = frets.first() else {
        return Vec::new();
    };
    if low == 0 {
        return frets.iter().map(|&f| f.min(4)).collect();
    }
    let span = |f: u8| f - low;
    match frets.len() {
        1 => vec![1],
        2 => vec![
            1,
            match span(frets[1]) {
                1 => 2,
                2 => 3,
                _ => 4,
            },
        ],
        3 => {
            let (middle, top) = (span(frets[1]), span(frets[2]));
            let last = if top >= 3 { 4 } else { 1 + top };
            let second = if middle == 1 || last < 4 || top - middle > 1 { 2 } else { 3 };
            vec![1, second, last]
        }
        n => (1..=n as u8).map(|f| f.min(4)).collect(),
    }
}

/// The groups of a sequence over some notes: `1 2 3, 2 3 4, 3 4 5...` for groups of 3.
pub fn sequence<T: Copy>(notes: &[T], group: usize) -> Vec<T> {
    let group = group.clamp(1, notes.len().max(1));
    notes.windows(group).flatten().copied().collect()
}

/// The notes in windows of `order.len()`, each played in `order` (from 1): 1-3-2-4 over
/// `a b c d e` is `a c b d, b d c e`.
pub fn permuted<T: Copy>(notes: &[T], order: &[u8]) -> Vec<T> {
    let len = order.len();
    if len == 0 || notes.len() < len {
        return notes.to_vec();
    }
    notes.windows(len).flat_map(|w| order.iter().map(move |&i| w[i as usize - 1])).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::music::{InstrumentKind, TuningPreset, midi_name};

    fn guitar() -> Instrument {
        Instrument::new(InstrumentKind::Guitar, 6, TuningPreset::Standard, 12)
    }

    fn names(scale: &Scale) -> String {
        scale.note_names().join(" ")
    }

    #[test]
    fn spelling() {
        assert_eq!(names(&Scale::default()), "G A B C D E F#");
        assert_eq!(names(&Scale { root: 5, ..Scale::default() }), "F G A Bb C D E");
        assert_eq!(names(&Scale { root: 7, mode: 1, ..Scale::default() }), "G A Bb C D E F");
        assert_eq!(names(&Scale { root: 9, family: ScaleFamily::HarmonicMinor, mode: 0 }), "A B C D E F G#");
        assert_eq!(names(&Scale { root: 4, family: ScaleFamily::HarmonicMinor, mode: 4 }), "E F G# A B C D");
        assert_eq!(names(&Scale { root: 9, family: ScaleFamily::Pentatonic, mode: 4 }), "A C D E G");
        assert_eq!(Scale { root: 3, ..Scale::default() }.name(), "Eb Ionian (major)");
        assert_eq!(Scale { root: 7, mode: 1, ..Scale::default() }.key(), "G-major-2");
    }

    #[test]
    fn g_major_three_per_string() {
        let notes = position(&Scale::default(), &guitar()).unwrap();
        assert_eq!(notes.len(), 18);
        let g = guitar();
        let frets: Vec<u8> = notes.iter().map(|n| n.position.fret).collect();
        assert_eq!(frets, [3, 5, 7, 3, 5, 7, 4, 5, 7, 4, 5, 7, 5, 7, 8, 5, 7, 8]);
        let fingers: Vec<u8> = notes.iter().map(|n| n.finger).collect();
        assert_eq!(fingers, [1, 2, 4, 1, 2, 4, 1, 2, 4, 1, 2, 4, 1, 3, 4, 1, 3, 4]);
        assert_eq!(midi_name(g.midi_at(notes[0].position)), "G2");
        // Every note in the scale, going up.
        let midis: Vec<i32> = notes.iter().map(|n| g.midi_at(n.position)).collect();
        assert!(midis.windows(2).all(|w| w[0] < w[1]));
        assert!(midis.iter().all(|m| Scale::default().contains(*m)));
    }

    #[test]
    fn positions_fit() {
        // Every root, family and mode fits on a 12-fret guitar, a 4 and a 5-string bass.
        let instruments = [
            guitar(),
            Instrument::new(InstrumentKind::Bass, 4, TuningPreset::Standard, 12),
            Instrument::tuned(InstrumentKind::Bass, 5, &[-2, 0, 0, 0, 0], 12),
            Instrument::new(InstrumentKind::Guitar, 6, TuningPreset::Drop, 12),
        ];
        for instrument in &instruments {
            for family in ScaleFamily::ALL {
                for mode in 0..family.modes().len() as u8 {
                    for root in 0..12 {
                        let scale = Scale { root, family, mode };
                        let notes = position(&scale, instrument).unwrap_or_else(|| panic!("{} on {:?}", scale.name(), instrument.open));
                        let midis: Vec<i32> = notes.iter().map(|n| instrument.midi_at(n.position)).collect();
                        assert!(midis.windows(2).all(|w| w[0] < w[1]), "{}", scale.name());
                        assert!(notes.iter().all(|n| n.finger <= 4 && n.position.fret <= instrument.frets));
                    }
                }
            }
        }
    }

    #[test]
    fn drop_tuning_takes_more_notes_low() {
        let drop = Instrument::new(InstrumentKind::Guitar, 6, TuningPreset::Drop, 12);
        // D major from the open low D: D E F# G on the low string, A (open) is next.
        let notes = position(&Scale { root: 2, ..Scale::default() }, &drop).unwrap();
        let low: Vec<u8> = notes.iter().filter(|n| n.position.string == 0).map(|n| n.position.fret).collect();
        assert_eq!(low, [0, 2, 4, 5]);
    }

    #[test]
    fn sequences() {
        let n = [1, 2, 3, 4, 5];
        assert_eq!(sequence(&n, 1), [1, 2, 3, 4, 5]);
        assert_eq!(sequence(&n, 2), [1, 2, 2, 3, 3, 4, 4, 5]);
        assert_eq!(sequence(&n, 3), [1, 2, 3, 2, 3, 4, 3, 4, 5]);
        assert_eq!(sequence(&[6, 5, 4, 3], 3), [6, 5, 4, 5, 4, 3]);
        assert_eq!(sequence(&n, 9), n);
        assert_eq!(permuted(&n, &[1, 3, 2, 4]), [1, 3, 2, 4, 2, 4, 3, 5]);
        assert_eq!(permuted(&n, &[2, 1]), [2, 1, 3, 2, 4, 3, 5, 4]);
    }
}
