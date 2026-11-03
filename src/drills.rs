//! The finger drills: the four fingers of the fretting hand in every order, on spans that grow
//! little by little, then with the index anchored on one string and the other fingers on another;
//! and the stretches, with gaps or shifts that grow.
//!
//! A drill climbs the strings from the lowest to the highest, moves one fret up the neck and comes
//! back down with the fingers in the reverse order. The orders that are rotations of each other
//! (1-2-3-4, 2-3-4-1, 3-4-1-2, 4-1-2-3) are the same loop once repeated: one of them is a drill.
use crate::music::{Instrument, Position, Spelled};

/// Index, middle, ring and pinky.
pub const FINGER_NAMES: [&str; 5] = ["open", "index", "middle", "ring", "pinky"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    /// The four fingers on one string.
    OneString,
    /// The index on a string, the three other fingers `skip` strings higher.
    Anchor { skip: usize },
}

/// A group of drills: one hand shape, every finger order.
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

/// One note of a drill.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Step {
    pub position: Position,
    /// 0 an open string, 1 the index, 4 the pinky.
    pub finger: u8,
    /// The spelling in the key of a scale, the sharps otherwise.
    pub note: Option<Spelled>,
}

impl Step {
    pub fn new(position: Position, finger: u8) -> Self {
        Step { position, finger, note: None }
    }
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

/// The same loop: the smallest of the rotations of an order.
pub fn rotation_class(order: [u8; 4]) -> [u8; 4] {
    (0..4)
        .map(|r| {
            let mut o = order;
            o.rotate_left(r);
            o
        })
        .min()
        .unwrap_or(order)
}

/// The 6 orders that are not rotations of each other, each as its easiest rotation.
pub fn distinct_orders() -> Vec<[u8; 4]> {
    let mut seen = Vec::new();
    finger_orders()
        .into_iter()
        .filter(|o| {
            let class = rotation_class(*o);
            let new = !seen.contains(&class);
            seen.push(class);
            new
        })
        .collect()
}

pub fn order_name(order: &[u8; 4]) -> String {
    order.iter().map(|f| f.to_string()).collect::<Vec<_>>().join("-")
}

/// The notes of a shape in a finger order on an instrument: up the strings at the start fret, one
/// fret higher and down the strings with the fingers in the reverse order. `shifts` repeats the
/// up and down further up the neck.
pub fn shape_steps(shape: &Shape, order: [u8; 4], instrument: &Instrument, shifts: u8) -> Vec<Step> {
    let strings = instrument.strings();
    let reversed = {
        let mut r = order;
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
                    push_cell(&mut out, string, string, up_fret, shape, &order);
                }
                for string in (0..strings).rev() {
                    push_cell(&mut out, string, string, down_fret, shape, &reversed);
                }
            }
            ShapeKind::Anchor { skip } => {
                let anchors = strings.saturating_sub(skip);
                for anchor in 0..anchors {
                    push_cell(&mut out, anchor, anchor + skip, up_fret, shape, &order);
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
        out.push(Step::new(position, finger));
    }
}

/// The four fingers `gap` frets apart on one string, high enough on the neck where the frets are
/// closer; `None` when it does not fit on the frets.
pub fn gap_shape(gap: u8, frets: u8) -> Option<Shape> {
    let span = 3 * gap;
    (span < frets).then(|| Shape {
        name: "Growing gaps",
        kind: ShapeKind::OneString,
        offsets: [0, gap, 2 * gap, 3 * gap],
        start_fret: frets.saturating_sub(span + 1).clamp(1, 5),
    })
}

/// The frets of the index for the shifts: from fret 1, jumps of 1 + `growth`, 1 + 2 `growth`...
/// while the four frets fit on the board.
pub fn shift_positions(growth: u8, frets: u8) -> Vec<u8> {
    let mut positions = vec![1u8];
    let mut jump = 1 + growth;
    while let Some(&last) = positions.last() {
        let next = last + jump;
        if next + 3 > frets {
            break;
        }
        positions.push(next);
        jump += growth;
    }
    positions
}

/// One finger per fret moving up the neck with growing jumps on the lowest string, down on the
/// next one with the fingers reversed, and so on across the strings.
pub fn shift_steps(growth: u8, instrument: &Instrument) -> Vec<Step> {
    let positions = shift_positions(growth, instrument.frets);
    let mut out = Vec::new();
    for string in 0..instrument.strings() {
        let up = string % 2 == 0;
        let order: [u8; 4] = if up { [1, 2, 3, 4] } else { [4, 3, 2, 1] };
        let frets: Vec<u8> = if up { positions.clone() } else { positions.iter().rev().copied().collect() };
        for fret in frets {
            for finger in order {
                out.push(Step::new(Position { string, fret: fret + finger - 1 }, finger));
            }
        }
    }
    out
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
        // 24 orders, 4 rotations each.
        let distinct = distinct_orders();
        assert_eq!(distinct.len(), 6);
        assert_eq!(distinct[..2], [[1, 2, 3, 4], [4, 3, 2, 1]]);
        assert_eq!(rotation_class([3, 4, 1, 2]), [1, 2, 3, 4]);
        let classes: std::collections::HashSet<_> = distinct.iter().map(|o| rotation_class(*o)).collect();
        assert_eq!(classes.len(), 6);
    }

    #[test]
    fn first_drill() {
        let steps = shape_steps(&SHAPES[0], [1, 2, 3, 4], &guitar(), 1);
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
        // The index on fret 1, the middle on 3, ring 4, pinky 5: nothing on fret 2.
        let stretch = shape_steps(&SHAPES[1], [1, 2, 3, 4], &guitar(), 1);
        let frets: Vec<u8> = stretch[..4].iter().map(|s| s.position.fret).collect();
        assert_eq!(frets, [1, 3, 4, 5]);
        // The index stays on a string, the others on the next one.
        let anchor = shape_steps(&SHAPES[4], [1, 2, 3, 4], &guitar(), 1);
        assert_eq!(anchor[0].position, Position { string: 0, fret: 1 });
        assert!(anchor[1..4].iter().all(|s| s.position.string == 1));
        // Five anchors up on six strings, five down.
        assert_eq!(anchor.len(), 5 * 4 * 2);
        // Skipping two strings on a 4-string bass: the anchor on the E and the A only.
        let bass = Instrument::new(InstrumentKind::Bass, 4, TuningPreset::Standard, 12);
        let skip = shape_steps(&SHAPES[5], [1, 2, 3, 4], &bass, 1);
        assert_eq!(skip.len(), 2 * 4 * 2);
    }

    #[test]
    fn shifts_stay_on_the_board() {
        let short = guitar();
        for shape in &SHAPES {
            for order in distinct_orders() {
                for step in shape_steps(shape, order, &short, 6) {
                    assert!(step.position.fret <= short.frets, "{} fret {}", shape.name, step.position.fret);
                }
            }
        }
    }

    #[test]
    fn gaps_and_shifts() {
        let gap = gap_shape(2, 12).unwrap();
        assert_eq!(gap.offsets, [0, 2, 4, 6]);
        assert_eq!(gap.start_fret, 5);
        let steps = shape_steps(&gap, [1, 2, 3, 4], &guitar(), 1);
        assert!(steps.iter().all(|s| s.position.fret <= 12));
        assert_eq!(gap_shape(3, 12).unwrap().start_fret, 2);
        assert!(gap_shape(4, 12).is_none());
        assert!(gap_shape(4, 24).is_some());
        // Jumps of 2, 3, then 4 would leave the 12 frets.
        assert_eq!(shift_positions(1, 12), [1, 3, 6]);
        assert_eq!(shift_positions(1, 24), [1, 3, 6, 10, 15, 21]);
        assert_eq!(shift_positions(2, 12), [1, 4, 9]);
        let steps = shift_steps(1, &guitar());
        assert_eq!(steps.len(), 6 * 3 * 4);
        // Up the neck on the low string, down on the next one.
        assert_eq!(steps[11].position, Position { string: 0, fret: 9 });
        assert_eq!(steps[12].position, Position { string: 1, fret: 9 });
        assert_eq!(steps[12].finger, 4);
    }
}
