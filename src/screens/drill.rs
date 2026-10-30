//! A finger drill: the notes as a tablature and on the fretboard, played on the instrument (heard
//! by the microphone) or tapped on the fretboard. The accuracy and the speed are recorded; a pause
//! longer than the hint delay shows where the next note is and with which finger.
use bevy::picking::events::PointerClick;
use bevy::prelude::*;

use crate::audio::NoteHeard;
use crate::drills::{FINGER_NAMES, Level, Step, is_unlocked, levels, steps};
use crate::fretboard::{FretTapped, Mark, MarksLayer, set_marks, spawn_fretboard};
use crate::music::{Instrument, midi_name};
use crate::progress::Progress;
use crate::screens::Expected;
use crate::screens::drills::{SelectedLevel, shape_description};
use crate::settings::Settings;
use crate::storage::now_unix_seconds;
use crate::ui::{Fonts, Screen, button, colors, label, percent_text, screen_root, spawn_top_bar};

/// The accuracy that completes a level.
pub const GOAL: f32 = 0.9;
/// The columns of the tablature.
const TAB_COLUMNS: usize = 16;

pub struct DrillPlugin;

impl Plugin for DrillPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Drill), spawn)
            .add_systems(OnExit(Screen::Drill), |mut expected: ResMut<Expected>| *expected = Expected::default())
            .add_systems(Update, (play, hesitation, render).chain().run_if(in_state(Screen::Drill)));
    }
}

#[derive(Resource)]
struct Drill {
    level: Level,
    instrument: Instrument,
    steps: Vec<Step>,
    index: usize,
    correct: u32,
    wrong: u32,
    /// The time of the first note, the speed is measured from it.
    started: Option<f32>,
    ended: Option<f32>,
    waiting_since: f32,
    hinted: bool,
    feedback: String,
    dirty: bool,
}

impl Drill {
    fn current(&self) -> Option<Step> {
        self.steps.get(self.index).copied()
    }

    fn accuracy(&self) -> f32 {
        let total = self.correct + self.wrong;
        if total == 0 { 0.0 } else { self.correct as f32 / total as f32 }
    }

    fn notes_per_minute(&self, now: f32) -> f32 {
        let Some(start) = self.started else {
            return 0.0;
        };
        let elapsed = self.ended.unwrap_or(now) - start;
        if elapsed < 0.5 || self.correct < 2 { 0.0 } else { (self.correct - 1) as f32 / elapsed * 60.0 }
    }
}

#[derive(Component)]
struct TabView;

#[derive(Component)]
struct StatusText;

#[derive(Component)]
struct FeedbackText;

#[derive(Component)]
struct Summary;

/// The root of the screen, the summary goes over it.
#[derive(Component)]
struct DrillRoot;

fn spawn(mut commands: Commands, fonts: Res<Fonts>, settings: Res<Settings>, selected: Res<SelectedLevel>, time: Res<Time>) {
    let fonts = &*fonts;
    let all = levels();
    let level = all[selected.0.min(all.len() - 1)];
    let instrument = settings.instrument();
    let steps = steps(&level, &instrument, settings.drill_shifts);
    commands.insert_resource(Drill {
        level,
        instrument: instrument.clone(),
        steps,
        index: 0,
        correct: 0,
        wrong: 0,
        started: None,
        ended: None,
        waiting_since: time.elapsed_secs(),
        hinted: false,
        feedback: String::new(),
        dirty: true,
    });
    commands.spawn((screen_root(Screen::Drill), DrillRoot)).with_children(|root| {
        spawn_top_bar(root, &format!("Drill {}: {}", level.id % 24 + 1, level.name()), Screen::Drill, fonts);
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
            body.spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: px(14),
                row_gap: px(4),
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|row| {
                row.spawn(label(shape_description(level.group), fonts.text(16.0), colors::DIM));
                for (finger, name) in FINGER_NAMES.iter().enumerate().skip(1) {
                    row.spawn((
                        Node { padding: UiRect::axes(px(8), px(2)), border_radius: BorderRadius::all(px(10)), ..default() },
                        BackgroundColor(colors::FINGERS[finger]),
                        children![label(format!("{finger} {name}"), fonts.bold(14.0), Color::WHITE)],
                    ));
                }
            });
            body.spawn((Node { flex_direction: FlexDirection::Column, flex_shrink: 0.0, ..default() }, TabView));
            body.spawn((label("", fonts.bold(18.0), colors::TEXT), StatusText));
            body.spawn((label("", fonts.text(17.0), colors::ACCENT), FeedbackText));
            spawn_fretboard(body, &instrument, settings.left_handed, fonts);
            body.spawn(Node { flex_direction: FlexDirection::Row, column_gap: px(10), ..default() }).with_children(|row| {
                row.spawn(button("Restart", fonts, colors::BUTTON)).observe(restart);
            });
        });
    });
}

fn restart(
    _: On<PointerClick>,
    mut drill: ResMut<Drill>,
    time: Res<Time>,
    mut commands: Commands,
    summaries: Query<Entity, With<Summary>>,
) {
    drill.index = 0;
    drill.correct = 0;
    drill.wrong = 0;
    drill.started = None;
    drill.ended = None;
    drill.waiting_since = time.elapsed_secs();
    drill.hinted = false;
    drill.feedback.clear();
    drill.dirty = true;
    for summary in &summaries {
        commands.entity(summary).despawn();
    }
}

#[allow(clippy::too_many_arguments)]
fn play(
    mut drill: ResMut<Drill>,
    mut heard: MessageReader<NoteHeard>,
    mut tapped: MessageReader<FretTapped>,
    time: Res<Time>,
    settings: Res<Settings>,
    mut progress: ResMut<Progress>,
    mut commands: Commands,
    fonts: Res<Fonts>,
    roots: Query<Entity, With<DrillRoot>>,
) {
    let now = time.elapsed_secs();
    // The answers: a note heard, or a place tapped.
    let mut answers: Vec<(bool, String)> = Vec::new();
    for note in heard.read() {
        let Some(step) = drill.current() else { break };
        let expected = drill.instrument.midi_at(step.position);
        // The previous note still ringing is not a mistake.
        let previous = drill.index.checked_sub(1).map(|i| drill.instrument.midi_at(drill.steps[i].position));
        if (note.midi - expected).abs() % 12 == 0 && (note.midi - expected).abs() <= 12 {
            answers.push((true, String::new()));
        } else if Some(note.midi) != previous {
            answers.push((false, format!("Heard {}, the note is {}", midi_name(note.midi), midi_name(expected))));
        }
    }
    for tap in tapped.read() {
        let Some(step) = drill.current() else { break };
        if tap.0 == step.position {
            answers.push((true, String::new()));
        } else {
            answers.push((
                false,
                format!(
                    "That is {}: string {} fret {}",
                    midi_name(drill.instrument.midi_at(tap.0)),
                    drill.instrument.strings() - tap.0.string,
                    tap.0.fret
                ),
            ));
        }
    }
    for (correct, feedback) in answers {
        if drill.current().is_none() {
            break;
        }
        drill.dirty = true;
        if !correct {
            drill.wrong += 1;
            drill.feedback = feedback;
            continue;
        }
        drill.correct += 1;
        drill.started.get_or_insert(now);
        drill.index += 1;
        drill.waiting_since = now;
        drill.hinted = false;
        drill.feedback.clear();
        if drill.index == drill.steps.len() {
            drill.ended = Some(now);
            finish(&mut drill, now, &settings, &mut progress, &mut commands, &fonts, &roots);
        }
    }
}

fn finish(
    drill: &mut Drill,
    now: f32,
    settings: &Settings,
    progress: &mut Progress,
    commands: &mut Commands,
    fonts: &Fonts,
    roots: &Query<Entity, With<DrillRoot>>,
) {
    let accuracy = drill.accuracy();
    let speed = drill.notes_per_minute(now);
    let key = settings.instrument().key();
    let record = progress.of(&key).drills.entry(drill.level.id).or_default();
    let first_time_done = !record.done && accuracy >= GOAL;
    record.plays += 1;
    record.done |= accuracy >= GOAL;
    record.best_accuracy = record.best_accuracy.max(accuracy);
    record.best_notes_per_minute = record.best_notes_per_minute.max(speed);
    record.last_accuracy = accuracy;
    record.last_notes_per_minute = speed;
    record.last_played = now_unix_seconds();
    progress.save();
    let instrument_progress = progress.of(&key).clone();
    let all = levels();
    let next = all.iter().skip(drill.level.id + 1).find(|l| is_unlocked(l, |id| instrument_progress.drill_done(id))).map(|l| l.id);
    let verdict = if accuracy >= GOAL {
        if first_time_done { "Level done! The next one is open." } else { "Goal reached." }
    } else {
        "Under 90%: once more, slower and cleaner."
    };
    let text = format!(
        "{verdict}\nAccuracy {} ({} right, {} wrong) · {:.0} notes per minute",
        percent_text(accuracy),
        drill.correct,
        drill.wrong,
        speed
    );
    let Some(root) = roots.iter().next() else { return };
    commands.entity(root).with_children(|root| {
        root.spawn((
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK.with_alpha(0.6)),
            Summary,
        ))
        .with_children(|overlay| {
            overlay
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(14),
                        padding: UiRect::all(px(24)),
                        border_radius: BorderRadius::all(px(12)),
                        max_width: percent(92),
                        ..default()
                    },
                    BackgroundColor(colors::PANEL),
                ))
                .with_children(|panel| {
                    let color = if accuracy >= GOAL { colors::OK } else { colors::BAD };
                    panel.spawn(label(text, fonts.bold(20.0), color));
                    panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: px(10), ..default() }).with_children(|row| {
                        row.spawn(button("Again", fonts, colors::BUTTON)).observe(restart);
                        if let Some(next) = next {
                            row.spawn(button("Next drill", fonts, colors::SELECTED)).observe(
                                move |_: On<PointerClick>, mut selected: ResMut<SelectedLevel>, mut state: ResMut<NextState<Screen>>| {
                                    selected.0 = next;
                                    // The same state: the drill screen is left and entered again.
                                    state.set(Screen::Drill);
                                },
                            );
                        }
                        row.spawn(button("Levels", fonts, colors::BUTTON)).observe(
                            |_: On<PointerClick>, mut state: ResMut<NextState<Screen>>| {
                                state.set(Screen::Drills);
                            },
                        );
                    });
                });
        });
    });
}

fn hesitation(mut drill: ResMut<Drill>, time: Res<Time>, settings: Res<Settings>) {
    if drill.current().is_some() && !drill.hinted && time.elapsed_secs() - drill.waiting_since > settings.hint_delay {
        drill.hinted = true;
        drill.dirty = true;
    }
}

#[allow(clippy::too_many_arguments)]
fn render(
    mut drill: ResMut<Drill>,
    mut commands: Commands,
    fonts: Res<Fonts>,
    settings: Res<Settings>,
    time: Res<Time>,
    layers: Query<(Entity, &MarksLayer)>,
    tabs: Query<Entity, With<TabView>>,
    mut status: Query<&mut Text, (With<StatusText>, Without<FeedbackText>)>,
    mut feedback: Query<&mut Text, (With<FeedbackText>, Without<StatusText>)>,
    mut expected: ResMut<Expected>,
) {
    if !drill.dirty {
        return;
    }
    drill.dirty = false;
    let now = time.elapsed_secs();
    let current = drill.current();
    *expected = Expected { midi: current.map(|s| drill.instrument.midi_at(s.position)), position: current.map(|s| s.position) };
    // The four notes of the current shape on the fretboard.
    if let Ok(layer) = layers.single() {
        let cell = (drill.index.min(drill.steps.len().saturating_sub(1)) / 4) * 4;
        let marks: Vec<Mark> = drill.steps[cell..(cell + 4).min(drill.steps.len())]
            .iter()
            .enumerate()
            .map(|(i, step)| {
                let text = if settings.show_note_names {
                    crate::music::Spelled::from_midi(drill.instrument.midi_at(step.position), false).name()
                } else {
                    step.finger.to_string()
                };
                let i = cell + i;
                let color = colors::FINGERS[step.finger as usize];
                if i < drill.index {
                    Mark::new(step.position, text, colors::OK)
                } else if i == drill.index {
                    let mark = Mark::new(step.position, text, color).big().ring(Color::WHITE);
                    if drill.hinted { mark.pulse() } else { mark }
                } else {
                    Mark::new(step.position, text, color.with_alpha(0.45))
                }
            })
            .collect();
        set_marks(&mut commands, layer, &marks, &fonts);
    }
    if let Ok(tab) = tabs.single() {
        draw_tab(&mut commands, tab, &drill, &fonts);
    }
    let total = drill.steps.len();
    let status_text = format!(
        "Note {}/{} · accuracy {} · {:.0} notes/min",
        (drill.index + 1).min(total),
        total,
        if drill.correct + drill.wrong == 0 { "-".to_string() } else { percent_text(drill.accuracy()) },
        drill.notes_per_minute(now)
    );
    for mut text in &mut status {
        text.0 = status_text.clone();
    }
    let feedback_text = match current {
        Some(step) if drill.hinted => format!(
            "{} finger on fret {} of the {} string ({})",
            capitalize(FINGER_NAMES[step.finger as usize]),
            step.position.fret,
            drill.instrument.string_label(step.position.string),
            midi_name(drill.instrument.midi_at(step.position))
        ),
        Some(_) if drill.feedback.is_empty() && drill.index == 0 => "Play the notes, or tap them on the fretboard".to_string(),
        _ => drill.feedback.clone(),
    };
    for mut text in &mut feedback {
        text.0 = feedback_text.clone();
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    c.next().map(|f| f.to_uppercase().chain(c).collect()).unwrap_or_default()
}

/// The tablature around the current note: the highest string on top, the frets to play in order.
fn draw_tab(commands: &mut Commands, tab: Entity, drill: &Drill, fonts: &Fonts) {
    commands.entity(tab).despawn_children();
    let start = drill.index.saturating_sub(4);
    let end = (start + TAB_COLUMNS).min(drill.steps.len());
    let strings = drill.instrument.strings();
    commands.entity(tab).with_children(|tab| {
        for string in (0..strings).rev() {
            tab.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, height: px(20), ..default() })
                .with_children(|row| {
                    row.spawn((
                        Node { width: px(22), ..default() },
                        children![label(drill.instrument.string_label(string), fonts.text(14.0), colors::DIM)],
                    ));
                    for i in start..start + TAB_COLUMNS {
                        let step = drill.steps.get(i).filter(|_| i < end);
                        let on_string = step.filter(|s| s.position.string == string);
                        let bg = if i == drill.index { colors::SELECTED.with_alpha(0.35) } else { Color::NONE };
                        row.spawn((
                            Node {
                                width: px(32),
                                height: percent(100),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(bg),
                        ))
                        .with_children(|cell| {
                            // The string line.
                            cell.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    width: percent(100),
                                    height: px(1),
                                    top: percent(50),
                                    ..default()
                                },
                                BackgroundColor(colors::DIM.with_alpha(0.6)),
                            ));
                            if let Some(step) = on_string {
                                let color = if i < drill.index { colors::OK } else { colors::FINGERS[step.finger as usize] };
                                cell.spawn((
                                    Node { padding: UiRect::horizontal(px(3)), ..default() },
                                    BackgroundColor(colors::BG),
                                    children![label(step.position.fret.to_string(), fonts.bold(15.0), color)],
                                ));
                            }
                        });
                    }
                });
        }
    });
}
