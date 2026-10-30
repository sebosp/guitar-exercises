//! "Learn notes": a note on the staff, played on the instrument or tapped on the fretboard.
//!
//! A new note is introduced with its name, its mnemonic and every place on the fretboard, then
//! asked again and again with reviews of the older notes in between, until it is answered right
//! a few times in a row without a hint. A pause shows the hints step by step: the letters of the
//! staff, the name, the places on the fretboard.
use bevy::prelude::*;

use crate::audio::NoteHeard;
use crate::fretboard::{FretTapped, Mark, MarksLayer, set_marks, spawn_fretboard};
use crate::learn::{Question, Scheduler};
use crate::music::{Clef, Instrument, Position, Spelled, learning_order, midi_name, staff_hint};
use crate::progress::Progress;
use crate::screens::Expected;
use crate::settings::Settings;
use crate::staff::{Letters, StaffDrawing, StaffView, draw_staff, spawn_staff};
use crate::ui::{Fonts, Screen, colors, label, screen_root, spawn_top_bar};

pub struct LearnPlugin;

impl Plugin for LearnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Learn), spawn)
            .add_systems(OnExit(Screen::Learn), |mut expected: ResMut<Expected>| *expected = Expected::default())
            .add_systems(Update, (answer, timers, render).chain().run_if(in_state(Screen::Learn)));
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Phase {
    Asking,
    /// The answer is shown until then.
    Showing {
        until: f32,
    },
}

#[derive(Resource)]
struct Learn {
    instrument: Instrument,
    clef: Clef,
    scheduler: Scheduler,
    rng: fastrand::Rng,
    question: Question,
    asked_at: f32,
    phase: Phase,
    /// A wrong answer was given to this question.
    missed: bool,
    /// 0 none, 1 the letters of the staff, 2 the name, 3 the places.
    hint: u8,
    wrong_taps: Vec<Position>,
    /// The wrong answers to this question, played or tapped.
    misses: u32,
    message: String,
    message_color: Color,
    dirty: bool,
}

impl Learn {
    fn note(&self) -> Spelled {
        self.scheduler.order[self.question.index()]
    }
}

#[derive(Component)]
struct NameText;

#[derive(Component)]
struct HintText;

#[derive(Component)]
struct MessageText;

#[derive(Component)]
struct ProgressText;

#[derive(Component)]
struct LearnedRow;

pub fn spawn_note_header(parent: &mut ChildSpawnerCommands, clef: Clef, extra: impl FnOnce(&mut ChildSpawnerCommands)) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(18),
            row_gap: px(8),
            align_items: AlignItems::Center,
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|row| {
            spawn_staff(row, clef);
            row.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(8), flex_grow: 1.0, flex_basis: px(260), ..default() })
                .with_children(extra);
        });
}

fn spawn(mut commands: Commands, fonts: Res<Fonts>, settings: Res<Settings>, progress: Res<Progress>, time: Res<Time>) {
    let fonts = &*fonts;
    let instrument = settings.instrument();
    let clef = instrument.kind.clef();
    let learned = progress.get(&instrument.key()).map(|p| p.learned).unwrap_or(0);
    let order = learning_order(&instrument, settings.accidentals);
    let mut scheduler = Scheduler::new(order, learned, settings.clear_streak);
    let mut rng = fastrand::Rng::new();
    let stats = progress.get(&instrument.key()).map(|p| p.reading.clone()).unwrap_or_default();
    let question = scheduler.next(&stats, &mut rng);
    commands.insert_resource(Learn {
        instrument: instrument.clone(),
        clef,
        scheduler,
        rng,
        question,
        asked_at: time.elapsed_secs(),
        phase: Phase::Asking,
        missed: false,
        hint: 0,
        wrong_taps: vec![],
        misses: 0,
        message: String::new(),
        message_color: colors::TEXT,
        dirty: true,
    });
    commands.spawn(screen_root(Screen::Learn)).with_children(|root| {
        spawn_top_bar(root, "Learn notes", Screen::Learn, fonts);
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
                info.spawn((label("", fonts.bold(34.0), colors::TEXT), NameText));
                info.spawn((label("", fonts.text(17.0), colors::REVEAL), HintText));
                info.spawn((label("", fonts.bold(18.0), colors::TEXT), MessageText));
                info.spawn((label("", fonts.text(15.0), colors::DIM), ProgressText));
            });
            body.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: px(6),
                    row_gap: px(4),
                    flex_shrink: 0.0,
                    ..default()
                },
                LearnedRow,
            ));
            spawn_fretboard(body, &instrument, settings.left_handed, fonts);
        });
    });
}

fn answer(
    mut learn: ResMut<Learn>,
    mut heard: MessageReader<NoteHeard>,
    mut tapped: MessageReader<FretTapped>,
    time: Res<Time>,
    mut progress: ResMut<Progress>,
) {
    let mut answers: Vec<(i32, Option<Position>)> = heard.read().map(|n| (n.midi, None)).collect();
    answers.extend(tapped.read().map(|t| (learn.instrument.midi_at(t.0), Some(t.0))));
    if learn.phase != Phase::Asking {
        return;
    }
    let now = time.elapsed_secs();
    let key = learn.instrument.key();
    for (midi, tap) in answers {
        let note = learn.note();
        learn.dirty = true;
        let index = learn.question.index();
        if midi != note.midi() {
            if let Some(tap) = tap {
                learn.wrong_taps.push(tap);
            }
            learn.message = format!("That is {}. Try again.", midi_name(midi));
            learn.message_color = colors::BAD;
            if matches!(learn.question, Question::Ask(_)) && !learn.missed {
                learn.missed = true;
                learn.scheduler.answer(index, false);
                progress.of(&key).reading.entry(note.full_name()).or_default().record(false, 0.0, learn.hint > 0);
                progress.save();
            }
            // Two misses: the places are shown.
            learn.misses += 1;
            if learn.misses >= 2 {
                learn.hint = 3;
            }
            continue;
        }
        let seconds = now - learn.asked_at;
        learn.message_color = colors::OK;
        match learn.question {
            Question::Introduce(_) => {
                learn.message = format!("Yes, that is {}.", note.full_name());
            }
            Question::Ask(_) => {
                let clean = !learn.missed && learn.hint == 0;
                if !learn.missed {
                    progress.of(&key).reading.entry(note.full_name()).or_default().record(true, seconds, learn.hint > 0);
                }
                let cleared = learn.scheduler.answer(index, clean);
                if cleared {
                    progress.of(&key).learned = learn.scheduler.learned;
                    learn.message = format!("{} cleared! A new note comes.", note.full_name());
                } else if clean {
                    learn.message = format!("Right, {} in {:.1} s.", note.full_name(), seconds);
                } else {
                    learn.message = format!("Right, {}.", note.full_name());
                }
                progress.save();
            }
        }
        learn.phase = Phase::Showing { until: now + 1.3 };
        break;
    }
}

fn timers(mut learn: ResMut<Learn>, time: Res<Time>, settings: Res<Settings>, progress: Res<Progress>) {
    let now = time.elapsed_secs();
    match learn.phase {
        Phase::Showing { until } if now >= until => {
            let stats = progress.get(&learn.instrument.key()).map(|p| p.reading.clone()).unwrap_or_default();
            let Learn { scheduler, rng, .. } = &mut *learn;
            let question = scheduler.next(&stats, rng);
            learn.question = question;
            learn.phase = Phase::Asking;
            learn.asked_at = now;
            learn.missed = false;
            learn.hint = 0;
            learn.wrong_taps.clear();
            learn.misses = 0;
            learn.message.clear();
            learn.dirty = true;
        }
        Phase::Asking if matches!(learn.question, Question::Ask(_)) => {
            let waited = now - learn.asked_at;
            let hint = (waited / settings.hint_delay).floor().min(3.0) as u8;
            if hint > learn.hint {
                learn.hint = hint;
                learn.dirty = true;
            }
        }
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn render(
    mut learn: ResMut<Learn>,
    mut commands: Commands,
    fonts: Res<Fonts>,
    staffs: Query<(Entity, &StaffView)>,
    layers: Query<(Entity, &MarksLayer)>,
    rows: Query<Entity, With<LearnedRow>>,
    mut texts: ParamSet<(
        Query<&mut Text, With<NameText>>,
        Query<&mut Text, With<HintText>>,
        Query<(&mut Text, &mut TextColor), With<MessageText>>,
        Query<&mut Text, With<ProgressText>>,
    )>,
    mut expected: ResMut<Expected>,
) {
    if !learn.dirty {
        return;
    }
    learn.dirty = false;
    let note = learn.note();
    let written = note.written();
    let introducing = matches!(learn.question, Question::Introduce(_));
    let showing = matches!(learn.phase, Phase::Showing { .. });
    let places = learn.instrument.positions_of(note.midi());
    *expected = Expected { midi: Some(note.midi()), position: places.first().copied() };

    let letters = if introducing || learn.hint >= 1 || showing { Letters::All } else { Letters::None };
    let color = if showing { Some(colors::OK) } else { None };
    if let Ok(staff) = staffs.single() {
        draw_staff(&mut commands, staff, &StaffDrawing { note: Some(written), color, letters }, &fonts);
    }
    if let Ok(layer) = layers.single() {
        let reveal = introducing || showing || learn.hint >= 3;
        let mut marks: Vec<Mark> = Vec::new();
        if reveal {
            let fill = if showing { colors::OK } else { colors::REVEAL };
            marks.extend(places.iter().map(|&p| {
                let mark = Mark::new(p, note.name(), fill);
                if learn.hint >= 3 && !showing { mark.pulse() } else { mark }
            }));
        }
        for &tap in &learn.wrong_taps {
            let name = Spelled::from_midi(learn.instrument.midi_at(tap), note.accidental == crate::music::Accidental::Flat).name();
            marks.push(Mark::new(tap, name, colors::BAD));
        }
        set_marks(&mut commands, layer, &marks, &fonts);
    }
    let name = if introducing || showing || learn.hint >= 2 {
        format!("{}  (written {})", note.full_name(), written.full_name())
    } else {
        "Which note is this?".to_string()
    };
    for mut text in &mut texts.p0() {
        text.0 = name.clone();
    }
    let hint = if introducing {
        format!(
            "New note. {}. It is in {} place{} on the fretboard: play it, or tap one.",
            staff_hint(learn.clef, written),
            places.len(),
            if places.len() == 1 { "" } else { "s" }
        )
    } else if learn.hint >= 1 || showing {
        staff_hint(learn.clef, written)
    } else {
        "Play it, or tap one of its places on the fretboard".to_string()
    };
    for mut text in &mut texts.p1() {
        text.0 = hint.clone();
    }
    let (message, message_color) = (learn.message.clone(), learn.message_color);
    for (mut text, mut color) in &mut texts.p2() {
        text.0 = message.clone();
        color.0 = message_color;
    }
    let total = learn.scheduler.order.len();
    let progress_text = match learn.scheduler.new_note() {
        Some(new) => format!(
            "{} of {} notes learned · {} right in a row on {} ({} needed)",
            learn.scheduler.learned,
            total,
            learn.scheduler.streak,
            learn.scheduler.order[new].full_name(),
            learn.scheduler.clear_streak
        ),
        None => format!("All {total} notes learned: reviewing the hardest ones"),
    };
    for mut text in &mut texts.p3() {
        text.0 = progress_text.clone();
    }
    // The learned notes, the new one in color.
    if let Ok(row) = rows.single() {
        commands.entity(row).despawn_children();
        let shown = (learn.scheduler.learned + 1).min(total);
        let current = learn.question.index();
        let new = learn.scheduler.new_note();
        commands.entity(row).with_children(|row| {
            for (i, n) in learn.scheduler.order[..shown].iter().enumerate() {
                let bg = if Some(i) == new { colors::SELECTED } else { colors::PANEL };
                let border = if i == current && showing { colors::ACCENT } else { Color::NONE };
                row.spawn((
                    Node {
                        padding: UiRect::axes(px(7), px(2)),
                        border: UiRect::all(px(2)),
                        border_radius: BorderRadius::all(px(6)),
                        ..default()
                    },
                    BackgroundColor(bg),
                    BorderColor::all(border),
                    children![label(n.full_name(), fonts.text(14.0), colors::TEXT)],
                ));
            }
        });
    }
}
