//! Every exercise of the drills page, in rows: the finger drills, the scales (the chosen scale and
//! mode, as runs and sequences, on adjacent strings or skipping some), the stretches, and the
//! extra exercises, hidden until enough stars are earned. Each exercise has a stable key for the
//! progress. Without Bevy.
//!
//! The first exercise of a row is open when the row opens; the next one when the previous one has
//! a star. A row opens at the start, or when the first three exercises of the row before have a
//! star. The extra rows open together once revealed.
use crate::drills::{
    SHAPES, ShapeKind, Step, distinct_orders, finger_orders, gap_shape, order_name, rotation_class, shape_steps, shift_positions,
    shift_steps,
};
use crate::music::Instrument;
use crate::progress::InstrumentProgress;
use crate::scales::{Scale, position, sequence};

/// The stars earned by the best round: 80%, 90% and every note right.
pub const STAR_ACCURACY: [f32; 3] = [0.8, 0.9, 1.0];
/// The stars, all exercises together, that reveal the extra exercises.
pub const EXTRA_STARS: u32 = 30;
/// The exercises with a star in a row that open the next row.
pub const OPEN_NEXT_ROW_AFTER: usize = 3;
/// The sequences of the scales: runs, then groups of 2 to 6 notes.
pub const SEQUENCES: std::ops::RangeInclusive<u8> = 1..=6;
/// The string skips of the scales on four strings at most; more are extra.
pub const MAX_SKIP: u8 = 2;

pub fn stars(accuracy: f32) -> u8 {
    STAR_ACCURACY.iter().filter(|&&s| accuracy >= s - 1e-4).count() as u8
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Section {
    Fingers,
    Scales,
    Stretches,
    Extra,
}

impl Section {
    pub const ALL: [Section; 4] = [Section::Fingers, Section::Scales, Section::Stretches, Section::Extra];

    pub fn name(self) -> &'static str {
        match self {
            Section::Fingers => "Finger drills",
            Section::Scales => "Scales",
            Section::Stretches => "Stretches",
            Section::Extra => "Extra exercises",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Finger {
        shape: usize,
        order: [u8; 4],
    },
    /// The scale of the catalog in groups of `group` notes, skipping `skip` strings.
    Scale {
        group: u8,
        skip: u8,
    },
    Gaps {
        gap: u8,
    },
    Shifts {
        growth: u8,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Exercise {
    pub key: String,
    pub kind: Kind,
    pub name: String,
    /// The short name on its tile.
    pub label: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Row {
    pub section: Section,
    pub title: String,
    /// Opens when the row before has enough stars, else at the start.
    pub after_previous: bool,
    pub exercises: Vec<Exercise>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Catalog {
    pub rows: Vec<Row>,
    pub scale: Scale,
}

pub fn group_name(group: u8) -> String {
    match group {
        1 => "Run".to_string(),
        g => format!("In {g}s"),
    }
}

/// `123 234 345...` for the tiles.
pub fn group_example(group: u8) -> String {
    match group {
        1 => "12345...".to_string(),
        g => format!(
            "{}...",
            (1..=3u8).map(|start| (start..start + g).map(|n| n.to_string()).collect::<String>()).collect::<Vec<_>>().join(" ")
        ),
    }
}

pub fn skip_name(skip: u8) -> String {
    match skip {
        0 => "Adjacent strings".to_string(),
        1 => "Skipping 1 string".to_string(),
        s => format!("Skipping {s} strings"),
    }
}

pub fn catalog(instrument: &Instrument, scale: Scale) -> Catalog {
    let strings = instrument.strings();
    let mut rows = Vec::new();
    let mut extra = Vec::new();
    for (i, shape) in SHAPES.iter().enumerate() {
        if let ShapeKind::Anchor { skip } = shape.kind
            && strings <= skip
        {
            continue;
        }
        rows.push(Row {
            section: Section::Fingers,
            title: format!("{}. {}", i + 1, shape.name),
            after_previous: i > 0,
            exercises: distinct_orders()
                .into_iter()
                .map(|order| Exercise {
                    key: finger_key(i, order),
                    kind: Kind::Finger { shape: i, order },
                    name: format!("{} · {}", shape.name, order_name(&order)),
                    label: order_name(&order),
                })
                .collect(),
        });
    }
    if position(&scale, instrument).is_some() {
        for skip in 0..strings.saturating_sub(1) as u8 {
            let row = Row {
                section: if skip > MAX_SKIP { Section::Extra } else { Section::Scales },
                title: format!("{}, {}", scale.name(), skip_name(skip).to_lowercase()),
                after_previous: skip > 0 && skip <= MAX_SKIP,
                exercises: SEQUENCES
                    .map(|group| Exercise {
                        key: format!("scale/{}/skip{skip}/in{group}", scale.key()),
                        kind: Kind::Scale { group, skip },
                        name: format!("{} · {} · {}", scale.name(), skip_name(skip), group_name(group)),
                        label: group_name(group),
                    })
                    .collect(),
            };
            if skip > MAX_SKIP { extra.push(row) } else { rows.push(row) }
        }
    }
    let gaps = |range: std::ops::RangeInclusive<u8>| -> Vec<Exercise> {
        range
            .filter(|gap| gap_shape(*gap, instrument.frets).is_some())
            .map(|gap| Exercise {
                key: format!("gaps/{gap}"),
                kind: Kind::Gaps { gap },
                name: format!("Growing gaps · {gap} frets apart"),
                label: format!("Gaps of {gap}"),
            })
            .collect()
    };
    let shifts = |range: std::ops::RangeInclusive<u8>| -> Vec<Exercise> {
        range
            .filter(|growth| shift_positions(*growth, instrument.frets).len() > 1)
            .map(|growth| Exercise {
                key: format!("shifts/{growth}"),
                kind: Kind::Shifts { growth },
                name: format!("Growing shifts · jumps growing by {growth}"),
                label: format!("Shifts +{growth}"),
            })
            .collect()
    };
    for (title, exercises) in [("Growing gaps", gaps(2..=2)), ("Growing shifts", shifts(1..=1))] {
        if !exercises.is_empty() {
            rows.push(Row { section: Section::Stretches, title: title.to_string(), after_previous: false, exercises });
        }
    }
    for (title, exercises) in [("Wider gaps", gaps(3..=5)), ("Bigger shifts", shifts(2..=3))] {
        if !exercises.is_empty() {
            extra.push(Row { section: Section::Extra, title: title.to_string(), after_previous: false, exercises });
        }
    }
    rows.extend(extra);
    Catalog { rows, scale }
}

pub fn finger_key(shape: usize, order: [u8; 4]) -> String {
    format!("fingers/{shape}/{}", order_name(&order))
}

/// The key of a finger drill saved by version 0.2, by its number: 24 orders per shape, now one
/// per loop of rotations.
pub fn legacy_finger_key(id: usize) -> Option<String> {
    let orders = finger_orders();
    let shape = id / orders.len();
    if shape >= SHAPES.len() {
        return None;
    }
    let class = rotation_class(orders[id % orders.len()]);
    let order = distinct_orders().into_iter().find(|o| rotation_class(*o) == class)?;
    Some(finger_key(shape, order))
}

impl Catalog {
    pub fn exercises(&self) -> impl Iterator<Item = &Exercise> {
        self.rows.iter().flat_map(|r| r.exercises.iter())
    }

    pub fn find(&self, key: &str) -> Option<&Exercise> {
        self.exercises().find(|e| e.key == key)
    }

    fn locate(&self, key: &str) -> Option<(usize, usize)> {
        self.rows.iter().enumerate().find_map(|(r, row)| row.exercises.iter().position(|e| e.key == key).map(|i| (r, i)))
    }

    pub fn section_of(&self, key: &str) -> Option<Section> {
        self.locate(key).map(|(r, _)| self.rows[r].section)
    }

    /// Revealed by the stars of every exercise, or by an extra exercise already opened.
    pub fn extra_revealed(&self, progress: &InstrumentProgress) -> bool {
        progress.total_stars() >= EXTRA_STARS
            || self
                .rows
                .iter()
                .filter(|r| r.section == Section::Extra)
                .flat_map(|r| &r.exercises)
                .any(|e| progress.unlocked.contains(&e.key) || progress.exercises.contains_key(&e.key))
    }

    pub fn is_unlocked(&self, key: &str, progress: &InstrumentProgress) -> bool {
        if progress.unlocked.contains(key) {
            return true;
        }
        let Some((r, i)) = self.locate(key) else {
            return false;
        };
        let row = &self.rows[r];
        if row.section == Section::Extra && !self.extra_revealed(progress) {
            return false;
        }
        if i > 0 {
            return progress.done(&row.exercises[i - 1].key);
        }
        if !row.after_previous || r == 0 {
            return true;
        }
        self.rows[r - 1].exercises.iter().take(OPEN_NEXT_ROW_AFTER).all(|e| progress.done(&e.key))
    }

    /// The next open exercise after one, in the order of the page.
    pub fn next_unlocked(&self, key: &str, progress: &InstrumentProgress) -> Option<&Exercise> {
        self.exercises().skip_while(|e| e.key != key).skip(1).find(|e| self.is_unlocked(&e.key, progress))
    }

    /// The open exercise to practice most: the fewest stars, then the lowest accuracy, then the
    /// longest unplayed; an open one never played when none is played yet.
    pub fn weakest(&self, progress: &InstrumentProgress) -> Option<&Exercise> {
        let open = || self.exercises().filter(|e| self.is_unlocked(&e.key, progress));
        open()
            .filter_map(|e| progress.exercises.get(&e.key).filter(|r| r.plays > 0).map(|r| (e, r)))
            .min_by(|(_, a), (_, b)| {
                (stars(a.best_accuracy), a.best_accuracy, a.last_played)
                    .partial_cmp(&(stars(b.best_accuracy), b.best_accuracy, b.last_played))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(e, _)| e)
            .or_else(|| open().next())
    }

    /// The locked exercises that can be opened at random: not in the extra rows before they are
    /// revealed.
    pub fn locked(&self, progress: &InstrumentProgress) -> Vec<&Exercise> {
        let revealed = self.extra_revealed(progress);
        self.rows
            .iter()
            .filter(|r| r.section != Section::Extra || revealed)
            .flat_map(|r| &r.exercises)
            .filter(|e| !self.is_unlocked(&e.key, progress))
            .collect()
    }

    pub fn steps(&self, exercise: &Exercise, instrument: &Instrument, shifts: u8) -> Vec<Step> {
        match exercise.kind {
            Kind::Finger { shape, order } => shape_steps(&SHAPES[shape], order, instrument, shifts),
            Kind::Scale { group, skip } => scale_steps(&self.scale, instrument, group, skip),
            Kind::Gaps { gap } => {
                gap_shape(gap, instrument.frets).map(|s| shape_steps(&s, [1, 2, 3, 4], instrument, shifts)).unwrap_or_default()
            }
            Kind::Shifts { growth } => shift_steps(growth, instrument),
        }
    }
}

/// A scale position on sets of strings `skip` apart (all the sets, one after the other), each up
/// in groups of `group` notes and back down; a run turns at the top note without playing it
/// twice.
pub fn scale_steps(scale: &Scale, instrument: &Instrument, group: u8, skip: u8) -> Vec<Step> {
    let Some(notes) = position(scale, instrument) else {
        return Vec::new();
    };
    let strings = instrument.strings();
    let skip = skip as usize;
    let mut out = Vec::new();
    for first in 0..=skip {
        let set: Vec<usize> = (first..strings).step_by(skip + 1).collect();
        if set.len() < 2 && strings > 1 {
            continue;
        }
        let up: Vec<Step> = notes
            .iter()
            .filter(|n| set.contains(&n.position.string))
            .map(|n| Step { position: n.position, finger: n.finger, note: Some(scale.spell(instrument.midi_at(n.position))) })
            .collect();
        let down: Vec<Step> = up.iter().rev().copied().collect();
        out.extend(sequence(&up, group as usize));
        let back = sequence(&down, group as usize);
        out.extend(back.into_iter().skip(if group == 1 { 1 } else { 0 }));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::music::{InstrumentKind, TuningPreset};
    use crate::progress::DrillRecord;

    fn guitar() -> Instrument {
        Instrument::new(InstrumentKind::Guitar, 6, TuningPreset::Standard, 12)
    }

    fn played(progress: &mut InstrumentProgress, key: &str, accuracy: f32) {
        progress.exercises.insert(key.to_string(), DrillRecord { plays: 1, best_accuracy: accuracy, ..Default::default() });
    }

    #[test]
    fn star_thresholds() {
        assert_eq!(stars(0.79), 0);
        assert_eq!(stars(0.8), 1);
        assert_eq!(stars(0.95), 2);
        assert_eq!(stars(1.0), 3);
    }

    #[test]
    fn examples() {
        assert_eq!(group_example(1), "12345...");
        assert_eq!(group_example(3), "123 234 345...");
    }

    #[test]
    fn rows() {
        let c = catalog(&guitar(), Scale::default());
        let fingers: Vec<&Row> = c.rows.iter().filter(|r| r.section == Section::Fingers).collect();
        assert_eq!(fingers.len(), 8);
        assert!(fingers.iter().all(|r| r.exercises.len() == 6));
        // Scales on adjacent strings and skipping 1 or 2 strings; 3 and 4 are extra on 6 strings.
        let scales: Vec<&Row> = c.rows.iter().filter(|r| r.section == Section::Scales).collect();
        assert_eq!(scales.len(), 3);
        assert!(scales.iter().all(|r| r.exercises.len() == 6));
        let extra: Vec<&str> = c.rows.iter().filter(|r| r.section == Section::Extra).map(|r| r.title.as_str()).collect();
        assert_eq!(extra.len(), 4, "{extra:?}");
        // Unique keys.
        let keys: std::collections::HashSet<&str> = c.exercises().map(|e| e.key.as_str()).collect();
        assert_eq!(keys.len(), c.exercises().count());
        // A 4-string bass: no scale skipping 3 strings; the anchor 3 strings up fits, E to G.
        let bass = catalog(&Instrument::new(InstrumentKind::Bass, 4, TuningPreset::Standard, 12), Scale::default());
        assert!(!bass.rows.iter().any(|r| r.exercises.iter().any(|e| e.kind == Kind::Scale { group: 1, skip: 3 })));
        assert_eq!(bass.rows.iter().filter(|r| r.section == Section::Fingers).count(), 8);
    }

    #[test]
    fn unlocking() {
        let c = catalog(&guitar(), Scale::default());
        let mut p = InstrumentProgress::default();
        let key = |r: usize, i: usize| c.rows[r].exercises[i].key.clone();
        assert!(c.is_unlocked(&key(0, 0), &p));
        assert!(!c.is_unlocked(&key(0, 1), &p));
        // The scales and the stretches open at the start.
        let scale_row = c.rows.iter().position(|r| r.section == Section::Scales).unwrap();
        assert!(c.is_unlocked(&key(scale_row, 0), &p));
        assert!(!c.is_unlocked(&key(scale_row + 1, 0), &p));
        played(&mut p, &key(0, 0), 0.85);
        assert!(c.is_unlocked(&key(0, 1), &p));
        played(&mut p, &key(0, 1), 0.7);
        assert!(!c.is_unlocked(&key(0, 2), &p));
        // Three with a star open the next row.
        played(&mut p, &key(0, 1), 0.9);
        assert!(!c.is_unlocked(&key(1, 0), &p));
        played(&mut p, &key(0, 2), 1.0);
        assert!(c.is_unlocked(&key(1, 0), &p));
        // Opened by hand.
        p.unlocked.insert(key(5, 3));
        assert!(c.is_unlocked(&key(5, 3), &p));
        assert_eq!(c.next_unlocked(&key(0, 2), &p).map(|e| e.key.clone()), Some(key(0, 3)));
        // The extra rows: hidden, then revealed by the stars.
        let extra = c.rows.iter().position(|r| r.section == Section::Extra).unwrap();
        assert!(!c.is_unlocked(&key(extra, 0), &p));
        assert!(c.locked(&p).iter().all(|e| c.section_of(&e.key) != Some(Section::Extra)));
        for (n, e) in c.exercises().take(10).enumerate() {
            played(&mut p, &e.key.clone(), if n < 10 { 1.0 } else { 0.0 });
        }
        assert!(p.total_stars() >= EXTRA_STARS);
        assert!(c.is_unlocked(&key(extra, 0), &p));
    }

    #[test]
    fn weakest() {
        let c = catalog(&guitar(), Scale::default());
        let mut p = InstrumentProgress::default();
        assert_eq!(c.weakest(&p).map(|e| e.key.clone()), Some(c.rows[0].exercises[0].key.clone()));
        played(&mut p, &c.rows[0].exercises[0].key, 1.0);
        played(&mut p, &c.rows[0].exercises[1].key, 0.82);
        assert_eq!(c.weakest(&p).map(|e| e.key.clone()), Some(c.rows[0].exercises[1].key.clone()));
    }

    #[test]
    fn legacy_keys() {
        // Level 0 was 1-2-3-4 of the first shape, 24 + 1 was 4-3-2-1 of the second.
        assert_eq!(legacy_finger_key(0).as_deref(), Some("fingers/0/1-2-3-4"));
        assert_eq!(legacy_finger_key(25).as_deref(), Some("fingers/1/4-3-2-1"));
        assert!(legacy_finger_key(24 * 8).is_none());
        // Every old level maps to one of the 6 drills of its shape.
        let c = catalog(&guitar(), Scale::default());
        assert!((0..24 * 8).all(|id| legacy_finger_key(id).is_some_and(|k| c.find(&k).is_some())));
    }

    #[test]
    fn scale_exercises() {
        let g = guitar();
        let scale = Scale::default();
        // G major up 18 notes and back down without the top twice.
        let run = scale_steps(&scale, &g, 1, 0);
        assert_eq!(run.len(), 18 + 17);
        assert_eq!(run[0].note.map(|n| n.full_name()).as_deref(), Some("G2"));
        assert_ne!(run[17].position, run[18].position);
        // Pairs: 17 pairs up, 17 down.
        assert_eq!(scale_steps(&scale, &g, 2, 0).len(), 2 * 17 * 2);
        // Skipping a string: the low E, D and B strings, then the A, G and high e.
        let skip = scale_steps(&scale, &g, 1, 1);
        assert_eq!(skip.len(), 2 * (9 + 8));
        let first_pass: std::collections::BTreeSet<usize> = skip[..9].iter().map(|s| s.position.string).collect();
        assert_eq!(first_pass.into_iter().collect::<Vec<_>>(), [0, 2, 4]);
        // Every note of a run in the scale, the fingers of the position.
        assert!(run.iter().all(|s| scale.contains(g.midi_at(s.position)) && (1..=4).contains(&s.finger)));
        // Skipping 4 strings on 6: the low and the high E only.
        let wide = scale_steps(&scale, &g, 1, 4);
        assert!(wide.iter().all(|s| s.position.string == 0 || s.position.string == 5));
    }
}
