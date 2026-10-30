//! The fretboard on the screen: the highest string on top as in a tablature, the open strings left
//! of the nut, the inlays on frets 3, 5, 7, 9 and the double one on 12 (and again from 15 to 24).
//! The frets get closer up the neck, as on the instrument but less.
//!
//! Every string and fret is a tap target: a tap writes a [`FretTapped`]. The screens draw their
//! notes in the [`MarksLayer`] with [`set_marks`].
use bevy::picking::events::PointerClick;
use bevy::prelude::*;

use crate::music::{Instrument, Position};
use crate::ui::{Fonts, colors, label};

/// The width of the open strings column, left of the nut, in percent.
const OPEN_COLUMN: f32 = 6.0;
/// The largest height of a string row.
const STRING_ROW: f32 = 46.0;
const MARK_SIZE: f32 = 28.0;

#[derive(Message, Debug, Clone, Copy)]
pub struct FretTapped(pub Position);

/// The layer of the notes drawn over the strings.
#[derive(Component)]
pub struct MarksLayer {
    pub strings: usize,
    pub frets: u8,
    pub left_handed: bool,
}

/// A mark that pulses, a hint.
#[derive(Component)]
pub struct Pulse;

/// A note to draw on the fretboard.
#[derive(Debug, Clone)]
pub struct Mark {
    pub position: Position,
    pub text: String,
    pub fill: Color,
    /// A ring around: the current note.
    pub ring: Option<Color>,
    pub big: bool,
    pub pulse: bool,
}

impl Mark {
    pub fn new(position: Position, text: impl Into<String>, fill: Color) -> Self {
        Mark { position, text: text.into(), fill, ring: None, big: false, pulse: false }
    }

    pub fn big(mut self) -> Self {
        self.big = true;
        self
    }

    pub fn ring(mut self, color: Color) -> Self {
        self.ring = Some(color);
        self
    }

    pub fn pulse(mut self) -> Self {
        self.pulse = true;
        self
    }
}

pub struct FretboardPlugin;

impl Plugin for FretboardPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<FretTapped>().add_systems(Update, pulse);
    }
}

/// The position of a fret wire from the left, in percent: a blend of the real distances
/// (`1 - 2^(-n/12)`) and equal ones, the high frets stay big enough to tap.
fn wire_x(fret: u8, frets: u8) -> f32 {
    let real = |n: f32| 1.0 - 2f32.powf(-n / 12.0);
    let t = 0.6 * real(fret as f32) / real(frets as f32) + 0.4 * fret as f32 / frets as f32;
    OPEN_COLUMN + (99.0 - OPEN_COLUMN) * t
}

/// The middle of a fret (or of the open column for fret 0), in percent from the left.
pub fn fret_center_x(fret: u8, frets: u8, left_handed: bool) -> f32 {
    let x = if fret == 0 { OPEN_COLUMN / 2.0 } else { (wire_x(fret - 1, frets) + wire_x(fret, frets)) / 2.0 };
    if left_handed { 100.0 - x } else { x }
}

fn mirror(x: f32, left_handed: bool) -> f32 {
    if left_handed { 100.0 - x } else { x }
}

/// The middle of a string row, in percent from the top: the highest string on top.
pub fn string_center_y(string: usize, strings: usize) -> f32 {
    let row = strings - 1 - string;
    (row as f32 + 0.5) / strings as f32 * 100.0
}

fn absolute(left: f32, top: f32) -> Node {
    Node { position_type: PositionType::Absolute, left: percent(left), top: percent(top), ..default() }
}

/// Spawns the fretboard: the board and the fret numbers under it.
pub fn spawn_fretboard(parent: &mut ChildSpawnerCommands, instrument: &Instrument, left_handed: bool, fonts: &Fonts) {
    let strings = instrument.strings();
    let frets = instrument.frets;
    let height = STRING_ROW * strings as f32;
    parent
        .spawn(Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            padding: UiRect::horizontal(px(8)),
            flex_shrink: 0.0,
            ..default()
        })
        .with_children(|column| {
            column
                .spawn((
                    Node {
                        width: percent(100),
                        height: vh(40),
                        max_height: px(height),
                        min_height: px(20.0 * strings as f32),
                        border_radius: BorderRadius::all(px(4)),
                        ..default()
                    },
                    BackgroundColor(colors::WOOD),
                ))
                .with_children(|board| spawn_board(board, instrument, left_handed, fonts));
            // The fret numbers.
            column.spawn(Node { width: percent(100), height: px(22), ..default() }).with_children(|numbers| {
                for fret in 0..=frets {
                    let marked = matches!(fret % 12, 3 | 5 | 7 | 9 | 0) && fret > 0;
                    let color = if marked { colors::TEXT } else { colors::DIM };
                    let x = fret_center_x(fret, frets, left_handed);
                    numbers.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: percent(x),
                            margin: UiRect::left(px(-8)),
                            width: px(16),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        Pickable::IGNORE,
                        children![label(fret.to_string(), fonts.text(13.0), color)],
                    ));
                }
            });
        });
}

fn spawn_board(board: &mut ChildSpawnerCommands, instrument: &Instrument, left_handed: bool, fonts: &Fonts) {
    let strings = instrument.strings();
    let frets = instrument.frets;
    // The inlays, under the strings.
    for fret in 1..=frets {
        let x = fret_center_x(fret, frets, left_handed);
        // Between two strings: the middle one, or a third from each edge for the double dots.
        let n = strings as f32;
        let third = (n / 3.0).round();
        let dots: Vec<f32> = match fret % 12 {
            0 => vec![third / n, (n - third) / n],
            3 | 5 | 7 | 9 => vec![(n / 2.0).floor() / n],
            _ => vec![],
        };
        for y in dots {
            board.spawn((
                Node {
                    margin: UiRect { left: px(-7), top: px(-7), ..default() },
                    width: px(14),
                    height: px(14),
                    border_radius: BorderRadius::MAX,
                    ..absolute(x, y * 100.0)
                },
                BackgroundColor(colors::INLAY.with_alpha(0.55)),
                Pickable::IGNORE,
            ));
        }
    }
    // The nut and the fret wires.
    for fret in 0..=frets {
        let x = mirror(wire_x(fret, frets), left_handed);
        let width = if fret == 0 { 6.0 } else { 3.0 };
        board.spawn((
            Node { margin: UiRect::left(px(-width / 2.0)), width: px(width), height: percent(100), ..absolute(x, 0.0) },
            BackgroundColor(if fret == 0 { colors::NUT } else { colors::FRET }),
            Pickable::IGNORE,
        ));
    }
    // The strings, thicker for the low ones, and their names left of the open column.
    for string in 0..strings {
        let y = string_center_y(string, strings);
        let thickness = 1.0 + (strings - 1 - string) as f32 * 0.45;
        board.spawn((
            Node { margin: UiRect::top(px(-thickness / 2.0)), width: percent(100), height: px(thickness), ..absolute(0.0, y) },
            BackgroundColor(colors::STRING),
            Pickable::IGNORE,
        ));
        let name_x = if left_handed { 100.5 } else { -0.5 };
        board.spawn((
            Node {
                margin: UiRect { left: px(-10), top: px(-9), ..default() },
                width: px(14),
                justify_content: JustifyContent::Center,
                ..absolute(name_x, y)
            },
            Pickable::IGNORE,
            children![label(instrument.string_label(string), fonts.text(13.0), colors::DIM)],
        ));
    }
    // The tap targets, one per string and fret.
    for string in 0..strings {
        let row = (strings - 1 - string) as f32;
        let top = row / strings as f32 * 100.0;
        let height = 100.0 / strings as f32;
        for fret in 0..=frets {
            let (a, b) = if fret == 0 { (0.0, OPEN_COLUMN) } else { (wire_x(fret - 1, frets), wire_x(fret, frets)) };
            let (left, right) = if left_handed { (100.0 - b, 100.0 - a) } else { (a, b) };
            let position = Position { string, fret };
            board
                .spawn((
                    Node { width: percent(right - left), height: percent(height), ..absolute(left, top) },
                    BackgroundColor(Color::NONE),
                ))
                .observe(move |_: On<PointerClick>, mut tapped: MessageWriter<FretTapped>| {
                    tapped.write(FretTapped(position));
                });
        }
    }
    board.spawn((
        Node { position_type: PositionType::Absolute, width: percent(100), height: percent(100), ..default() },
        MarksLayer { strings, frets, left_handed },
        Pickable::IGNORE,
    ));
}

/// Replaces the notes drawn on the fretboard.
pub fn set_marks(commands: &mut Commands, layer: (Entity, &MarksLayer), marks: &[Mark], fonts: &Fonts) {
    let (entity, geometry) = layer;
    commands.entity(entity).despawn_children();
    for mark in marks {
        if mark.position.string >= geometry.strings || mark.position.fret > geometry.frets {
            continue;
        }
        let x = fret_center_x(mark.position.fret, geometry.frets, geometry.left_handed);
        let y = string_center_y(mark.position.string, geometry.strings);
        let size = if mark.big { MARK_SIZE + 6.0 } else { MARK_SIZE };
        let border = if mark.ring.is_some() { 3.0 } else { 0.0 };
        let mut spawned = commands.spawn((
            Node {
                margin: UiRect { left: px(-size / 2.0), top: px(-size / 2.0), ..default() },
                width: px(size),
                height: px(size),
                border: UiRect::all(px(border)),
                border_radius: BorderRadius::MAX,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..absolute(x, y)
            },
            BackgroundColor(mark.fill),
            BorderColor::all(mark.ring.unwrap_or(Color::NONE)),
            UiTransform::default(),
            Pickable::IGNORE,
            ChildOf(entity),
            children![label(mark.text.clone(), fonts.bold(if mark.big { 17.0 } else { 14.0 }), Color::WHITE)],
        ));
        if mark.pulse {
            spawned.insert(Pulse);
        }
    }
}

fn pulse(time: Res<Time>, mut marks: Query<&mut UiTransform, With<Pulse>>) {
    let scale = 1.0 + 0.18 * (time.elapsed_secs() * 6.0).sin().abs();
    for mut transform in &mut marks {
        transform.scale = Vec2::splat(scale);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry() {
        assert_eq!(wire_x(0, 12), OPEN_COLUMN);
        assert!((wire_x(12, 12) - 99.0).abs() < 1e-4);
        // The frets get narrower up the neck.
        let w1 = wire_x(1, 12) - wire_x(0, 12);
        let w12 = wire_x(12, 12) - wire_x(11, 12);
        assert!(w1 > w12 * 1.3);
        assert!(fret_center_x(5, 12, true) > 50.0);
        assert!(string_center_y(0, 6) > string_center_y(5, 6));
    }
}
