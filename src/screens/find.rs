//! "Find them all": a note on the staff, tap every place it is on the fretboard. The notes come
//! from the ones learned in "Learn notes" (the first five at least). A pause shows the letters of
//! the staff, then one of the places left.
use bevy::prelude::*;

use crate::fretboard::{FretTapped, Mark, MarksLayer, set_marks, spawn_fretboard};
use crate::learn::pick_find;
use crate::music::{Accidental, Clef, Instrument, Position, Spelled, learning_order, staff_hint};
use crate::progress::Progress;
use crate::screens::Expected;
use crate::screens::learn::spawn_note_header;
use crate::settings::Settings;
use crate::staff::{Letters, StaffDrawing, StaffView, draw_staff};
use crate::ui::{Fonts, Screen, colors, label, screen_root, spawn_top_bar};

/// The notes asked before any is learned.
const MIN_POOL: usize = 5;

pub struct FindPlugin;

impl Plugin for FindPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Find), spawn)
            .add_systems(OnExit(Screen::Find), |mut expected: ResMut<Expected>| *expected = Expected::default())
            .add_systems(Update, (tap, timers, render).chain().run_if(in_state(Screen::Find)));
    }
}

#[derive(Resource)]
struct Find {
    instrument: Instrument,
    clef: Clef,
    order: Vec<Spelled>,
    learned: usize,
    rng: fastrand::Rng,
    index: usize,
    places: Vec<Position>,
    found: Vec<Position>,
    wrong: Vec<Position>,
    asked_at: f32,
    /// 0 none, 1 the letters of the staff, 2 and more: that many places shown.
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

    fn ask(&mut self, previous: Option<usize>, progress: &Progress, now: f32) {
        let stats = progress.get(&self.instrument.key()).map(|p| p.finding.clone()).unwrap_or_default();
        self.index = pick_find(&self.order, self.learned, MIN_POOL, previous, &stats, &mut self.rng);
        self.places = self.instrument.positions_of(self.note().midi());
        self.found.clear();
        self.wrong.clear();
        self.asked_at = now;
        self.hint = 0;
        self.done_until = None;
        self.dirty = true;
    }
}

#[derive(Component)]
struct InfoText;

#[derive(Component)]
struct HintText;

#[derive(Component)]
struct ScoreText;

fn spawn(mut commands: Commands, fonts: Res<Fonts>, settings: Res<Settings>, progress: Res<Progress>, time: Res<Time>) {
    let fonts = &*fonts;
    let instrument = settings.instrument();
    let clef = instrument.kind.clef();
    let learned = progress.get(&instrument.key()).map(|p| p.learned).unwrap_or(0);
    let mut find = Find {
        instrument: instrument.clone(),
        clef,
        order: learning_order(&instrument, settings.accidentals),
        learned,
        rng: fastrand::Rng::new(),
        index: 0,
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
            spawn_note_header(body, clef, |info| {
                info.spawn((label("", fonts.bold(26.0), colors::TEXT), InfoText));
                info.spawn((label("", fonts.text(17.0), colors::REVEAL), HintText));
                info.spawn((label("", fonts.text(15.0), colors::DIM), ScoreText));
            });
            spawn_fretboard(body, &instrument, settings.left_handed, fonts);
        });
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
            let hinted = find.hint >= 2;
            let name = find.note().full_name();
            progress.of(&find.instrument.key()).finding.entry(name).or_default().record(clean, seconds, hinted);
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
            let hint = (((now - find.asked_at) / settings.hint_delay).floor() as usize).min(1 + find.places.len()) as u8;
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
    let written = note.written();
    let done = find.done_until.is_some();
    let missing: Vec<Position> = find.places.iter().copied().filter(|p| !find.found.contains(p)).collect();
    *expected = Expected { midi: Some(note.midi()), position: missing.first().copied() };
    if let Ok(staff) = staffs.single() {
        let letters = if find.hint >= 1 || done { Letters::All } else { Letters::None };
        draw_staff(&mut commands, staff, &StaffDrawing { note: Some(written), color: done.then_some(colors::OK), letters }, &fonts);
    }
    if let Ok(layer) = layers.single() {
        let mut marks: Vec<Mark> = find.found.iter().map(|&p| Mark::new(p, note.name(), colors::OK)).collect();
        let flats = note.accidental == Accidental::Flat;
        marks.extend(find.wrong.iter().map(|&p| Mark::new(p, Spelled::from_midi(find.instrument.midi_at(p), flats).name(), colors::BAD)));
        // From the second hint on, one more missing place each time.
        let shown = (find.hint as usize).saturating_sub(1);
        marks.extend(missing.iter().take(shown).map(|&p| Mark::new(p, "?", colors::REVEAL).pulse()));
        set_marks(&mut commands, layer, &marks, &fonts);
    }
    let info = if done {
        format!("{}: all {} found", note.full_name(), find.places.len())
    } else if find.hint >= 2 {
        format!("{}: {} of {} found", note.full_name(), find.found.len(), find.places.len())
    } else {
        format!("Tap every place of this note: {} of {} found", find.found.len(), find.places.len())
    };
    for mut text in &mut texts.p0() {
        text.0 = info.clone();
    }
    let hint = if find.hint >= 1 || done { staff_hint(find.clef, written) } else { String::new() };
    for mut text in &mut texts.p1() {
        text.0 = hint.clone();
    }
    let pool = find.learned.max(MIN_POOL).min(find.order.len());
    let score = format!(
        "{} notes found, {} without a mistake · notes asked: the first {} of the learning order",
        find.rounds, find.clean_rounds, pool
    );
    for mut text in &mut texts.p2() {
        text.0 = score.clone();
    }
}
