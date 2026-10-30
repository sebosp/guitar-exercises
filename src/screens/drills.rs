//! The drill levels, grouped by hand shape: the finger orders of each shape, done or not, locked
//! until the previous one is done.
use bevy::picking::events::PointerClick;
use bevy::prelude::*;

use crate::drills::{FINGER_NAMES, Level, SHAPES, ShapeKind, is_unlocked, levels};
use crate::progress::Progress;
use crate::settings::Settings;
use crate::ui::{Fonts, Screen, button_frame, colors, label, percent_text, screen_root, scroll_area, spawn_top_bar};

/// The level to play.
#[derive(Resource, Default, Clone, Copy)]
pub struct SelectedLevel(pub usize);

pub struct DrillsPlugin;

impl Plugin for DrillsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedLevel>().add_systems(OnEnter(Screen::Drills), spawn);
    }
}

/// How the fingers sit, `index fret 1, middle 3, ring 4, pinky 5`.
pub fn shape_description(group: usize) -> String {
    let shape = &SHAPES[group];
    let frets: Vec<String> = (0..4).map(|f| format!("{} {}", FINGER_NAMES[f + 1], shape.start_fret + shape.offsets[f])).collect();
    let strings = match shape.kind {
        ShapeKind::OneString => "on one string".to_string(),
        ShapeKind::Anchor { skip: 1 } => "index on a string, the others on the next one".to_string(),
        ShapeKind::Anchor { skip } => format!("index on a string, the others {skip} strings higher"),
    };
    format!("Frets: {}; {strings}", frets.join(", "))
}

fn spawn(mut commands: Commands, fonts: Res<Fonts>, settings: Res<Settings>, progress: Res<Progress>) {
    let fonts = &*fonts;
    let key = settings.instrument().key();
    let record = progress.get(&key).cloned().unwrap_or_default();
    let all = levels();
    commands.spawn(screen_root(Screen::Drills)).with_children(|root| {
        spawn_top_bar(root, "Finger drills", Screen::Drills, fonts);
        root.spawn(scroll_area()).observe(crate::ui::scroll_on_wheel).observe(crate::ui::scroll_on_drag).with_children(|list| {
            list.spawn(label(
                "Up the strings, one fret higher, back down. 90% accuracy opens the next order; three orders done open the next shape.",
                fonts.text(16.0),
                colors::DIM,
            ));
            for (group, shape) in SHAPES.iter().enumerate() {
                list.spawn(label(format!("{}. {}", group + 1, shape.name), fonts.bold(20.0), colors::ACCENT));
                list.spawn(label(shape_description(group), fonts.text(15.0), colors::DIM));
                list.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: px(8),
                    row_gap: px(8),
                    ..default()
                })
                .with_children(|grid| {
                    for level in all.iter().filter(|l| l.group == group) {
                        spawn_tile(grid, level, &record, fonts);
                    }
                });
            }
        });
    });
}

fn spawn_tile(grid: &mut ChildSpawnerCommands, level: &Level, record: &crate::progress::InstrumentProgress, fonts: &Fonts) {
    let unlocked = is_unlocked(level, |id| record.drill_done(id));
    let drill = record.drills.get(&level.id);
    let (detail, detail_color) = match drill {
        Some(d) if d.done => (format!("{} · {:.0}/min", percent_text(d.best_accuracy), d.best_notes_per_minute), colors::OK),
        Some(d) => (percent_text(d.best_accuracy), colors::BAD),
        None if unlocked => ("new".to_string(), colors::DIM),
        None => ("locked".to_string(), colors::DIM),
    };
    let color = if unlocked { colors::BUTTON } else { colors::LOCKED };
    let id = level.id;
    grid.spawn(button_frame(color))
        .insert(Node {
            width: px(104),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::axes(px(6), px(6)),
            border_radius: BorderRadius::all(px(8)),
            ..default()
        })
        .with_children(|tile| {
            tile.spawn(label(level.order_name(), fonts.bold(18.0), if unlocked { colors::TEXT } else { colors::DIM }));
            tile.spawn(label(detail, fonts.text(13.0), detail_color));
        })
        .observe(move |_: On<PointerClick>, mut selected: ResMut<SelectedLevel>, mut next: ResMut<NextState<Screen>>| {
            if unlocked {
                selected.0 = id;
                next.set(Screen::Drill);
            }
        });
}
