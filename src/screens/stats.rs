//! The statistics, as in claudralingo: the streak of days with the daily goal met and today, the
//! notes a minute and the accuracy of the last 30 days, the calendars of this month and the
//! previous one (a day is a success with a star earned), the rounds by type; then for the
//! instrument set up the drills played, and every note read and found with its accuracy, its
//! time to answer and the hints.
//!
//! The charts follow the dataviz method: one measure per chart and one axis, one hue, the values
//! in the text colors, the goal as a dashed line, a readout of the day under the pointer.
use bevy::prelude::*;

use crate::music::learning_order;
use crate::practice;
use crate::progress::{NoteStats, Progress};
use crate::settings::Settings;
use crate::ui::{Fonts, Screen, colors, label, percent_text, screen_root, scroll_area, size, spawn_top_bar};

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Stats), spawn).add_systems(Update, readout.run_if(in_state(Screen::Stats)));
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
        Some(a) if a >= 0.9 => colors::ok(),
        Some(a) if a < 0.7 => colors::bad(),
        Some(_) => colors::accent(),
        None => colors::dim(),
    }
}

fn note_rows(parent: &mut ChildSpawnerCommands, names: &[String], stats: &std::collections::BTreeMap<String, NoteStats>, fonts: &Fonts) {
    table_row(
        parent,
        &[
            ("Note".into(), 70.0, colors::dim()),
            ("Answers".into(), 80.0, colors::dim()),
            ("Accuracy".into(), 90.0, colors::dim()),
            ("Time".into(), 80.0, colors::dim()),
            ("Hints".into(), 60.0, colors::dim()),
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
                (name.clone(), 70.0, colors::text()),
                (s.answers().to_string(), 80.0, colors::text()),
                (accuracy.map(percent_text).unwrap_or("-".into()), 90.0, accuracy_color(accuracy)),
                (s.average_time().map(|t| format!("{t:.1} s")).unwrap_or("-".into()), 80.0, colors::text()),
                (s.hints.to_string(), 60.0, colors::text()),
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
    let catalog = settings.catalog();
    let order = learning_order(&instrument, true);
    let names: Vec<String> = order.iter().map(|n| n.full_name()).collect();
    commands.spawn(screen_root(Screen::Stats)).with_children(|root| {
        spawn_top_bar(root, &format!("Statistics: {} with {} strings", settings.instrument.name(), settings.strings), Screen::Stats, fonts);
        root.spawn(scroll_area()).observe(crate::ui::scroll_on_wheel).observe(crate::ui::scroll_on_drag).with_children(|list| {
            spawn_overview(list, &progress, &settings, fonts);
            let done = record.exercises.values().filter(|d| d.stars() > 0).count();
            list.spawn(label(
                format!("Drills: {done} with a star, {} stars in all", record.total_stars()),
                fonts.bold(20.0),
                colors::accent(),
            ));
            if record.exercises.is_empty() {
                list.spawn(label("No drill played yet.", fonts.text(15.0), colors::dim()));
            } else {
                table_row(
                    list,
                    &[
                        ("Drill".into(), 420.0, colors::dim()),
                        ("Plays".into(), 60.0, colors::dim()),
                        ("Stars".into(), 60.0, colors::dim()),
                        ("Best".into(), 70.0, colors::dim()),
                        ("Last".into(), 70.0, colors::dim()),
                        ("Notes/min".into(), 90.0, colors::dim()),
                        ("Tier".into(), 100.0, colors::dim()),
                    ],
                    fonts,
                    true,
                );
                // The exercises of the page first, in its order, then the other scales.
                let mut keys: Vec<(String, String)> = catalog
                    .exercises()
                    .filter(|e| record.exercises.contains_key(&e.key))
                    .map(|e| (e.key.clone(), e.name.clone()))
                    .collect();
                for key in record.exercises.keys() {
                    if catalog.find(key).is_none() {
                        keys.push((key.clone(), key.replace('/', ", ")));
                    }
                }
                for (key, name) in keys {
                    let d = &record.exercises[&key];
                    let tier = crate::tempo::Tier::of(d);
                    table_row(
                        list,
                        &[
                            (name, 420.0, if d.stars() > 0 { colors::ok() } else { colors::text() }),
                            (d.plays.to_string(), 60.0, colors::text()),
                            (d.stars().to_string(), 60.0, colors::text()),
                            (percent_text(d.best_accuracy), 70.0, accuracy_color(Some(d.best_accuracy))),
                            (percent_text(d.last_accuracy), 70.0, accuracy_color(Some(d.last_accuracy))),
                            (format!("{:.0}", d.best_notes_per_minute), 90.0, colors::text()),
                            (
                                tier.map_or("-".into(), |t| t.name().to_string()),
                                100.0,
                                if tier.is_some() { colors::text() } else { colors::dim() },
                            ),
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
                colors::accent(),
            ));
            if record.reading.is_empty() {
                list.spawn(label("No note read yet.", fonts.text(15.0), colors::dim()));
            } else {
                note_rows(list, &names, &record.reading, fonts);
            }
            list.spawn(label("Find them all", fonts.bold(20.0), colors::accent()));
            if record.finding.is_empty() {
                list.spawn(label("No note found yet.", fonts.text(15.0), colors::dim()));
            } else {
                note_rows(list, &names, &record.finding, fonts);
            }
        });
    });
}

/// The days of the charts.
const DAYS: u64 = 30;
const CHART_HEIGHT: f32 = 120.0;
const COLUMN: f32 = 14.0;

/// A day of a chart, a hover target: the text of its readout.
#[derive(Component)]
struct ChartDay {
    chart: usize,
    text: String,
}

/// The readout of a chart: what the day under the pointer holds.
#[derive(Component)]
struct Readout(usize);

const READOUT_HINT: &str = "Point at a day for its value";

/// The streak and today, the charts, the calendars and the rounds by type.
fn spawn_overview(list: &mut ChildSpawnerCommands, progress: &Progress, settings: &Settings, fonts: &Fonts) {
    let today = practice::today();
    let goal = settings.daily_goal_minutes;
    let streak = practice::streak(&progress.days, today, goal);
    let minutes = progress.days.get(&today).map_or(0.0, |d| d.seconds / 60.0);
    // The streak, the one big thing; today against the goal beside it.
    list.spawn(Node {
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        align_items: AlignItems::End,
        column_gap: px(40),
        row_gap: px(10),
        ..default()
    })
    .with_children(|row| {
        row.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Baseline, column_gap: px(10), ..default() })
            .with_children(|s| {
                s.spawn(label(streak.to_string(), fonts.display(size::HERO * 2.0), colors::text()));
                s.spawn(label(if streak == 1 { "day in a row" } else { "days in a row" }, fonts.display(size::HEADING), colors::dim()));
            });
        row.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(6), padding: UiRect::bottom(px(14)), ..default() })
            .with_children(|t| {
                let met = minutes >= goal as f32;
                t.spawn(label(
                    format!("{minutes:.0} of {goal} minutes today{}", if met { ", goal met" } else { "" }),
                    fonts.display(size::HEADING),
                    colors::text(),
                ));
                t.spawn((
                    Node { width: px(280), height: px(8), border_radius: BorderRadius::all(px(4)), ..default() },
                    BackgroundColor(colors::button()),
                    children![(
                        Node {
                            width: percent((minutes / goal as f32).min(1.0) * 100.0),
                            height: percent(100),
                            border_radius: BorderRadius::all(px(4)),
                            ..default()
                        },
                        BackgroundColor(colors::accent()),
                    )],
                ));
            });
    });
    let first = today + 1 - DAYS;
    let recent: Vec<&practice::Session> = progress.sessions.iter().filter(|s| practice::day_of(s.finished_at) >= first).collect();
    let average = |day: u64, value: &dyn Fn(&practice::Session) -> Option<f32>| -> Option<f32> {
        let values: Vec<f32> = recent.iter().filter(|s| practice::day_of(s.finished_at) == day).filter_map(|s| value(s)).collect();
        (!values.is_empty()).then(|| values.iter().sum::<f32>() / values.len() as f32)
    };
    let speed: Vec<Option<f32>> =
        (first..=today).map(|d| average(d, &|s| (s.notes_per_minute > 0.0).then_some(s.notes_per_minute))).collect();
    let right: Vec<Option<f32>> = (first..=today).map(|d| average(d, &|s| Some(s.accuracy * 100.0))).collect();
    list.spawn(Node {
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        column_gap: px(40),
        row_gap: px(20),
        margin: UiRect::top(px(10)),
        ..default()
    })
    .with_children(|row| {
        spawn_bars(row, 0, "Notes a minute, the average of the day", &speed, first, fonts);
        spawn_dots(row, 1, "Right, the average of the day, 90% dashed", &right, first, fonts);
    });
    spawn_calendars(list, progress, today, fonts);
    spawn_kinds(list, &recent, fonts);
}

fn chart_frame<'a>(parent: &'a mut ChildSpawnerCommands, chart: usize, title: &str, fonts: &Fonts) -> EntityCommands<'a> {
    let mut column = parent.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(6), ..default() });
    column.with_children(|c| {
        c.spawn(label(title, fonts.display(size::HEADING), colors::text()));
        c.spawn((label(READOUT_HINT, fonts.text(size::CAPTION), colors::dim()), Readout(chart)));
    });
    column
}

/// One column per day, a hover target as tall as the chart.
fn day_column<'a>(plot: &'a mut ChildSpawnerCommands, chart: usize, day: u64, text: String) -> EntityCommands<'a> {
    plot.spawn((
        Node {
            width: px(COLUMN),
            height: percent(100),
            justify_content: JustifyContent::End,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        bevy::picking::hover::Hovered::default(),
        ChartDay { chart, text: format!("{}: {text}", practice::short_date(day)) },
    ))
}

fn plot_area<'a>(parent: &'a mut ChildSpawnerCommands) -> EntityCommands<'a> {
    parent.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::End,
            column_gap: px(3),
            height: px(CHART_HEIGHT),
            border: UiRect::bottom(px(1)),
            ..default()
        },
        BorderColor::all(colors::dim().with_alpha(0.5)),
    ))
}

/// The notes a minute: bars from zero, one hue.
fn spawn_bars(parent: &mut ChildSpawnerCommands, chart: usize, title: &str, values: &[Option<f32>], first: u64, fonts: &Fonts) {
    let max = values.iter().flatten().copied().fold(0.0f32, f32::max).max(10.0);
    let top = (max / 20.0).ceil() * 20.0;
    chart_frame(parent, chart, title, fonts).with_children(|c| {
        c.spawn(label(format!("{top:.0}"), fonts.text(size::CAPTION), colors::dim()));
        plot_area(c).with_children(|plot| {
            for (i, value) in values.iter().enumerate() {
                let day = first + i as u64;
                let text = value.map_or("no round".to_string(), |v| format!("{v:.0} notes a minute"));
                day_column(plot, chart, day, text).with_children(|col| {
                    if let Some(v) = value {
                        col.spawn((
                            Node {
                                width: percent(100),
                                height: percent(v / top * 100.0),
                                border_radius: BorderRadius::top(px(3)),
                                ..default()
                            },
                            BackgroundColor(colors::accent()),
                            Pickable::IGNORE,
                        ));
                    }
                });
            }
        });
        c.spawn(label("0", fonts.text(size::CAPTION), colors::dim()));
    });
}

/// The accuracy: a dot per day on a scale from 50% (or lower), the goal of 90% dashed.
fn spawn_dots(parent: &mut ChildSpawnerCommands, chart: usize, title: &str, values: &[Option<f32>], first: u64, fonts: &Fonts) {
    let low = values.iter().flatten().copied().fold(100.0f32, f32::min).min(50.0).floor().max(0.0);
    let y = |v: f32| (v - low) / (100.0 - low) * 100.0;
    chart_frame(parent, chart, title, fonts).with_children(|c| {
        c.spawn(label("100%", fonts.text(size::CAPTION), colors::dim()));
        plot_area(c).with_children(|plot| {
            // The goal, dashed across the days.
            for dash in 0..(DAYS as usize * 17 / 8) {
                plot.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(dash as f32 * 8.0),
                        bottom: percent(y(90.0)),
                        width: px(4),
                        height: px(1),
                        ..default()
                    },
                    BackgroundColor(colors::dim()),
                    Pickable::IGNORE,
                ));
            }
            for (i, value) in values.iter().enumerate() {
                let day = first + i as u64;
                let text = value.map_or("no round".to_string(), |v| format!("{v:.0}% right"));
                day_column(plot, chart, day, text).with_children(|col| {
                    if let Some(v) = value {
                        col.spawn((
                            Node {
                                position_type: PositionType::Absolute,
                                bottom: percent(y(*v)),
                                left: px(COLUMN / 2.0 - 5.0),
                                margin: UiRect::bottom(px(-5)),
                                width: px(10),
                                height: px(10),
                                border_radius: BorderRadius::MAX,
                                ..default()
                            },
                            BackgroundColor(colors::accent()),
                            Pickable::IGNORE,
                        ));
                    }
                });
            }
        });
        c.spawn(label(format!("{low:.0}%"), fonts.text(size::CAPTION), colors::dim()));
    });
}

/// The day under the pointer, in its chart's readout.
fn readout(
    days: Query<(&ChartDay, &bevy::picking::hover::Hovered), Changed<bevy::picking::hover::Hovered>>,
    mut readouts: Query<(&Readout, &mut Text)>,
) {
    for (day, hovered) in &days {
        for (readout, mut text) in &mut readouts {
            if readout.0 == day.chart {
                text.0 = if hovered.get() { day.text.clone() } else { READOUT_HINT.to_string() };
            }
        }
    }
}

/// This month and the previous one: a day with a star earned is filled, today outlined.
fn spawn_calendars(list: &mut ChildSpawnerCommands, progress: &Progress, today: u64, fonts: &Fonts) {
    let (y, m, _) = practice::civil(today);
    let previous = if m == 1 { (y - 1, 12) } else { (y, m - 1) };
    let success = colors::ok();
    list.spawn(Node {
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        column_gap: px(32),
        row_gap: px(16),
        margin: UiRect::top(px(14)),
        ..default()
    })
    .with_children(|row| {
        for (year, month) in [previous, (y, m)] {
            row.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(6), ..default() }).with_children(|column| {
                column.spawn(label(
                    format!("{} {year}", practice::MONTHS[month as usize - 1]),
                    fonts.display(size::HEADING),
                    colors::text(),
                ));
                column
                    .spawn(Node {
                        display: Display::Grid,
                        grid_template_columns: RepeatedGridTrack::px::<Vec<RepeatedGridTrack>>(7, 30.0),
                        column_gap: px(3),
                        row_gap: px(3),
                        ..default()
                    })
                    .with_children(|grid| {
                        for name in ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"] {
                            grid.spawn(label(name, fonts.text(size::CAPTION), colors::dim()));
                        }
                        let start = practice::days_from_civil(year, month, 1);
                        // 1970-01-01 was a Thursday: Monday is 0.
                        for _ in 0..((start + 3) % 7) {
                            grid.spawn(Node { width: px(30), height: px(26), ..default() });
                        }
                        let next = if month == 12 {
                            practice::days_from_civil(year + 1, 1, 1)
                        } else {
                            practice::days_from_civil(year, month + 1, 1)
                        };
                        for day in start..next {
                            let starred = progress.days.get(&day).is_some_and(|d| d.starred > 0);
                            let fill = if starred { success } else { colors::button() };
                            let text = if starred {
                                crate::theme::text_on(success)
                            } else if day > today {
                                colors::dim()
                            } else {
                                colors::text()
                            };
                            grid.spawn((
                                Node {
                                    width: px(30),
                                    height: px(26),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(px(2)),
                                    border_radius: BorderRadius::all(px(4)),
                                    ..default()
                                },
                                BackgroundColor(fill),
                                BorderColor::all(if day == today { colors::accent() } else { Color::NONE }),
                                children![label((day - start + 1).to_string(), fonts.text(size::CAPTION), text)],
                            ));
                        }
                    });
            });
        }
        // The legend: the words say it, not the color alone.
        row.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(6), margin: UiRect::top(px(34)), ..default() }).with_children(
            |legend| {
                for (fill, outline, text) in [
                    (success, None, "a star earned"),
                    (colors::button(), None, "no star"),
                    (colors::button(), Some(colors::accent()), "today"),
                ] {
                    legend
                        .spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: px(6), ..default() })
                        .with_children(|item| {
                            item.spawn((
                                Node {
                                    width: px(14),
                                    height: px(14),
                                    border: UiRect::all(px(2)),
                                    border_radius: BorderRadius::all(px(3)),
                                    ..default()
                                },
                                BackgroundColor(fill),
                                BorderColor::all(outline.unwrap_or(Color::NONE)),
                            ));
                            item.spawn(label(text, fonts.text(size::CAPTION), colors::dim()));
                        });
                }
            },
        );
    });
}

/// The rounds of the last 30 days by type: one hue, the longest first, the counts in text.
fn spawn_kinds(list: &mut ChildSpawnerCommands, recent: &[&practice::Session], fonts: &Fonts) {
    let mut counts: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for s in recent {
        *counts.entry(if s.kind.is_empty() { "Other" } else { s.kind.as_str() }).or_default() += 1;
    }
    let mut counts: Vec<(&str, usize)> = counts.into_iter().collect();
    counts.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
    let most = counts.first().map_or(1, |(_, n)| *n).max(1);
    list.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(6), margin: UiRect::vertical(px(14)), ..default() })
        .with_children(|column| {
            column.spawn(label("Rounds of the last 30 days", fonts.display(size::HEADING), colors::text()));
            if counts.is_empty() {
                column.spawn(label("Play a drill to fill this in.", fonts.text(size::TEXT), colors::dim()));
            }
            for (name, n) in counts {
                column
                    .spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: px(8), ..default() })
                    .with_children(|row| {
                        row.spawn((Node { width: px(160), ..default() }, children![label(name, fonts.text(size::TEXT), colors::text())]));
                        row.spawn((
                            Node {
                                width: px(380.0 * n as f32 / most as f32),
                                height: px(12),
                                border_radius: BorderRadius::right(px(4)),
                                ..default()
                            },
                            BackgroundColor(colors::accent()),
                        ));
                        row.spawn(label(n.to_string(), fonts.text(size::TEXT), colors::dim()));
                    });
            }
        });
}
