//! The finger drills: the four fingers of the fretting hand in every order, on spans that grow
//! little by little, then with the index anchored on one string and the other fingers on another.
//!
//! A drill climbs the strings from the lowest to the highest, moves one fret up the neck and comes
//! back down with the fingers in the reverse order. The levels are grouped by shape, easiest
//! first; inside a group the finger orders go from 1-2-3-4 to the awkward ones.
use crate::music::{Instrument, Position};

/// Index, middle, ring and pinky.
pub const FINGER_NAMES: [&str; 5] = ["open", "index", "middle", "ring", "pinky"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    /// The four fingers on one string.
    OneString,
    /// The index on a string, the three other fingers `skip` strings higher.
    Anchor { skip: usize },
}

/// A group of levels: one hand shape, every finger order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shape {
    pub name: &'static str,
    pub kind: ShapeKind,
    /// The fret of each finger, from the index, relative to the index.
    pub offsets: [u8; 4],
    /// The fret of the index at the start: wide spans start higher, where the frets are closer.
    pub start_fret: u8,
}

pub const SHAPES: [Shape; 8] = [
    Shape { name: "One finger per fret", kind: ShapeKind::OneString, offsets: [0, 1, 2, 3], start_fret: 1 },
    Shape { name: "Index stretch", kind: ShapeKind::OneString, offsets: [0, 2, 3, 4], start_fret: 1 },
    Shape { name: "Pinky stretch", kind: ShapeKind::OneString, offsets: [0, 1, 2, 4], start_fret: 1 },
    Shape { name: "Middle-ring stretch", kind: ShapeKind::OneString, offsets: [0, 1, 3, 4], start_fret: 1 },
    Shape { name: "Index anchor, next string", kind: ShapeKind::Anchor { skip: 1 }, offsets: [0, 1, 2, 3], start_fret: 1 },
    Shape { name: "Index anchor, skip a string", kind: ShapeKind::Anchor { skip: 2 }, offsets: [0, 1, 2, 3], start_fret: 3 },
    Shape { name: "Wide span", kind: ShapeKind::OneString, offsets: [0, 2, 4, 5], start_fret: 5 },
    Shape { name: "Index anchor, skip two strings", kind: ShapeKind::Anchor { skip: 3 }, offsets: [0, 1, 2, 3], start_fret: 5 },
];

/// The levels of a group done before the next group opens.
pub const OPEN_NEXT_GROUP_AFTER: usize = 3;

/// One note of a drill.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Step {
    pub position: Position,
    /// 1 is the index, 4 the pinky.
    pub finger: u8,
}

/// The 24 orders of the four fingers, the easiest first: 1-2-3-4 and 4-3-2-1, then the orders
/// with the smallest jumps between the fingers.
pub fn finger_orders() -> Vec<[u8; 4]> {
    let mut orders = Vec::with_capacity(24);
    for a in 1..=4u8 {
        for b in 1..=4u8 {
            for c in 1..=4u8 {
                for d in 1..=4u8 {
                    let o = [a, b, c, d];
                    if (1..=4).all(|f| o.contains(&f)) {
                        orders.push(o);
                    }
                }
            }
        }
    }
    let cost = |o: &[u8; 4]| -> u32 {
        let jumps: u32 = o.windows(2).map(|w| w[0].abs_diff(w[1]) as u32).sum();
        // The ascending and the descending runs first.
        if *o == [1, 2, 3, 4] {
            0
        } else if *o == [4, 3, 2, 1] {
            1
        } else {
            jumps * 10
        }
    };
    orders.sort_by_key(cost);
    orders
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Level {
    /// The index in the flat list of levels, the key of the progress.
    pub id: usize,
    pub group: usize,
    pub order: [u8; 4],
}

impl Level {
    pub fn shape(&self) -> &'static Shape {
        &SHAPES[self.group]
    }

    pub fn order_name(&self) -> String {
        self.order.iter().map(|f| f.to_string()).collect::<Vec<_>>().join("-")
    }

    pub fn name(&self) -> String {
        format!("{} · {}", self.shape().name, self.order_name())
    }
}

/// Every level: the groups in order, every finger order inside a group.
pub fn levels() -> Vec<Level> {
    let orders = finger_orders();
    let mut levels = Vec::with_capacity(SHAPES.len() * orders.len());
    for group in 0..SHAPES.len() {
        for order in &orders {
            levels.push(Level { id: levels.len(), group, order: *order });
        }
    }
    levels
}

/// Whether a level can be played: the first one, the next one of a done level, or the first of a
/// group when enough levels of the previous group are done.
pub fn is_unlocked(level: &Level, done: impl Fn(usize) -> bool) -> bool {
    let per_group = 24;
    let index_in_group = level.id % per_group;
    if level.id == 0 {
        return true;
    }
    if index_in_group == 0 {
        let previous_group = level.id - per_group;
        return (previous_group..previous_group + OPEN_NEXT_GROUP_AFTER).all(&done);
    }
    done(level.id - 1)
}

/// The notes of a level on an instrument: up the strings at the start fret, one fret higher and
/// down the strings with the fingers in the reverse order. `shifts` repeats the up and down
/// further up the neck.
pub fn steps(level: &Level, instrument: &Instrument, shifts: u8) -> Vec<Step> {
    let shape = level.shape();
    let strings = instrument.strings();
    let reversed = {
        let mut r = level.order;
        r.reverse();
        r
    };
    // The highest fret of a finger stays on the board.
    let span = shape.offsets[3];
    let max_start = instrument.frets.saturating_sub(span).max(1);
    let mut out = Vec::new();
    for shift in 0..shifts.max(1) {
        let up_fret = (shape.start_fret + 2 * shift).min(max_start);
        let down_fret = (up_fret + 1).min(max_start);
        match shape.kind {
            ShapeKind::OneString => {
                for string in 0..strings {
                    push_cell(&mut out, string, string, up_fret, shape, &level.order);
                }
                for string in (0..strings).rev() {
                    push_cell(&mut out, string, string, down_fret, shape, &reversed);
                }
            }
            ShapeKind::Anchor { skip } => {
                let anchors = strings.saturating_sub(skip);
                for anchor in 0..anchors {
                    push_cell(&mut out, anchor, anchor + skip, up_fret, shape, &level.order);
                }
                for anchor in (0..anchors).rev() {
                    push_cell(&mut out, anchor, anchor + skip, down_fret, shape, &reversed);
                }
            }
        }
    }
    out
}

/// The four notes of a shape: the index on `index_string`, the other fingers on `other_string`.
fn push_cell(out: &mut Vec<Step>, index_string: usize, other_string: usize, fret: u8, shape: &Shape, order: &[u8; 4]) {
    for &finger in order {
        let string = if finger == 1 { index_string } else { other_string };
        let position = Position { string, fret: fret + shape.offsets[finger as usize - 1] };
        out.push(Step { position, finger });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::music::{InstrumentKind, TuningPreset};

    fn guitar() -> Instrument {
        Instrument::new(InstrumentKind::Guitar, 6, TuningPreset::Standard, 12)
    }

    #[test]
    fn orders() {
        let orders = finger_orders();
        assert_eq!(orders.len(), 24);
        assert_eq!(orders[0], [1, 2, 3, 4]);
        assert_eq!(orders[1], [4, 3, 2, 1]);
        let unique: std::collections::HashSet<_> = orders.iter().collect();
        assert_eq!(unique.len(), 24);
    }

    #[test]
    fn first_drill() {
        let level = levels()[0];
        let steps = steps(&level, &guitar(), 1);
        assert_eq!(steps.len(), 48);
        // Low E string, frets 1 2 3 4 with the index to the pinky.
        let first: Vec<(usize, u8, u8)> = steps[..4].iter().map(|s| (s.position.string, s.position.fret, s.finger)).collect();
        assert_eq!(first, [(0, 1, 1), (0, 2, 2), (0, 3, 3), (0, 4, 4)]);
        // At the top, one fret up and back down from the pinky.
        assert_eq!(steps[24].position, Position { string: 5, fret: 5 });
        assert_eq!(steps[24].finger, 4);
        // Never the same note twice in a row.
        let g = guitar();
        assert!(steps.windows(2).all(|w| g.midi_at(w[0].position) != g.midi_at(w[1].position)));
    }

    #[test]
    fn stretch_and_anchor() {
        let all = levels();
        // The index on fret 1, the middle on 3, ring 4, pinky 5: nothing on fret 2.
        let stretch = steps(&all[24], &guitar(), 1);
        let frets: Vec<u8> = stretch[..4].iter().map(|s| s.position.fret).collect();
        assert_eq!(frets, [1, 3, 4, 5]);
        // The index stays on a string, the others on the next one.
        let anchor = steps(&all[4 * 24], &guitar(), 1);
        assert_eq!(anchor[0].position, Position { string: 0, fret: 1 });
        assert!(anchor[1..4].iter().all(|s| s.position.string == 1));
        // Five anchors up on six strings, five down.
        assert_eq!(anchor.len(), 5 * 4 * 2);
        // Skipping two strings on a 4-string bass: the anchor on the E and the A only.
        let bass = Instrument::new(InstrumentKind::Bass, 4, TuningPreset::Standard, 12);
        let skip = steps(&all[5 * 24], &bass, 1);
        assert_eq!(skip.len(), 2 * 4 * 2);
    }

    #[test]
    fn unlocking() {
        let all = levels();
        let none = |_| false;
        assert!(is_unlocked(&all[0], none));
        assert!(!is_unlocked(&all[1], none));
        assert!(is_unlocked(&all[1], |id| id == 0));
        assert!(!is_unlocked(&all[24], |id| id < 2));
        assert!(is_unlocked(&all[24], |id| id < 3));
    }

    #[test]
    fn shifts_stay_on_the_board() {
        let all = levels();
        let short = Instrument::new(InstrumentKind::Guitar, 6, TuningPreset::Standard, 12);
        for level in &all {
            for step in steps(level, &short, 6) {
                assert!(step.position.fret <= short.frets, "{} fret {}", level.name(), step.position.fret);
            }
        }
    }
}
