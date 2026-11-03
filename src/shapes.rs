//! The shapes that find a note again on the fretboard, the tutorial of "Find them all": the same
//! note on the next string, the octaves two and three strings over, twelve frets up. They come
//! from the tuning of the instrument, a drop or a custom tuning has its own. Without Bevy.
use crate::music::{Instrument, Position, Spelled};

/// A page of the tutorial: what it shows, the notes to tap and the lines between them.
#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub title: String,
    pub text: String,
    /// A short way to remember it.
    pub mnemonic: String,
    /// The notes shown to start from.
    pub shown: Vec<Position>,
    /// The notes to find and tap.
    pub targets: Vec<Position>,
    pub links: Vec<(Position, Position)>,
}

/// The same note on the next string up, `None` past the nut or the last string.
pub fn on_next_string(instrument: &Instrument, p: Position) -> Option<Position> {
    let gap = instrument.open.get(p.string + 1)? - instrument.open[p.string];
    let fret = p.fret as i32 - gap;
    (0..=instrument.frets as i32).contains(&fret).then_some(Position { string: p.string + 1, fret: fret as u8 })
}

/// The octave above `strings` strings up (0: the same string).
pub fn octave_up(instrument: &Instrument, p: Position, strings: usize) -> Option<Position> {
    let gap = instrument.open.get(p.string + strings)? - instrument.open[p.string];
    let fret = p.fret as i32 + 12 - gap;
    (0..=instrument.frets as i32).contains(&fret).then_some(Position { string: p.string + strings, fret: fret as u8 })
}

/// The semitones between each string and the next.
fn gaps(instrument: &Instrument) -> Vec<i32> {
    instrument.open.windows(2).map(|w| w[1] - w[0]).collect()
}

fn name_of(instrument: &Instrument, p: Position) -> String {
    Spelled::from_midi(instrument.midi_at(p), false).name()
}

fn frets_word(n: i32) -> String {
    if n.abs() == 1 { format!("{} fret", n.abs()) } else { format!("{} frets", n.abs()) }
}

/// The pages for an instrument, the ones its strings and frets allow.
pub fn pages(instrument: &Instrument) -> Vec<Page> {
    let strings = instrument.strings();
    let gaps = gaps(instrument);
    // The usual gap, 5 frets, and the strings tuned otherwise (the B string of a guitar).
    let usual = gaps.iter().copied().max_by_key(|g| gaps.iter().filter(|x| *x == g).count()).unwrap_or(5);
    let label = |s: usize| instrument.string_label(s);
    let mut pages = Vec::new();

    // The same note on the next string: a chain from the lowest string.
    let mut chain = vec![Position { string: 0, fret: (usual as u8 * 2).min(instrument.frets) }];
    while let Some(next) = chain.last().and_then(|p| on_next_string(instrument, *p)) {
        chain.push(next);
    }
    if chain.len() > 1 {
        let start = chain[0];
        pages.push(Page {
            title: "The same note on the next string".into(),
            text: format!(
                "The {} at fret {} of the {} string is also {} lower on the next string, and again on the one after.",
                name_of(instrument, start),
                start.fret,
                label(0),
                frets_word(usual)
            ),
            mnemonic: format!("{} back, one string over: the way you tune at the {}th fret.", frets_word(usual), usual),
            shown: vec![start],
            targets: chain[1..].to_vec(),
            links: chain.windows(2).map(|w| (w[0], w[1])).collect(),
        });
    }

    // The strings tuned with another gap.
    if let Some(s) = gaps.iter().position(|g| *g != usual) {
        let gap = gaps[s];
        let from = Position { string: s, fret: (gap.max(0) as u8 + 2).min(instrument.frets) };
        if let Some(to) = on_next_string(instrument, from) {
            pages.push(Page {
                title: format!("The {} string is {}", label(s + 1), frets_word(gap)),
                text: format!(
                    "From the {} string to the {} string the same note is {} lower, not {}.",
                    label(s),
                    label(s + 1),
                    frets_word(gap),
                    usual
                ),
                mnemonic: format!("Crossing to the {} string, one fret off.", label(s + 1)),
                shown: vec![from],
                targets: vec![to],
                links: vec![(from, to)],
            });
        }
    }

    // The octaves two and three strings over, from the lowest strings.
    for (over, title, mnemonic) in [
        (2, "An octave: two strings over, two frets up", "Two strings, two frets; one more fret when you cross the B string."),
        (3, "An octave: three strings over, three frets down", "Three strings over, three frets back."),
    ] {
        let pairs: Vec<(Position, Position)> = (0..strings)
            .filter_map(|s| {
                let from = Position { string: s, fret: 3 + over as u8 / 3 * 2 };
                octave_up(instrument, from, over).map(|to| (from, to))
            })
            .take(2)
            .collect();
        if pairs.is_empty() {
            continue;
        }
        let (from, to) = pairs[0];
        pages.push(Page {
            title: title.into(),
            text: format!(
                "The {} at fret {} of the {} string is an octave higher at fret {} of the {} string.",
                name_of(instrument, from),
                from.fret,
                label(from.string),
                to.fret,
                label(to.string)
            ),
            mnemonic: mnemonic.into(),
            shown: pairs.iter().map(|(f, _)| *f).collect(),
            targets: pairs.iter().map(|(_, t)| *t).collect(),
            links: pairs,
        });
    }

    // Twelve frets up the same string.
    let from = Position { string: 1.min(strings - 1), fret: 2 };
    if let Some(to) = octave_up(instrument, from, 0) {
        pages.push(Page {
            title: "An octave: twelve frets up the same string".into(),
            text: format!(
                "Fret {} of the {} string and fret {} are the same note an octave apart: the double dots of the 12th fret start the neck again.",
                from.fret,
                label(from.string),
                to.fret
            ),
            mnemonic: "Twelve frets, the double dots: the neck starts again.".into(),
            shown: vec![from],
            targets: vec![to],
            links: vec![(from, to)],
        });
    }

    // Together: every C.
    let c: Vec<Position> =
        (instrument.lowest()..=instrument.highest()).filter(|m| m.rem_euclid(12) == 0).flat_map(|m| instrument.positions_of(m)).collect();
    if let Some(&first) = c.iter().min_by_key(|p| (instrument.midi_at(**p), p.string)) {
        let targets: Vec<Position> = c.iter().copied().filter(|p| *p != first).collect();
        // Each one linked from a note found with a shape: the next string or an octave.
        let links: Vec<(Position, Position)> = targets
            .iter()
            .filter_map(|t| {
                c.iter()
                    .copied()
                    .find(|f| {
                        f != t && (on_next_string(instrument, *f) == Some(*t) || (0..=3).any(|o| octave_up(instrument, *f, o) == Some(*t)))
                    })
                    .map(|f| (f, *t))
            })
            .collect();
        pages.push(Page {
            title: "Together: every C".into(),
            text: "From one C, the shapes find all the others: the next string, the octaves over two and three strings, twelve frets up."
                .into(),
            mnemonic: "One note found, the shapes give you the rest.".into(),
            shown: vec![first],
            targets,
            links,
        });
    }
    pages
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::music::{InstrumentKind, TuningPreset};

    fn guitar() -> Instrument {
        Instrument::new(InstrumentKind::Guitar, 6, TuningPreset::Standard, 12)
    }

    #[test]
    fn shapes_of_a_guitar() {
        let g = guitar();
        // Fret 5 of the low E is the open A.
        assert_eq!(on_next_string(&g, Position { string: 0, fret: 5 }), Some(Position { string: 1, fret: 0 }));
        // The G string to the B string: 4 frets.
        assert_eq!(on_next_string(&g, Position { string: 3, fret: 4 }), Some(Position { string: 4, fret: 0 }));
        // G2 at the 3rd fret of the low E: G3 at the 5th fret of the D string.
        assert_eq!(octave_up(&g, Position { string: 0, fret: 3 }, 2), Some(Position { string: 2, fret: 5 }));
        // Crossing the B string: 3 frets up.
        assert_eq!(octave_up(&g, Position { string: 2, fret: 2 }, 2), Some(Position { string: 4, fret: 5 }));
        // Three strings over, three frets back.
        assert_eq!(octave_up(&g, Position { string: 0, fret: 5 }, 3), Some(Position { string: 3, fret: 2 }));
        assert_eq!(octave_up(&g, Position { string: 1, fret: 3 }, 0), Some(Position { string: 1, fret: 15 }).filter(|p| p.fret <= 12));
    }

    #[test]
    fn pages_are_right() {
        for instrument in [
            guitar(),
            Instrument::new(InstrumentKind::Bass, 4, TuningPreset::Standard, 12),
            Instrument::tuned(InstrumentKind::Bass, 5, &[-2, 0, 0, 0, 0], 12),
            Instrument::new(InstrumentKind::Guitar, 6, TuningPreset::Drop, 12),
        ] {
            let pages = pages(&instrument);
            assert!(pages.len() >= 4, "{:?}", instrument.open);
            for page in &pages {
                // Every note to find is the note shown or its octaves, linked from one.
                let pc = |p: &Position| instrument.midi_at(*p).rem_euclid(12);
                assert!(page.targets.iter().all(|t| page.shown.iter().any(|s| pc(s) == pc(t))), "{}", page.title);
                assert!(!page.targets.is_empty() && !page.links.is_empty(), "{}", page.title);
                assert!(page.targets.iter().chain(&page.shown).all(|p| p.fret <= instrument.frets), "{}", page.title);
            }
        }
        // The guitar has its B string page, the 4-string bass none.
        assert!(pages(&guitar()).iter().any(|p| p.title.contains("B string")));
        let bass = Instrument::new(InstrumentKind::Bass, 4, TuningPreset::Standard, 12);
        assert!(!pages(&bass).iter().any(|p| p.title.contains("string is")));
    }
}
