//! "Find them all", in three levels: the shapes (a tutorial, the next screen), every octave of a
//! note (all of them on the staff, all their places on the fretboard), then one octave (a note on
//! the staff, all its places). The letters of the lines and the spaces stay next to the staff. A
//! pause shows one of the places left, then another.
use bevy::picking::events::PointerClick;
use bevy::prelude::*;

use crate::fretboard::{FretTapped, Mark, MarksLayer, set_marks, spawn_fretboard};
use crate::learn::pick_find;
use crate::music::{Accidental, Clef, Instrument, Position, Spelled, learning_order, staff_hint};
use crate::progress::{NoteStats, Progress};
use crate::screens::Expected;
use crate::screens::learn::spawn_note_header;
use crate::settings::Settings;
use crate::staff::{Letters, StaffDrawing, StaffView, draw_staff};
use crate::ui::{Btn, Fonts, Screen, button, colors, label, screen_root, size, spawn_top_bar};

/// The notes asked in one octave before any is learned.
const MIN_POOL: usize = 5;
/// The notes found in every octave without a mistake that open the level of one octave.
pub const OCTAVES_TO_OPEN: usize = 5;

/// The level played.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FindLevel {
    #[default]
    EveryOctave,
    OneOctave,
}

pub struct FindPlugin;

impl Plugin for FindPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FindLevel>()
            .add_systems(OnEnter(Screen::Find), spawn)
            .add_systems(OnExit(Screen::Find), |mut expected: ResMut<Expected>| *expected = Expected::default())
            .add_systems(Update, (tap, timers, render).chain().run_if(in_state(Screen::Find)));
    }
}

/// The notes of every octave found without a mistake, for the opening of the next level.
pub fn octaves_cleared(stats: &std::collections::BTreeMap<String, NoteStats>) -> usize {
    stats.values().filter(|s| s.correct > 0).count()
}

#[derive(Resource)]
struct Find {
    level: FindLevel,
    instrument: Instrument,
    clef: Clef,
    order: Vec<Spelled>,
    learned: usize,
    rng: fastrand::Rng,
    index: usize,
    /// Every octave of the note asked in the range of the instrument, the lowest first.
    octaves: Vec<Spelled>,
    places: Vec<Position>,
    found: Vec<Position>,
    wrong: Vec<Position>,
    asked_at: f32,
    /// 0 none, then that many places shown.
    hint: u8,
    done_until: Option<f32>,
    /// Notes found in this session, and found without a mistake.
    rounds: u32,
    clean_rounds: u32,
    dirty: bool,
}

impl Find {
    fn note(&self) -> Spelled {
        self.order[self.index]
    }

    /// The name of the stats: the full name for one octave, the letter for every octave.
    fn stat_name(&self) -> String {
        match self.level {
            FindLevel::OneOctave => self.note().full_name(),
            FindLevel::EveryOctave => self.note().name(),
        }
    }

    fn ask(&mut self, previous: Option<usize>, progress: &Progress, now: f32) {
        let record = progress.get(&self.instrument.key()).cloned().unwrap_or_default();
        self.index = match self.level {
            FindLevel::OneOctave => pick_find(&self.order, self.learned, MIN_POOL, previous, &record.finding, &mut self.rng),
            FindLevel::EveryOctave => self.pick_name(previous, &record.finding_octaves),
        };
        let note = self.note();
        let flats = note.accidental == Accidental::Flat;
        self.octaves = match self.level {
            FindLevel::OneOctave => vec![note],
            FindLevel::EveryOctave => (self.instrument.lowest()..=self.instrument.highest())
                .filter(|m| m.rem_euclid(12) == note.midi().rem_euclid(12))
                .map(|m| Spelled::from_midi(m, flats))
                .collect(),
        };
        self.places = self.octaves.iter().flat_map(|n| self.instrument.positions_of(n.midi())).collect();
        self.found.clear();
        self.wrong.clear();
        self.asked_at = now;
        self.hint = 0;
        self.done_until = None;
        self.dirty = true;
    }

    /// A note name for every octave: the ones never found first, then the least accurate, never
    /// the previous one.
    fn pick_name(&mut self, previous: Option<usize>, stats: &std::collections::BTreeMap<String, NoteStats>) -> usize {
        let previous_name = previous.map(|i| self.order[i].name());
        let mut seen: Vec<String> = Vec::new();
        let mut best: Option<(f32, usize)> = None;
        for (i, note) in self.order.iter().enumerate() {
            let name = note.name();
            if seen.contains(&name) || Some(&name) == previous_name.as_ref() {
                continue;
            }
            seen.push(name.clone());
            let score = stats.get(&name).and_then(|s| s.accuracy()).unwrap_or(-1.0) + self.rng.f32() * 0.3;
            if best.is_none_or(|(b, _)| score < b) {
                best = Some((score, i));
            }
        }
        best.map_or(0, |(_, i)| i)
    }
}

#[derive(Component)]
struct InfoText;

#[derive(Component)]
struct HintText;

#[derive(Component)]
struct ScoreText;

fn spawn(
    mut commands: Commands,
    fonts: Res<Fonts>,
    settings: Res<Settings>,
    progress: Res<Progress>,
    mut level: ResMut<FindLevel>,
    time: Res<Time>,
    mut next: ResMut<NextState<Screen>>,
) {
    // The shapes first, the first time.
    if !settings.find_tutorial_done {
        next.set(Screen::FindTutorial);
    }
    let fonts = &*fonts;
    let instrument = settings.instrument();
    let clef = instrument.kind.clef();
    let record = progress.get(&instrument.key()).cloned().unwrap_or_default();
    let one_octave_open = octaves_cleared(&record.finding_octaves) >= OCTAVES_TO_OPEN;
    if !one_octave_open {
        *level = FindLevel::EveryOctave;
    }
    let mut find = Find {
        level: *level,
        instrument: instrument.clone(),
        clef,
        order: learning_order(&instrument, settings.accidentals),
        learned: record.learned,
        rng: fastrand::Rng::new(),
        index: 0,
        octaves: vec![],
        places: vec![],
        found: vec![],
        wrong: vec![],
        asked_at: 0.0,
        hint: 0,
        done_until: None,
        rounds: 0,
        clean_rounds: 0,
        dirty: true,
    };
    find.ask(None, &progress, time.elapsed_secs());
    commands.insert_resource(find);
    commands.spawn(screen_root(Screen::Find)).with_children(|root| {
        spawn_top_bar(root, "Find them all", Screen::Find, fonts);
        root.spawn(Node {
            flex_grow: 1.0,
            min_height: px(0),
            flex_direction: FlexDirection::Column,
            row_gap: px(10),
            padding: UiRect::axes(px(12), px(10)),
            overflow: Overflow::clip_y(),
            ..default()
        })
        .with_children(|body| {
            spawn_levels(body, *level, one_octave_open, OCTAVES_TO_OPEN.saturating_sub(octaves_cleared(&record.finding_octaves)), fonts);
            spawn_note_header(body, clef, |info| {
                info.spawn((label("", fonts.display(size::TITLE), colors::text()), InfoText));
                info.spawn((label("", fonts.text(size::LEAD), colors::reveal()), HintText));
                info.spawn((label("", fonts.text(size::TEXT), colors::dim()), ScoreText));
            });
            spawn_fretboard(body, &instrument, settings.left_handed, fonts);
        });
    });
}

/// The three levels: the shapes, every octave, one octave (locked until enough notes are found
/// in every octave).
fn spawn_levels(body: &mut ChildSpawnerCommands, level: FindLevel, one_octave_open: bool, left: usize, fonts: &Fonts) {
    body.spawn(Node {
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        align_items: AlignItems::Center,
        column_gap: px(8),
        row_gap: px(6),
        ..default()
    })
    .with_children(|row| {
        row.spawn(button("1. The shapes", fonts, colors::button())).observe(|_: On<PointerClick>, mut next: ResMut<NextState<Screen>>| {
            next.set(Screen::FindTutorial);
        });
        for (name, this) in [("2. Every octave", FindLevel::EveryOctave), ("3. One octave", FindLevel::OneOctave)] {
            let open = this == FindLevel::EveryOctave || one_octave_open;
            let color = if this == level {
                colors::selected()
            } else if open {
                colors::button()
            } else {
                colors::locked()
            };
            let mut b = row.spawn(button(name, fonts, color));
            b.insert(Btn { color, hover: if open { colors::button_hover() } else { color } });
            b.observe(move |_: On<PointerClick>, mut level: ResMut<FindLevel>, mut next: ResMut<NextState<Screen>>| {
                if open {
                    *level = this;
                    // The same state: the screen is built again on this level.
                    next.set(Screen::Find);
                }
            });
        }
        if !one_octave_open {
            let notes = if left == 1 { "note" } else { "notes" };
            row.spawn(label(format!("Find {left} more {notes} in every octave to open one octave"), fonts.text(size::TEXT), colors::dim()));
        }
    });
}

fn tap(mut find: ResMut<Find>, mut tapped: MessageReader<FretTapped>, time: Res<Time>, mut progress: ResMut<Progress>) {
    let now = time.elapsed_secs();
    for t in tapped.read() {
        if find.done_until.is_some() {
            break;
        }
        let position = t.0;
        if find.found.contains(&position) || find.wrong.contains(&position) {
            continue;
        }
        find.dirty = true;
        if find.places.contains(&position) {
            find.found.push(position);
        } else {
            find.wrong.push(position);
        }
        if find.found.len() == find.places.len() {
            let clean = find.wrong.is_empty();
            let seconds = now - find.asked_at;
            let hinted = find.hint >= 1;
            let name = find.stat_name();
            let record = progress.of(&find.instrument.key());
            let stats = match find.level {
                FindLevel::OneOctave => &mut record.finding,
                FindLevel::EveryOctave => &mut record.finding_octaves,
            };
            stats.entry(name).or_default().record(clean, seconds, hinted);
            progress.save();
            find.rounds += 1;
            if clean {
                find.clean_rounds += 1;
            }
            find.done_until = Some(now + 1.2);
        }
    }
}

fn timers(mut find: ResMut<Find>, time: Res<Time>, settings: Res<Settings>, progress: Res<Progress>) {
    let now = time.elapsed_secs();
    match find.done_until {
        Some(until) if now >= until => {
            let previous = find.index;
            find.ask(Some(previous), &progress, now);
        }
        None => {
            let hint = (((now - find.asked_at) / settings.hint_delay).floor() as usize).min(find.places.len()) as u8;
            if hint > find.hint {
                find.hint = hint;
                find.dirty = true;
            }
        }
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn render(
    mut find: ResMut<Find>,
    mut commands: Commands,
    fonts: Res<Fonts>,
    staffs: Query<(Entity, &StaffView)>,
    layers: Query<(Entity, &MarksLayer)>,
    mut texts: ParamSet<(Query<&mut Text, With<InfoText>>, Query<&mut Text, With<HintText>>, Query<&mut Text, With<ScoreText>>)>,
    mut expected: ResMut<Expected>,
) {
    if !find.dirty {
        return;
    }
    find.dirty = false;
    let note = find.note();
    let done = find.done_until.is_some();
    let missing: Vec<Position> = find.places.iter().copied().filter(|p| !find.found.contains(p)).collect();
    let next_midi = missing.first().map(|p| find.instrument.midi_at(*p));
    *expected = Expected { midi: next_midi.or(Some(note.midi())), position: missing.first().copied() };
    if let Ok(staff) = staffs.single() {
        let written: Vec<Spelled> = find.octaves.iter().map(|n| n.written()).collect();
        let drawing = StaffDrawing {
            note: written.first().copied(),
            more: written.iter().skip(1).copied().collect(),
            color: done.then_some(colors::ok()),
            // The letters of the lines and the spaces, always.
            letters: Letters::All,
        };
        draw_staff(&mut commands, staff, &drawing, &fonts);
    }
    if let Ok(layer) = layers.single() {
        let mut marks: Vec<Mark> = find.found.iter().map(|&p| Mark::new(p, note.name(), colors::ok())).collect();
        let flats = note.accidental == Accidental::Flat;
        marks.extend(find.wrong.iter().map(|&p| Mark::new(p, Spelled::from_midi(find.instrument.midi_at(p), flats).name(), colors::bad())));
        // One more missing place for each hint.
        marks.extend(missing.iter().take(find.hint as usize).map(|&p| Mark::new(p, "?", colors::reveal()).pulse()));
        set_marks(&mut commands, layer, &marks, &fonts);
    }
    let what = match find.level {
        FindLevel::OneOctave => note.full_name(),
        FindLevel::EveryOctave => format!("Every {}", note.name()),
    };
    let info = if done {
        format!("{what}: all {} found", find.places.len())
    } else {
        format!("{what}: {} of {} found", find.found.len(), find.places.len())
    };
    for mut text in &mut texts.p0() {
        text.0 = info.clone();
    }
    let hint = match find.level {
        FindLevel::OneOctave => staff_hint(find.clef, note.written()),
        FindLevel::EveryOctave => "The same letter an octave apart: 12 frets up the string, or two strings and two frets up".to_string(),
    };
    for mut text in &mut texts.p1() {
        text.0 = hint.clone();
    }
    let score = match find.level {
        FindLevel::OneOctave => {
            let pool = find.learned.max(MIN_POOL).min(find.order.len());
            format!(
                "{} notes found, {} without a mistake. The notes asked are the first {pool} of the learning order.",
                find.rounds, find.clean_rounds
            )
        }
        FindLevel::EveryOctave => format!("{} notes found, {} without a mistake.", find.rounds, find.clean_rounds),
    };
    for mut text in &mut texts.p2() {
        text.0 = score.clone();
    }
}
