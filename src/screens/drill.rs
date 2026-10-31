//! A finger drill: the notes as a tablature, on a staff and on the fretboard, played on the
//! instrument (heard by the microphone) or tapped on the fretboard. The accuracy and the speed are
//! recorded; a pause longer than the hint delay shows where the next note is and with which
//! finger.
//!
//! With the metronome the notes are also checked in time, and the tempo of the drill goes up to
//! its target round after round, see [`crate::tempo`].
use bevy::color::Mix;
use bevy::ecs::system::SystemParam;
use bevy::picking::events::PointerClick;
use bevy::prelude::*;

use crate::audio::NoteHeard;
use crate::drills::{FINGER_NAMES, Level, Step, is_unlocked, levels, steps};
use crate::fretboard::{FretTapped, Mark, MarksLayer, set_marks, spawn_fretboard};
use crate::metronome::{BEATS_PER_BAR, Metronome};
use crate::music::{Instrument, Spelled, midi_name};
use crate::progress::Progress;
use crate::screens::Expected;
use crate::screens::drills::{SelectedLevel, shape_description};
use crate::settings::Settings;
use crate::staff::{StaffDrawing, StaffView, draw_staff, spawn_staff};
use crate::storage::now_unix_seconds;
use crate::tempo::{BPM_STEP, TIMING_GOAL, TempoChange, after_round, notes_in_time, set_tempo, tempo_of};
use crate::ui::{
    Btn, Fonts, Screen, button, button_frame, colors, label, percent_text, screen_root, scroll_area, small_button, spawn_top_bar,
};

/// The accuracy that completes a level.
pub const GOAL: f32 = 0.9;
/// The columns of the tablature.
const TAB_COLUMNS: usize = 16;

pub struct DrillPlugin;

impl Plugin for DrillPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Drill), spawn)
            .add_systems(OnExit(Screen::Drill), |mut expected: ResMut<Expected>, mut metronome: ResMut<Metronome>| {
                *expected = Expected::default();
                metronome.on = false;
            })
            .add_systems(Update, ((play, hesitation, render).chain(), metronome_panel).run_if(in_state(Screen::Drill)));
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
    /// The real times of the right notes, with the metronome.
    hits: Vec<f32>,
    /// The tempo of this round and the target of the drill.
    bpm: u32,
    target: u32,
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

    /// The share of the notes in time with the metronome, `None` before the first one.
    fn in_time(&self, notes_per_beat: u8) -> Option<f32> {
        let slot = 60.0 / self.bpm.max(1) as f32 / notes_per_beat.max(1) as f32;
        (!self.hits.is_empty()).then(|| notes_in_time(&self.hits, slot) as f32 / self.hits.len() as f32)
    }
}

#[derive(Component)]
struct TabView;

#[derive(Component)]
struct StatusText;

#[derive(Component)]
struct FeedbackText;

/// The name of the current note under the staff.
#[derive(Component)]
struct NoteCaption;

#[derive(Component)]
struct Summary;

/// The root of the screen, the summary goes over it.
#[derive(Component)]
struct DrillRoot;

/// The texts of the metronome panel.
#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum PanelText {
    Toggle,
    Bpm,
    Target,
    NotesPerBeat,
    Plan,
}

/// The metronome on/off button.
#[derive(Component)]
struct ToggleButton;

/// A beat of the bar, lit when it clicks.
#[derive(Component)]
struct BeatDot(u64);

/// What a new round needs: the drill, the metronome and the times.
#[derive(SystemParam)]
struct Round<'w, 's> {
    drill: ResMut<'w, Drill>,
    metronome: ResMut<'w, Metronome>,
    time: Res<'w, Time>,
    real: Res<'w, Time<Real>>,
    commands: Commands<'w, 's>,
    summaries: Query<'w, 's, Entity, With<Summary>>,
}

impl Round<'_, '_> {
    /// The drill from its first note, the metronome from the first beat of a bar.
    fn restart(&mut self) {
        let drill = &mut *self.drill;
        drill.index = 0;
        drill.correct = 0;
        drill.wrong = 0;
        drill.started = None;
        drill.ended = None;
        drill.waiting_since = self.time.elapsed_secs();
        drill.hinted = false;
        drill.feedback.clear();
        drill.hits.clear();
        drill.dirty = true;
        self.metronome.bpm = drill.bpm;
        self.metronome.restart(self.real.elapsed_secs_f64());
        for summary in &self.summaries {
            self.commands.entity(summary).despawn();
        }
    }

    /// A new tempo or target for this drill, saved; the round starts again.
    fn retempo(&mut self, progress: &mut Progress, settings: &Settings, bpm: u32, target: u32) {
        let record = progress.of(&settings.instrument().key()).drills.entry(self.drill.level.id).or_default();
        set_tempo(record, bpm, target);
        (self.drill.bpm, self.drill.target) = tempo_of(record);
        progress.save();
        self.restart();
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn(
    mut commands: Commands,
    fonts: Res<Fonts>,
    settings: Res<Settings>,
    selected: Res<SelectedLevel>,
    progress: Res<Progress>,
    mut metronome: ResMut<Metronome>,
    time: Res<Time>,
    real: Res<Time<Real>>,
) {
    let fonts = &*fonts;
    let all = levels();
    let level = all[selected.0.min(all.len() - 1)];
    let instrument = settings.instrument();
    let steps = steps(&level, &instrument, settings.drill_shifts);
    let record = progress.get(&instrument.key()).and_then(|p| p.drills.get(&level.id)).cloned().unwrap_or_default();
    let (bpm, target) = tempo_of(&record);
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
        hits: Vec::new(),
        bpm,
        target,
        dirty: true,
    });
    metronome.on = settings.metronome;
    metronome.bpm = bpm;
    metronome.restart(real.elapsed_secs_f64());
    commands.spawn((screen_root(Screen::Drill), DrillRoot)).with_children(|root| {
        spawn_top_bar(root, &format!("Drill {}: {}", level.id % 24 + 1, level.name()), Screen::Drill, fonts);
        root.spawn(scroll_area()).observe(crate::ui::scroll_on_wheel).observe(crate::ui::scroll_on_drag).with_children(|body| {
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
            spawn_metronome_panel(body, fonts);
            // The note three ways: the tablature, the staff, and what the microphone hears.
            body.spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: px(16),
                row_gap: px(10),
                align_items: AlignItems::Center,
                flex_shrink: 0.0,
                ..default()
            })
            .with_children(|row| {
                row.spawn((Node { flex_direction: FlexDirection::Column, flex_shrink: 0.0, ..default() }, TabView));
                row.spawn(Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: px(4), ..default() })
                    .with_children(|column| {
                        spawn_staff(column, instrument.kind.clef());
                        column.spawn((label("", fonts.bold(16.0), colors::TEXT), NoteCaption));
                    });
                crate::tuner::spawn_tuner(row, fonts, false);
            });
            body.spawn((label("", fonts.bold(18.0), colors::TEXT), StatusText));
            body.spawn((label("", fonts.text(17.0), colors::ACCENT), FeedbackText));
            spawn_fretboard(body, &instrument, settings.left_handed, fonts);
            body.spawn(Node { flex_direction: FlexDirection::Row, column_gap: px(10), flex_shrink: 0.0, ..default() }).with_children(
                |row| {
                    row.spawn(button("Restart", fonts, colors::BUTTON)).observe(|_: On<PointerClick>, mut round: Round| round.restart());
                },
            );
        });
    });
}

/// Metronome on/off, the tempo, the target, the notes per beat and the beats of the bar.
fn spawn_metronome_panel(body: &mut ChildSpawnerCommands, fonts: &Fonts) {
    body.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            align_items: AlignItems::Center,
            column_gap: px(10),
            row_gap: px(8),
            padding: UiRect::axes(px(12), px(8)),
            border_radius: BorderRadius::all(px(8)),
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundColor(colors::PANEL),
    ))
    .with_children(|panel| {
        panel
            .spawn((button_frame(colors::BUTTON), ToggleButton))
            .with_children(|b| {
                b.spawn((label("", fonts.text(17.0), colors::TEXT), PanelText::Toggle));
            })
            .observe(|_: On<PointerClick>, mut settings: ResMut<Settings>, mut round: Round| {
                settings.metronome = !settings.metronome;
                settings.save();
                round.metronome.on = settings.metronome;
                round.restart();
            });
        let stepper = |panel: &mut ChildSpawnerCommands,
                       name: &str,
                       text: PanelText,
                       down: fn(&mut Round, &mut Progress, &mut Settings),
                       up: fn(&mut Round, &mut Progress, &mut Settings)| {
            panel.spawn(label(name, fonts.text(16.0), colors::DIM));
            panel.spawn(small_button("-", fonts)).observe(
                move |_: On<PointerClick>, mut round: Round, mut progress: ResMut<Progress>, mut settings: ResMut<Settings>| {
                    down(&mut round, &mut progress, &mut settings)
                },
            );
            panel.spawn((
                Node { min_width: px(74), justify_content: JustifyContent::Center, ..default() },
                children![(label("", fonts.bold(17.0), colors::ACCENT), text)],
            ));
            panel.spawn(small_button("+", fonts)).observe(
                move |_: On<PointerClick>, mut round: Round, mut progress: ResMut<Progress>, mut settings: ResMut<Settings>| {
                    up(&mut round, &mut progress, &mut settings)
                },
            );
        };
        stepper(
            panel,
            "Tempo",
            PanelText::Bpm,
            |r, p, s| {
                let (bpm, target) = (r.drill.bpm, r.drill.target);
                r.retempo(p, s, bpm.saturating_sub(BPM_STEP), target)
            },
            |r, p, s| {
                let (bpm, target) = (r.drill.bpm, r.drill.target);
                // Faster than the target moves the target too.
                r.retempo(p, s, bpm + BPM_STEP, target.max(bpm + BPM_STEP))
            },
        );
        stepper(
            panel,
            "Target",
            PanelText::Target,
            |r, p, s| {
                let (bpm, target) = (r.drill.bpm, r.drill.target);
                r.retempo(p, s, bpm, target.saturating_sub(BPM_STEP))
            },
            |r, p, s| {
                let (bpm, target) = (r.drill.bpm, r.drill.target);
                r.retempo(p, s, bpm, target + BPM_STEP)
            },
        );
        stepper(
            panel,
            "Notes per beat",
            PanelText::NotesPerBeat,
            |r, _, s| {
                s.notes_per_beat = s.notes_per_beat.saturating_sub(1).max(1);
                s.save();
                r.restart();
            },
            |r, _, s| {
                s.notes_per_beat = (s.notes_per_beat + 1).min(4);
                s.save();
                r.restart();
            },
        );
        panel
            .spawn(Node { flex_direction: FlexDirection::Row, column_gap: px(6), margin: UiRect::left(px(6)), ..default() })
            .with_children(|dots| {
                for beat in 0..BEATS_PER_BAR {
                    let size = if beat == 0 { 18.0 } else { 14.0 };
                    dots.spawn((
                        Node {
                            width: px(size),
                            height: px(size),
                            border_radius: BorderRadius::MAX,
                            align_self: AlignSelf::Center,
                            ..default()
                        },
                        BackgroundColor(colors::BUTTON),
                        BeatDot(beat),
                        Pickable::IGNORE,
                    ));
                }
            });
        panel.spawn((label("", fonts.text(15.0), colors::DIM), PanelText::Plan));
    });
}

#[allow(clippy::too_many_arguments)]
fn metronome_panel(
    drill: Res<Drill>,
    settings: Res<Settings>,
    metronome: Res<Metronome>,
    real: Res<Time<Real>>,
    mut texts: Query<(&PanelText, &mut Text)>,
    mut toggles: Query<(&mut Btn, &mut BackgroundColor), (With<ToggleButton>, Without<BeatDot>)>,
    mut dots: Query<(&BeatDot, &mut BackgroundColor), Without<ToggleButton>>,
    progress: Res<Progress>,
) {
    let lit = metronome.on.then(|| metronome.beat_at(real.elapsed_secs_f64()) % BEATS_PER_BAR);
    // A beat lights up for the first third of its length.
    let fresh = metronome.on && {
        let beat = metronome.beat_seconds() as f64;
        let into = real.elapsed_secs_f64() - metronome.beat_at(real.elapsed_secs_f64()) as f64 * beat;
        into.rem_euclid(beat) < beat / 3.0
    };
    for (dot, mut bg) in &mut dots {
        let color = match lit {
            Some(beat) if beat == dot.0 && fresh => {
                if dot.0 == 0 {
                    colors::ACCENT
                } else {
                    colors::OK
                }
            }
            Some(_) => colors::BUTTON_HOVER,
            None => colors::LOCKED,
        };
        if bg.0 != color {
            bg.0 = color;
        }
    }
    if !(drill.is_changed() || settings.is_changed() || metronome.is_changed() || progress.is_changed()) {
        return;
    }
    for (mut btn, mut bg) in &mut toggles {
        btn.color = if settings.metronome { colors::SELECTED } else { colors::BUTTON };
        btn.hover = btn.color.mix(&Color::WHITE, 0.12);
        bg.0 = btn.color;
    }
    let record = progress.get(&drill.instrument.key()).and_then(|p| p.drills.get(&drill.level.id)).cloned().unwrap_or_default();
    let plan = if !settings.metronome {
        "Turn the metronome on to practice at a tempo".to_string()
    } else if record.target_reached && drill.bpm >= drill.target {
        format!("Lesson complete at {} BPM: raise the target to go on", drill.target)
    } else if record.misses > 0 {
        format!("Once more at {} BPM, a second miss slows down by {BPM_STEP}", drill.bpm)
    } else {
        format!("+{BPM_STEP} BPM after each clean round, up to {}", drill.target)
    };
    for (text, mut shown) in &mut texts {
        let value = match text {
            PanelText::Toggle => if settings.metronome { "Metronome on" } else { "Metronome off" }.to_string(),
            PanelText::Bpm => format!("{} BPM", drill.bpm),
            PanelText::Target => format!("{} BPM", drill.target),
            PanelText::NotesPerBeat => settings.notes_per_beat.to_string(),
            PanelText::Plan => plan.clone(),
        };
        if shown.0 != value {
            shown.0 = value;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn play(
    mut drill: ResMut<Drill>,
    mut heard: MessageReader<NoteHeard>,
    mut tapped: MessageReader<FretTapped>,
    time: Res<Time>,
    real: Res<Time<Real>>,
    settings: Res<Settings>,
    mut metronome: ResMut<Metronome>,
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
        if metronome.on {
            drill.hits.push(real.elapsed_secs());
        }
        drill.index += 1;
        drill.waiting_since = now;
        drill.hinted = false;
        drill.feedback.clear();
        if drill.index == drill.steps.len() {
            drill.ended = Some(now);
            finish(&mut drill, now, &settings, &mut metronome, &mut progress, &mut commands, &fonts, &roots);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn finish(
    drill: &mut Drill,
    now: f32,
    settings: &Settings,
    metronome: &mut Metronome,
    progress: &mut Progress,
    commands: &mut Commands,
    fonts: &Fonts,
    roots: &Query<Entity, With<DrillRoot>>,
) {
    let accuracy = drill.accuracy();
    let speed = drill.notes_per_minute(now);
    let in_time = drill.in_time(settings.notes_per_beat).filter(|_| metronome.on);
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
    let played_bpm = drill.bpm;
    let change = in_time.map(|in_time| after_round(record, accuracy >= GOAL && in_time >= TIMING_GOAL));
    (drill.bpm, drill.target) = tempo_of(record);
    progress.save();
    // The next round at the new tempo, the clicks too.
    metronome.bpm = drill.bpm;
    let instrument_progress = progress.of(&key).clone();
    let all = levels();
    let next = all.iter().skip(drill.level.id + 1).find(|l| is_unlocked(l, |id| instrument_progress.drill_done(id))).map(|l| l.id);
    let verdict = if accuracy >= GOAL {
        if first_time_done { "Level done! The next one is open." } else { "Goal reached." }
    } else {
        "Under 90%: once more, slower and cleaner."
    };
    let mut lines = vec![verdict.to_string()];
    lines.push(format!(
        "Accuracy {} ({} right, {} wrong) · {:.0} notes per minute",
        percent_text(accuracy),
        drill.correct,
        drill.wrong,
        speed
    ));
    if let Some(in_time) = in_time {
        lines.push(format!("In time {} at {played_bpm} BPM (the goal: {})", percent_text(in_time), percent_text(TIMING_GOAL)));
    }
    let tempo_line = match change {
        Some(TempoChange::Faster(bpm)) => Some(format!("Clean and in time: the next round at {bpm} BPM.")),
        Some(TempoChange::TargetReached(bpm)) => {
            Some(format!("Lesson complete: clean at the target, {bpm} BPM! Raise the target to go faster."))
        }
        Some(TempoChange::Again(bpm)) => Some(format!("Not clean yet: once more at {bpm} BPM.")),
        Some(TempoChange::Slower(bpm)) => Some(format!("Twice under the goal: slowing down to {bpm} BPM.")),
        None => None,
    };
    let clean = matches!(change, Some(TempoChange::Faster(_) | TempoChange::TargetReached(_)));
    let complete = matches!(change, Some(TempoChange::TargetReached(_)));
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
                    panel.spawn(label(lines.join("\n"), fonts.bold(20.0), color));
                    if let Some(tempo_line) = tempo_line {
                        let color = if complete {
                            colors::ACCENT
                        } else if clean {
                            colors::OK
                        } else {
                            colors::TEXT
                        };
                        panel.spawn(label(tempo_line, fonts.bold(20.0), color));
                    }
                    panel
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: px(10),
                            row_gap: px(8),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn(button("Again", fonts, colors::BUTTON))
                                .observe(|_: On<PointerClick>, mut round: Round| round.restart());
                            if complete {
                                let raised = drill.target + BPM_STEP;
                                row.spawn(button(format!("Target {raised} BPM, again"), fonts, colors::SELECTED)).observe(
                                    move |_: On<PointerClick>,
                                          mut round: Round,
                                          mut progress: ResMut<Progress>,
                                          settings: Res<Settings>| {
                                        let bpm = round.drill.bpm;
                                        round.retempo(&mut progress, &settings, bpm, raised);
                                    },
                                );
                            }
                            if let Some(next) = next {
                                row.spawn(button("Next drill", fonts, colors::SELECTED)).observe(
                                    move |_: On<PointerClick>,
                                          mut selected: ResMut<SelectedLevel>,
                                          mut state: ResMut<NextState<Screen>>| {
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
    metronome: Res<Metronome>,
    time: Res<Time>,
    layers: Query<(Entity, &MarksLayer)>,
    tabs: Query<Entity, With<TabView>>,
    staffs: Query<(Entity, &StaffView)>,
    mut texts: ParamSet<(Query<&mut Text, With<StatusText>>, Query<&mut Text, With<FeedbackText>>, Query<&mut Text, With<NoteCaption>>)>,
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
                    Spelled::from_midi(drill.instrument.midi_at(step.position), false).name()
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
    // The current note on the staff, as written for the instrument.
    let sounding = current.map(|step| Spelled::from_midi(drill.instrument.midi_at(step.position), false));
    if let Ok(staff) = staffs.single() {
        let color = current.map(|step| colors::FINGERS[step.finger as usize].mix(&colors::INK, 0.35));
        draw_staff(
            &mut commands,
            staff,
            &StaffDrawing { note: sounding.map(Spelled::written), color, letters: Default::default() },
            &fonts,
        );
    }
    let caption = match (current, sounding) {
        (Some(step), Some(note)) => format!(
            "{}: string {} fret {}, written {}",
            note.full_name(),
            drill.instrument.strings() - step.position.string,
            step.position.fret,
            note.written().full_name()
        ),
        _ => String::new(),
    };
    for mut text in &mut texts.p2() {
        text.0 = caption.clone();
    }
    let total = drill.steps.len();
    let mut status_text = format!(
        "Note {}/{} · accuracy {} · {:.0} notes/min",
        (drill.index + 1).min(total),
        total,
        if drill.correct + drill.wrong == 0 { "-".to_string() } else { percent_text(drill.accuracy()) },
        drill.notes_per_minute(now)
    );
    if metronome.on {
        let in_time = drill.in_time(settings.notes_per_beat).map_or("-".to_string(), percent_text);
        status_text.push_str(&format!(" · in time {in_time} at {} BPM", drill.bpm));
    }
    for mut text in &mut texts.p0() {
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
        Some(_) if drill.feedback.is_empty() && drill.index == 0 => {
            if metronome.on {
                let per_beat = match settings.notes_per_beat {
                    1 => "one note per click".to_string(),
                    n => format!("{n} notes per click"),
                };
                format!("Play the notes with the metronome, {per_beat}, or tap them on the fretboard")
            } else {
                "Play the notes, or tap them on the fretboard".to_string()
            }
        }
        _ => drill.feedback.clone(),
    };
    for mut text in &mut texts.p1() {
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
