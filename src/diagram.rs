//! A small diagram of a hand shape: the frets in columns, the strings in rows, a dot of the
//! finger's color on each fret played, as on the fretboard of the drills.
use bevy::prelude::*;

use crate::drills::{Shape, ShapeKind};
use crate::ui::{Fonts, colors, label};

const CELL: f32 = 30.0;
const DOT: f32 = 22.0;

/// The diagram of a shape: one row for one string; for an anchor the index on the lower row and
/// the other fingers `skip` strings higher.
pub fn spawn_shape_diagram(parent: &mut ChildSpawnerCommands, shape: &Shape, fonts: &Fonts) {
    let first = shape.start_fret;
    let last = first + shape.offsets[3];
    let fret_of = |finger: usize| first + shape.offsets[finger - 1];
    // The rows from the top: (the name of the string, the fingers on it).
    let rows: Vec<(String, Vec<usize>)> = match shape.kind {
        ShapeKind::OneString => vec![("string".into(), vec![1, 2, 3, 4])],
        ShapeKind::Anchor { skip } => {
            let higher = if skip == 1 { "next string".to_string() } else { format!("{skip} strings up") };
            vec![(higher, vec![2, 3, 4]), ("index string".into(), vec![1])]
        }
    };
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::axes(px(8), px(6)),
                border_radius: BorderRadius::all(px(6)),
                align_self: AlignSelf::Start,
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(colors::panel()),
            Pickable::IGNORE,
        ))
        .with_children(|d| {
            // The fret numbers.
            d.spawn(Node { flex_direction: FlexDirection::Row, ..default() }).with_children(|r| {
                r.spawn(Node { width: px(96), ..default() });
                for fret in first..=last {
                    r.spawn((
                        Node { width: px(CELL), justify_content: JustifyContent::Center, ..default() },
                        children![label(fret.to_string(), fonts.text(13.0), colors::dim())],
                    ));
                }
            });
            for (name, fingers) in rows {
                d.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, height: px(CELL), ..default() })
                    .with_children(|r| {
                        r.spawn((Node { width: px(96), ..default() }, children![label(name, fonts.text(13.0), colors::dim())]));
                        for fret in first..=last {
                            let finger = fingers.iter().copied().find(|f| fret_of(*f) == fret);
                            r.spawn((
                                Node {
                                    width: px(CELL),
                                    height: percent(100),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::right(px(2)),
                                    ..default()
                                },
                                BorderColor::all(colors::dim().with_alpha(0.6)),
                            ))
                            .with_children(|cell| {
                                // The string across the cell.
                                cell.spawn((
                                    Node {
                                        position_type: PositionType::Absolute,
                                        width: percent(100),
                                        height: px(1),
                                        top: percent(50),
                                        ..default()
                                    },
                                    BackgroundColor(colors::dim().with_alpha(0.6)),
                                ));
                                if let Some(f) = finger {
                                    cell.spawn((
                                        Node {
                                            width: px(DOT),
                                            height: px(DOT),
                                            border_radius: BorderRadius::MAX,
                                            justify_content: JustifyContent::Center,
                                            align_items: AlignItems::Center,
                                            ..default()
                                        },
                                        BackgroundColor(colors::FINGERS[f]),
                                        children![label(f.to_string(), fonts.bold(13.0), Color::WHITE)],
                                    ));
                                }
                            });
                        }
                    });
            }
        });
}

/// The finger colors with their names, the legend of every diagram and fretboard.
pub fn spawn_finger_legend(parent: &mut ChildSpawnerCommands, fonts: &Fonts) {
    parent
        .spawn(Node { flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, column_gap: px(8), row_gap: px(4), ..default() })
        .with_children(|row| {
            for (finger, name) in crate::drills::FINGER_NAMES.iter().enumerate().skip(1) {
                row.spawn((
                    Node { padding: UiRect::axes(px(8), px(2)), border_radius: BorderRadius::all(px(10)), ..default() },
                    BackgroundColor(colors::FINGERS[finger]),
                    children![label(format!("{finger} {name}"), fonts.bold(14.0), Color::WHITE)],
                ));
            }
        });
}
