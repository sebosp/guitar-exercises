//! The drills page: every exercise in its row, with its stars and the tier of its tempo, locked
//! until it opens. The scale of the scale exercises is chosen here; a button practices the
//! weakest exercise, another opens a locked one at random.
use bevy::picking::events::PointerClick;
use bevy::prelude::*;

use crate::diagram::{spawn_finger_legend, spawn_shape_diagram};
use crate::drills::{SHAPES, gap_shape};
use crate::exercises::{Catalog, EXTRA_STARS, Exercise, Kind, Row, Section, group_example};
use crate::progress::{InstrumentProgress, Progress};
use crate::scales::{ROOT_NAMES, ScaleFamily};
use crate::settings::Settings;
use crate::tempo::{Tier, tempo_of};
use crate::ui::{
    Fonts, Icons, Screen, button, button_frame, colors, label, screen_root, scroll_area, small_button, spawn_stars, spawn_top_bar,
};

/// The exercise to play, by its key.
#[derive(Resource, Default, Clone)]
pub struct SelectedLevel(pub String);

/// A line shown on top of the page once, after a random unlock.
#[derive(Resource, Default)]
struct Notice(String);

pub struct DrillsPlugin;

impl Plugin for DrillsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedLevel>().init_resource::<Notice>().add_systems(OnEnter(Screen::Drills), spawn);
    }
}

fn spawn(
    mut commands: Commands,
    fonts: Res<Fonts>,
    icons: Res<Icons>,
    settings: Res<Settings>,
    progress: Res<Progress>,
    mut notice: ResMut<Notice>,
) {
    let fonts = &*fonts;
    let icons = &*icons;
    let catalog = settings.catalog();
    let record = progress.get(&settings.instrument().key()).cloned().unwrap_or_default();
    let notice = std::mem::take(&mut notice.0);
    commands.spawn(screen_root(Screen::Drills)).with_children(|root| {
        spawn_top_bar(root, "Drills", Screen::Drills, fonts);
        root.spawn(scroll_area()).observe(crate::ui::scroll_on_wheel).observe(crate::ui::scroll_on_drag).with_children(|list| {
            spawn_header(list, &catalog, &record, &notice, fonts, icons);
            for section in Section::ALL {
                let rows: Vec<&Row> = catalog.rows.iter().filter(|r| r.section == section).collect();
                if section == Section::Extra && !catalog.extra_revealed(&record) {
                    spawn_extra_teaser(list, &record, fonts);
                    continue;
                }
                list.spawn(Node { height: px(6), ..default() });
                list.spawn(label(section.name(), fonts.bold(24.0), colors::text()));
                match section {
                    Section::Fingers => {
                        list.spawn(label(
                            "The four fingers in every order: up the strings, one fret higher, back down. The orders that loop into each other (1-2-3-4, 2-3-4-1, 3-4-1-2, 4-1-2-3) are one drill.",
                            fonts.text(15.0),
                            colors::dim(),
                        ));
                        spawn_finger_legend(list, fonts);
                    }
                    Section::Scales => spawn_scale_picker(list, &settings, fonts),
                    Section::Stretches => {
                        list.spawn(label(
                            "Growing gaps: the four fingers 2 frets apart, then wider in the extra exercises. Growing shifts: one finger per fret moving up the neck by jumps that grow, back down on the next string.",
                            fonts.text(15.0),
                            colors::dim(),
                        ));
                    }
                    Section::Extra => {
                        list.spawn(label("Skipping 3 strings and more, wider gaps, bigger shifts.", fonts.text(15.0), colors::dim()));
                    }
                }
                if section == Section::Scales && rows.is_empty() {
                    list.spawn(label("This scale does not fit on the frets of the instrument.", fonts.text(15.0), colors::bad()));
                }
                for row in rows {
                    spawn_row(list, row, &catalog, &record, settings.frets, fonts, icons);
                }
            }
        });
    });
}

/// The stars, the two buttons, the stars and tiers explained.
fn spawn_header(
    list: &mut ChildSpawnerCommands,
    catalog: &Catalog,
    record: &InstrumentProgress,
    notice: &str,
    fonts: &Fonts,
    icons: &Icons,
) {
    list.spawn(Node {
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        align_items: AlignItems::Center,
        column_gap: px(12),
        row_gap: px(8),
        ..default()
    })
    .with_children(|row| {
        row.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: px(6), ..default() })
            .with_children(|stars| {
                stars.spawn((
                    Node { width: px(26), height: px(26), ..default() },
                    ImageNode::new(icons.star.clone()).with_color(crate::ui::STAR),
                ));
                stars.spawn(label(record.total_stars().to_string(), fonts.bold(22.0), colors::text()));
            });
        row.spawn(button("Practice the weakest", fonts, colors::selected())).observe(
            |_: On<PointerClick>,
             settings: Res<Settings>,
             progress: Res<Progress>,
             mut selected: ResMut<SelectedLevel>,
             mut next: ResMut<NextState<Screen>>| {
                let record = progress.get(&settings.instrument().key()).cloned().unwrap_or_default();
                if let Some(weakest) = settings.catalog().weakest(&record) {
                    selected.0 = weakest.key.clone();
                    next.set(Screen::Drill);
                }
            },
        );
        let locked = catalog.locked(record).len();
        row.spawn(button(format!("Unlock a random level ({locked} locked)"), fonts, colors::button())).observe(
            |_: On<PointerClick>,
             settings: Res<Settings>,
             mut progress: ResMut<Progress>,
             mut notice: ResMut<Notice>,
             mut next: ResMut<NextState<Screen>>| {
                let catalog = settings.catalog();
                let key = settings.instrument().key();
                let record = progress.of(&key);
                let locked = catalog.locked(record);
                notice.0 = if locked.is_empty() {
                    "Every level is open.".to_string()
                } else {
                    let pick = locked[fastrand::usize(..locked.len())];
                    record.unlocked.insert(pick.key.clone());
                    format!("Opened at random: {}", pick.name)
                };
                progress.save();
                // The same state: the page is built again.
                next.set(Screen::Drills);
            },
        );
    });
    if !notice.is_empty() {
        list.spawn(label(notice, fonts.bold(17.0), colors::accent()));
    }
    list.spawn(label(
        "A star for 80% of the notes right, two for 90%, three for all of them; a star opens the next level. The border is the fastest clean round with the metronome, in notes per minute (the tempo times the notes per beat):",
        fonts.text(15.0),
        colors::dim(),
    ));
    list.spawn(Node { flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, column_gap: px(8), row_gap: px(6), ..default() })
        .with_children(|row| {
            for tier in Tier::ALL {
                let text = match tier {
                    Tier::Basic => "Basic: a clean round".to_string(),
                    t => format!("{}: {}+", t.name(), t.threshold()),
                };
                row.spawn((
                    Node {
                        padding: UiRect::axes(px(10), px(4)),
                        border: UiRect::all(px(3)),
                        border_radius: BorderRadius::all(px(8)),
                        ..default()
                    },
                    BorderColor::all(tier.color()),
                    BackgroundColor(colors::button()),
                    children![label(text, fonts.text(14.0), colors::text())],
                ));
            }
        });
}

fn spawn_extra_teaser(list: &mut ChildSpawnerCommands, record: &InstrumentProgress, fonts: &Fonts) {
    list.spawn(Node { height: px(6), ..default() });
    list.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(4),
            padding: UiRect::all(px(14)),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        BackgroundColor(colors::locked()),
    ))
    .with_children(|card| {
        card.spawn(label("??? Extra exercises", fonts.bold(22.0), colors::dim()));
        card.spawn(label(
            format!("Earn {EXTRA_STARS} stars to reveal them: {} so far.", record.total_stars().min(EXTRA_STARS)),
            fonts.text(15.0),
            colors::dim(),
        ));
    });
}

/// Root - +, the family, mode - +, and the notes of the scale.
fn spawn_scale_picker(list: &mut ChildSpawnerCommands, settings: &Settings, fonts: &Fonts) {
    let scale = settings.scale;
    list.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: px(8),
            padding: UiRect::all(px(12)),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        BackgroundColor(colors::panel()),
    ))
    .with_children(|panel| {
        panel
            .spawn(Node { flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, align_items: AlignItems::Center, column_gap: px(8), row_gap: px(6), ..default() })
            .with_children(|row| {
                row.spawn(label("Root", fonts.text(16.0), colors::dim()));
                row.spawn(small_button("-", fonts)).observe(|_: On<PointerClick>, s: ResMut<Settings>, n: ResMut<NextState<Screen>>| {
                    change_scale(s, n, |sc| sc.root = (sc.root + 11) % 12)
                });
                row.spawn((Node { min_width: px(44), justify_content: JustifyContent::Center, ..default() }, children![label(
                    ROOT_NAMES[scale.root as usize % 12],
                    fonts.bold(20.0),
                    colors::accent()
                )]));
                row.spawn(small_button("+", fonts)).observe(|_: On<PointerClick>, s: ResMut<Settings>, n: ResMut<NextState<Screen>>| {
                    change_scale(s, n, |sc| sc.root = (sc.root + 1) % 12)
                });
                row.spawn(Node { width: px(12), ..default() });
                for family in ScaleFamily::ALL {
                    let color = if family == scale.family { colors::selected() } else { colors::button() };
                    row.spawn(button(family.name(), fonts, color)).observe(move |_: On<PointerClick>, s: ResMut<Settings>, n: ResMut<NextState<Screen>>| {
                        change_scale(s, n, move |sc| {
                            if sc.family != family {
                                sc.family = family;
                                sc.mode = 0;
                            }
                        })
                    });
                }
            });
        panel
            .spawn(Node { flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, align_items: AlignItems::Center, column_gap: px(8), row_gap: px(6), ..default() })
            .with_children(|row| {
                row.spawn(label("Mode", fonts.text(16.0), colors::dim()));
                row.spawn(small_button("-", fonts)).observe(|_: On<PointerClick>, s: ResMut<Settings>, n: ResMut<NextState<Screen>>| {
                    change_scale(s, n, |sc| {
                        let modes = sc.family.modes().len() as u8;
                        sc.mode = (sc.mode + modes - 1) % modes;
                    })
                });
                row.spawn((Node { min_width: px(230), justify_content: JustifyContent::Center, ..default() }, children![label(
                    format!("{}. {}", scale.mode + 1, scale.mode_name()),
                    fonts.bold(18.0),
                    colors::accent()
                )]));
                row.spawn(small_button("+", fonts)).observe(|_: On<PointerClick>, s: ResMut<Settings>, n: ResMut<NextState<Screen>>| {
                    change_scale(s, n, |sc| sc.mode = (sc.mode + 1) % sc.family.modes().len() as u8)
                });
                row.spawn(Node { width: px(12), ..default() });
                row.spawn(label(scale.note_names().join("  "), fonts.bold(18.0), colors::text()));
            });
        let per_string = scale.family.notes_per_string();
        panel.spawn(label(
            format!(
                "{per_string} notes per string from the root on the lowest string: as a run, then in groups of 2 to 6 notes, up and back down; then on every other string. Each scale and mode keeps its own stars."
            ),
            fonts.text(14.0),
            colors::dim(),
        ));
    });
}

fn change_scale(mut settings: ResMut<Settings>, mut next: ResMut<NextState<Screen>>, change: impl FnOnce(&mut crate::scales::Scale)) {
    change(&mut settings.scale);
    settings.sanitize();
    settings.save();
    next.set(Screen::Drills);
}

fn spawn_row(
    list: &mut ChildSpawnerCommands,
    row: &Row,
    catalog: &Catalog,
    record: &InstrumentProgress,
    frets: u8,
    fonts: &Fonts,
    icons: &Icons,
) {
    list.spawn(label(&row.title, fonts.bold(19.0), colors::accent()));
    // The shape of a finger drill or a gap, drawn.
    match row.exercises.first().map(|e| e.kind) {
        Some(Kind::Finger { shape, .. }) => spawn_shape_diagram(list, &SHAPES[shape], fonts),
        Some(Kind::Gaps { gap }) => {
            if let Some(shape) = gap_shape(gap, frets) {
                spawn_shape_diagram(list, &shape, fonts)
            }
        }
        _ => {}
    }
    list.spawn(Node { flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, column_gap: px(8), row_gap: px(8), ..default() })
        .with_children(|grid| {
            for exercise in &row.exercises {
                spawn_tile(grid, exercise, catalog, record, fonts, icons);
            }
        });
}

fn spawn_tile(
    grid: &mut ChildSpawnerCommands,
    exercise: &Exercise,
    catalog: &Catalog,
    record: &InstrumentProgress,
    fonts: &Fonts,
    icons: &Icons,
) {
    let unlocked = catalog.is_unlocked(&exercise.key, record);
    let drill = record.exercises.get(&exercise.key);
    let tier = drill.and_then(Tier::of);
    let color = if unlocked { colors::button() } else { colors::locked() };
    let border = tier.map(Tier::color).unwrap_or(color);
    let key = exercise.key.clone();
    grid.spawn(button_frame(color))
        .insert((
            Node {
                width: px(122),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(3),
                padding: UiRect::axes(px(6), px(6)),
                border: UiRect::all(px(3)),
                border_radius: BorderRadius::all(px(10)),
                ..default()
            },
            BorderColor::all(border),
        ))
        .with_children(|tile| {
            let text_color = if unlocked { colors::text() } else { colors::dim() };
            tile.spawn(label(&exercise.label, fonts.bold(17.0), text_color));
            if let Kind::Scale { group, .. } = exercise.kind {
                tile.spawn(label(group_example(group), fonts.text(13.0), colors::dim()));
            }
            if !unlocked {
                tile.spawn(label("locked", fonts.text(13.0), colors::dim()));
                return;
            }
            spawn_stars(tile, drill.map_or(0, |d| d.stars()), 18.0, icons);
            let detail = match drill {
                Some(d) if d.bpm > 0 => {
                    let (bpm, target) = tempo_of(d);
                    if d.target_reached { format!("{target} BPM done") } else { format!("{bpm}/{target} BPM") }
                }
                Some(d) if d.plays > 0 => format!("best {:.0}%", d.best_accuracy * 100.0),
                _ => "new".to_string(),
            };
            let detail_color = match tier {
                Some(t) => t.color(),
                None => colors::dim(),
            };
            tile.spawn(label(detail, fonts.text(12.0), detail_color));
        })
        .observe(move |_: On<PointerClick>, mut selected: ResMut<SelectedLevel>, mut next: ResMut<NextState<Screen>>| {
            if unlocked {
                selected.0 = key.clone();
                next.set(Screen::Drill);
            }
        });
}
