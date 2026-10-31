//! The pages of exercises, the drills and the scales: every exercise in its row, with its stars
//! and the tier of its tempo, locked until it opens. The scale is chosen on its page; a button
//! practices the weakest exercise of the page, another opens a locked one at random.
use bevy::picking::events::PointerClick;
use bevy::prelude::*;

use crate::diagram::{spawn_finger_legend, spawn_shape_diagram};
use crate::drills::{SHAPES, gap_shape};
use crate::exercises::{Catalog, EXTRA_STARS, Exercise, Kind, Page, Row, Section, group_example, permutation_example};
use crate::progress::{InstrumentProgress, Progress};
use crate::scales::{ROOT_NAMES, ScaleFamily};
use crate::settings::Settings;
use crate::tempo::{Tier, tempo_of};
use crate::ui::{
    Fonts, Icons, ReturnTo, Screen, button, button_frame, colors, label, screen_root, scroll_area, small_button, spawn_stars, spawn_top_bar,
};

/// The exercise to play, by its key.
#[derive(Resource, Default, Clone)]
pub struct SelectedLevel(pub String);

/// The columns of a row: the name, the shape, the tiles (wrapping under when narrower).
const NAME_COLUMN: f32 = 150.0;
const SHAPE_COLUMN: f32 = 190.0;
const TILES_MIN: f32 = 330.0;
const TILE: f32 = 104.0;

/// A line shown on top of the page once, after a random unlock.
#[derive(Resource, Default)]
struct Notice(String);

pub struct DrillsPlugin;

impl Plugin for DrillsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedLevel>()
            .init_resource::<Notice>()
            .add_systems(OnEnter(Screen::Drills), |p: PageParams| spawn(p, Page::Drills))
            .add_systems(OnEnter(Screen::Scales), |p: PageParams| spawn(p, Page::Scales));
    }
}

/// The screen of a page.
pub fn screen_of(page: Page) -> Screen {
    match page {
        Page::Drills => Screen::Drills,
        Page::Scales => Screen::Scales,
    }
}

#[derive(bevy::ecs::system::SystemParam)]
struct PageParams<'w, 's> {
    commands: Commands<'w, 's>,
    fonts: Res<'w, Fonts>,
    icons: Res<'w, Icons>,
    settings: Res<'w, Settings>,
    progress: Res<'w, Progress>,
    notice: ResMut<'w, Notice>,
}

fn spawn(mut p: PageParams, page: Page) {
    let fonts = &*p.fonts;
    let icons = &*p.icons;
    let settings = &*p.settings;
    let catalog = settings.catalog();
    let record = p.progress.get(&settings.instrument().key()).cloned().unwrap_or_default();
    let notice = std::mem::take(&mut p.notice.0);
    let screen = screen_of(page);
    let title = match page {
        Page::Drills => "Drills",
        Page::Scales => "Scales",
    };
    p.commands.spawn(screen_root(screen)).with_children(|root| {
        spawn_top_bar(root, title, screen, fonts);
        root.spawn(scroll_area()).observe(crate::ui::scroll_on_wheel).observe(crate::ui::scroll_on_drag).with_children(|list| {
            spawn_header(list, &catalog, &record, page, &notice, fonts, icons);
            if page == Page::Drills {
                list.spawn(button("Scales: runs, sequences and orders of the notes  >", fonts, colors::button())).insert(Node {
                    align_self: AlignSelf::Start,
                    padding: UiRect::axes(px(16), px(9)),
                    border_radius: BorderRadius::all(px(8)),
                    ..default()
                })
                .observe(|_: On<PointerClick>, mut next: ResMut<NextState<Screen>>| next.set(Screen::Scales));
            }
            for section in Section::ALL.into_iter().filter(|s| s.page() == page) {
                let rows: Vec<&Row> = catalog.rows.iter().filter(|r| r.section == section).collect();
                if section.is_extra() && !catalog.extra_revealed(&record) {
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
                    Section::Scales => spawn_scale_picker(list, settings, fonts),
                    Section::Stretches => {
                        list.spawn(label(
                            "Growing gaps: the four fingers 2 frets apart, then wider in the extra exercises. Growing shifts: one finger per fret moving up the neck by jumps that grow, back down on the next string.",
                            fonts.text(15.0),
                            colors::dim(),
                        ));
                    }
                    Section::Extra => {
                        list.spawn(label("Wider gaps, bigger shifts.", fonts.text(15.0), colors::dim()));
                    }
                    Section::ScaleExtra => {
                        list.spawn(label("The scale skipping 3 strings and more: more than four strings apart.", fonts.text(15.0), colors::dim()));
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
    page: Page,
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
            move |_: On<PointerClick>,
                  settings: Res<Settings>,
                  progress: Res<Progress>,
                  mut selected: ResMut<SelectedLevel>,
                  mut back: ResMut<ReturnTo>,
                  mut next: ResMut<NextState<Screen>>| {
                let record = progress.get(&settings.instrument().key()).cloned().unwrap_or_default();
                if let Some(weakest) = settings.catalog().weakest(&record, page) {
                    selected.0 = weakest.key.clone();
                    back.0 = screen_of(page);
                    next.set(Screen::Drill);
                }
            },
        );
        let locked = catalog.locked(record, page).len();
        row.spawn(button(format!("Unlock a random level ({locked} locked)"), fonts, colors::button())).observe(
            move |_: On<PointerClick>,
                  settings: Res<Settings>,
                  mut progress: ResMut<Progress>,
                  mut notice: ResMut<Notice>,
                  mut next: ResMut<NextState<Screen>>| {
                let catalog = settings.catalog();
                let key = settings.instrument().key();
                let record = progress.of(&key);
                let locked = catalog.locked(record, page);
                notice.0 = if locked.is_empty() {
                    "Every level is open.".to_string()
                } else {
                    let pick = locked[fastrand::usize(..locked.len())];
                    record.unlocked.insert(pick.key.clone());
                    format!("Opened at random: {}", pick.name)
                };
                progress.save();
                // The same state: the page is built again.
                next.set(screen_of(page));
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
    next.set(Screen::Scales);
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
    // Three columns: the name, the shape, the levels; on a narrow screen the levels wrap under.
    list.spawn(Node {
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        align_items: AlignItems::Center,
        column_gap: px(12),
        row_gap: px(6),
        padding: UiRect::vertical(px(4)),
        border: UiRect::top(px(1)),
        ..default()
    })
    .insert(BorderColor::all(colors::button()))
    .with_children(|r| {
        r.spawn(Node { width: px(NAME_COLUMN), flex_shrink: 0.0, ..default() }).with_children(|c| {
            c.spawn(label(&row.title, fonts.bold(16.0), colors::accent()));
        });
        // The shape of a finger drill or a gap, drawn.
        let shape = match row.exercises.first().map(|e| e.kind) {
            Some(Kind::Finger { shape, .. }) => Some(SHAPES[shape]),
            Some(Kind::Gaps { gap }) => gap_shape(gap, frets),
            _ => None,
        };
        if let Some(shape) = shape {
            r.spawn(Node { min_width: px(SHAPE_COLUMN), flex_shrink: 0.0, ..default() })
                .with_children(|c| spawn_shape_diagram(c, &shape, fonts));
        }
        r.spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            flex_grow: 1.0,
            flex_basis: px(TILES_MIN),
            column_gap: px(6),
            row_gap: px(6),
            ..default()
        })
        .with_children(|grid| {
            for exercise in &row.exercises {
                spawn_tile(grid, exercise, catalog, record, row.section.page(), fonts, icons);
            }
        });
    });
}

fn spawn_tile(
    grid: &mut ChildSpawnerCommands,
    exercise: &Exercise,
    catalog: &Catalog,
    record: &InstrumentProgress,
    page: Page,
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
                width: px(TILE),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(2),
                padding: UiRect::axes(px(4), px(4)),
                border: UiRect::all(px(3)),
                border_radius: BorderRadius::all(px(10)),
                ..default()
            },
            BorderColor::all(border),
        ))
        .with_children(|tile| {
            let text_color = if unlocked { colors::text() } else { colors::dim() };
            tile.spawn(label(&exercise.label, fonts.bold(16.0), text_color));
            match exercise.kind {
                Kind::Scale { group, .. } => {
                    tile.spawn(label(group_example(group), fonts.text(12.0), colors::dim()));
                }
                Kind::Permutation { order, len } => {
                    tile.spawn(label(permutation_example(&order[..len as usize]), fonts.text(12.0), colors::dim()));
                }
                _ => {}
            }
            if !unlocked {
                tile.spawn(label("locked", fonts.text(12.0), colors::dim()));
                return;
            }
            spawn_stars(tile, drill.map_or(0, |d| d.stars()), 16.0, icons);
            let detail = match drill {
                Some(d) if d.bpm > 0 => {
                    let (bpm, target) = tempo_of(d);
                    if d.target_reached { format!("{target} BPM done") } else { format!("{bpm}/{target} BPM") }
                }
                Some(d) if d.plays > 0 => format!("best {:.0}%", d.best_accuracy * 100.0),
                _ => "new".to_string(),
            };
            tile.spawn(label(detail, fonts.text(12.0), colors::dim()));
            // The tier in words: the border is not the only sign of it.
            if let Some(tier) = tier {
                tile.spawn(label(tier.name(), fonts.bold(12.0), colors::text()));
            }
        })
        .observe(
            move |_: On<PointerClick>,
                  mut selected: ResMut<SelectedLevel>,
                  mut back: ResMut<ReturnTo>,
                  mut next: ResMut<NextState<Screen>>| {
                if unlocked {
                    selected.0 = key.clone();
                    back.0 = screen_of(page);
                    next.set(Screen::Drill);
                }
            },
        );
}
