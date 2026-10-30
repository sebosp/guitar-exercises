//! The statistics of the instrument set up: the drills played, and every note read and found with
//! its accuracy, its time to answer and the hints.
use bevy::prelude::*;

use crate::drills::levels;
use crate::music::learning_order;
use crate::progress::{NoteStats, Progress};
use crate::settings::Settings;
use crate::ui::{Fonts, Screen, colors, label, percent_text, screen_root, scroll_area, spawn_top_bar};

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Stats), spawn);
    }
}

fn table_row(parent: &mut ChildSpawnerCommands, cells: &[(String, f32, Color)], fonts: &Fonts, bold: bool) {
    parent.spawn(Node { flex_direction: FlexDirection::Row, column_gap: px(6), ..default() }).with_children(|row| {
        for (text, width, color) in cells {
            let font = if bold { fonts.bold(15.0) } else { fonts.text(15.0) };
            row.spawn((Node { width: px(*width), ..default() }, children![label(text.clone(), font, *color)]));
        }
    });
}

fn accuracy_color(accuracy: Option<f32>) -> Color {
    match accuracy {
        Some(a) if a >= 0.9 => colors::OK,
        Some(a) if a < 0.7 => colors::BAD,
        Some(_) => colors::ACCENT,
        None => colors::DIM,
    }
}

fn note_rows(parent: &mut ChildSpawnerCommands, names: &[String], stats: &std::collections::BTreeMap<String, NoteStats>, fonts: &Fonts) {
    table_row(
        parent,
        &[
            ("Note".into(), 70.0, colors::DIM),
            ("Answers".into(), 80.0, colors::DIM),
            ("Accuracy".into(), 90.0, colors::DIM),
            ("Time".into(), 80.0, colors::DIM),
            ("Hints".into(), 60.0, colors::DIM),
        ],
        fonts,
        true,
    );
    for name in names {
        let Some(s) = stats.get(name) else { continue };
        let accuracy = s.accuracy();
        table_row(
            parent,
            &[
                (name.clone(), 70.0, colors::TEXT),
                (s.answers().to_string(), 80.0, colors::TEXT),
                (accuracy.map(percent_text).unwrap_or("-".into()), 90.0, accuracy_color(accuracy)),
                (s.average_time().map(|t| format!("{t:.1} s")).unwrap_or("-".into()), 80.0, colors::TEXT),
                (s.hints.to_string(), 60.0, colors::TEXT),
            ],
            fonts,
            false,
        );
    }
}

fn spawn(mut commands: Commands, fonts: Res<Fonts>, settings: Res<Settings>, progress: Res<Progress>) {
    let fonts = &*fonts;
    let instrument = settings.instrument();
    let key = instrument.key();
    let record = progress.get(&key).cloned().unwrap_or_default();
    let all = levels();
    let order = learning_order(&instrument, true);
    let names: Vec<String> = order.iter().map(|n| n.full_name()).collect();
    commands.spawn(screen_root(Screen::Stats)).with_children(|root| {
        spawn_top_bar(root, &format!("Statistics: {} with {} strings", settings.instrument.name(), settings.strings), Screen::Stats, fonts);
        root.spawn(scroll_area()).observe(crate::ui::scroll_on_wheel).observe(crate::ui::scroll_on_drag).with_children(|list| {
            let done = record.drills.values().filter(|d| d.done).count();
            list.spawn(label(format!("Finger drills: {done} of {} done", all.len()), fonts.bold(20.0), colors::ACCENT));
            if record.drills.is_empty() {
                list.spawn(label("No drill played yet.", fonts.text(15.0), colors::DIM));
            } else {
                table_row(
                    list,
                    &[
                        ("Drill".into(), 330.0, colors::DIM),
                        ("Plays".into(), 60.0, colors::DIM),
                        ("Best".into(), 70.0, colors::DIM),
                        ("Last".into(), 70.0, colors::DIM),
                        ("Notes/min".into(), 90.0, colors::DIM),
                    ],
                    fonts,
                    true,
                );
                for (id, d) in &record.drills {
                    let Some(level) = all.get(*id) else { continue };
                    table_row(
                        list,
                        &[
                            (level.name(), 330.0, if d.done { colors::OK } else { colors::TEXT }),
                            (d.plays.to_string(), 60.0, colors::TEXT),
                            (percent_text(d.best_accuracy), 70.0, accuracy_color(Some(d.best_accuracy))),
                            (percent_text(d.last_accuracy), 70.0, accuracy_color(Some(d.last_accuracy))),
                            (format!("{:.0}", d.best_notes_per_minute), 90.0, colors::TEXT),
                        ],
                        fonts,
                        false,
                    );
                }
            }
            let natural_total = learning_order(&instrument, false).len();
            list.spawn(label(
                format!(
                    "Learn notes: {} of {} learned",
                    record.learned.min(order.len()),
                    if settings.accidentals { order.len() } else { natural_total }
                ),
                fonts.bold(20.0),
                colors::ACCENT,
            ));
            if record.reading.is_empty() {
                list.spawn(label("No note read yet.", fonts.text(15.0), colors::DIM));
            } else {
                note_rows(list, &names, &record.reading, fonts);
            }
            list.spawn(label("Find them all", fonts.bold(20.0), colors::ACCENT));
            if record.finding.is_empty() {
                list.spawn(label("No note found yet.", fonts.text(15.0), colors::DIM));
            } else {
                note_rows(list, &names, &record.finding, fonts);
            }
        });
    });
}
