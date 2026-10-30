//! The look shared by the screens: colors, fonts, buttons, the top bar with the microphone, the
//! scrolling, and the screen state.
use bevy::color::Mix;
use bevy::input::mouse::MouseScrollPixelsPerLine;
use bevy::picking::events::{PointerClick, PointerDrag, PointerScroll};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::{FontSize, FontSource};

use crate::audio::{Mic, MicStatus};
use crate::music::midi_name;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Screen {
    #[default]
    Menu,
    /// The list of the drill levels.
    Drills,
    /// A drill being played.
    Drill,
    Learn,
    Find,
    Settings,
    Stats,
}

impl Screen {
    /// Where Escape and the back button go.
    pub fn back(self) -> Screen {
        match self {
            Screen::Drill => Screen::Drills,
            _ => Screen::Menu,
        }
    }
}

pub mod colors {
    use bevy::prelude::Color;

    pub const BG: Color = Color::srgb(0.11, 0.12, 0.14);
    pub const PANEL: Color = Color::srgb(0.16, 0.17, 0.21);
    pub const BUTTON: Color = Color::srgb(0.22, 0.24, 0.29);
    pub const BUTTON_HOVER: Color = Color::srgb(0.29, 0.32, 0.39);
    pub const SELECTED: Color = Color::srgb(0.66, 0.45, 0.13);
    pub const TEXT: Color = Color::srgb(0.92, 0.91, 0.89);
    pub const DIM: Color = Color::srgb(0.60, 0.63, 0.66);
    pub const ACCENT: Color = Color::srgb(0.95, 0.68, 0.25);
    pub const OK: Color = Color::srgb(0.25, 0.73, 0.50);
    pub const BAD: Color = Color::srgb(0.90, 0.33, 0.29);
    pub const REVEAL: Color = Color::srgb(0.30, 0.56, 0.92);
    pub const WOOD: Color = Color::srgb(0.24, 0.16, 0.11);
    pub const FRET: Color = Color::srgb(0.72, 0.72, 0.70);
    pub const NUT: Color = Color::srgb(0.90, 0.87, 0.78);
    pub const STRING: Color = Color::srgb(0.85, 0.81, 0.72);
    pub const INLAY: Color = Color::srgb(0.88, 0.85, 0.78);
    pub const PAPER: Color = Color::srgb(0.95, 0.93, 0.88);
    pub const INK: Color = Color::srgb(0.10, 0.10, 0.12);
    pub const LOCKED: Color = Color::srgb(0.17, 0.18, 0.20);

    /// Index, middle, ring and pinky: the categorical palette of claudralingo (yellow, aqua,
    /// orange, blue), neighbors apart for the color blind; the numbers are always written too.
    pub const FINGERS: [Color; 5] = [
        Color::srgb(0.55, 0.55, 0.60),
        Color::srgb(0.79, 0.52, 0.0),
        Color::srgb(0.10, 0.62, 0.44),
        Color::srgb(0.85, 0.35, 0.15),
        Color::srgb(0.22, 0.53, 0.90),
    ];
}

#[derive(Resource, Clone)]
pub struct Fonts {
    pub regular: Handle<Font>,
    pub bold: Handle<Font>,
    /// Noto Music: the clefs, the sharps and the flats.
    pub music: Handle<Font>,
}

impl Fonts {
    pub fn text(&self, size: f32) -> TextFont {
        TextFont { font: FontSource::Handle(self.regular.clone()), font_size: FontSize::Px(size), ..default() }
    }

    pub fn bold(&self, size: f32) -> TextFont {
        TextFont { font: FontSource::Handle(self.bold.clone()), font_size: FontSize::Px(size), ..default() }
    }

    pub fn music(&self, size: f32) -> TextFont {
        TextFont { font: FontSource::Handle(self.music.clone()), font_size: FontSize::Px(size), ..default() }
    }
}

/// A button: its colors at rest and under the pointer.
#[derive(Component, Clone, Copy)]
pub struct Btn {
    pub color: Color,
    pub hover: Color,
}

/// The text of the microphone in the top bar.
#[derive(Component)]
struct MicText;

/// The bar of the microphone level.
#[derive(Component)]
struct MicLevel;

/// A node scrolled by the wheel and by dragging.
#[derive(Component)]
pub struct Scrollable;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<Screen>()
            .add_systems(Startup, setup)
            .add_systems(Update, (hover_buttons, update_mic, escape_goes_back, scale_to_window));
    }

    /// The fonts are there before the first screen is entered, which happens before `Startup`.
    fn finish(&self, app: &mut App) {
        app.init_resource::<Fonts>();
    }
}

impl FromWorld for Fonts {
    fn from_world(world: &mut World) -> Self {
        let mut fonts = world.resource_mut::<Assets<Font>>();
        let mut load = |bytes: &[u8]| fonts.add(Font::from_bytes(bytes.to_vec()));
        Fonts {
            regular: load(include_bytes!("../assets/fonts/FiraSans-Regular.ttf")),
            bold: load(include_bytes!("../assets/fonts/FiraSans-Bold.ttf")),
            music: load(include_bytes!("../assets/fonts/NotoMusic-Regular.ttf")),
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, Camera { clear_color: ClearColorConfig::Custom(colors::BG), ..default() }));
}

/// Smaller windows (phones) scale the whole interface down: the screens are laid out for about
/// 1150x800 logical pixels.
fn scale_to_window(windows: Query<&Window, Changed<Window>>, mut scale: ResMut<UiScale>) {
    let Some(window) = windows.iter().next() else {
        return;
    };
    let fit = (window.height() / 800.0).min(window.width() / 1150.0).clamp(0.6, 1.0);
    if (scale.0 - fit).abs() > 0.01 {
        scale.0 = fit;
    }
}

fn hover_buttons(mut buttons: Query<(&Hovered, &Btn, &mut BackgroundColor), Changed<Hovered>>) {
    for (hovered, btn, mut bg) in &mut buttons {
        bg.0 = if hovered.get() { btn.hover } else { btn.color };
    }
}

fn escape_goes_back(keys: Res<ButtonInput<KeyCode>>, screen: Res<State<Screen>>, mut next: ResMut<NextState<Screen>>) {
    if keys.just_pressed(KeyCode::Escape) && *screen.get() != Screen::Menu {
        next.set(screen.get().back());
    }
}

/// A text bundle.
pub fn label(text: impl Into<String>, font: TextFont, color: Color) -> impl Bundle {
    (Text::new(text), font, TextColor(color), Pickable::IGNORE)
}

/// A button without content, its `Node` can be replaced.
pub fn button_frame(color: Color) -> impl Bundle {
    let hover = color.mix(&Color::WHITE, 0.12);
    (
        Node {
            padding: UiRect::axes(px(16), px(9)),
            border_radius: BorderRadius::all(px(8)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(color),
        Btn { color, hover },
        Hovered::default(),
    )
}

/// A button with a text; the click is an observer of the caller.
pub fn button(text: impl Into<String>, fonts: &Fonts, color: Color) -> impl Bundle {
    (button_frame(color), children![label(text, fonts.text(18.0), colors::TEXT)])
}

/// A smaller square button, for the + and - of the settings.
pub fn small_button(text: impl Into<String>, fonts: &Fonts) -> impl Bundle {
    (
        Node {
            width: px(38),
            height: px(34),
            border_radius: BorderRadius::all(px(6)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(colors::BUTTON),
        Btn { color: colors::BUTTON, hover: colors::BUTTON_HOVER },
        Hovered::default(),
        children![label(text, fonts.bold(20.0), colors::TEXT)],
    )
}

/// The root of a screen: the whole window, a column, despawned when the screen is left.
pub fn screen_root(screen: Screen) -> impl Bundle {
    (Node { width: percent(100), height: percent(100), flex_direction: FlexDirection::Column, ..default() }, DespawnOnExit(screen))
}

/// The bar on top of every screen: back, the title, the microphone.
pub fn spawn_top_bar(parent: &mut ChildSpawnerCommands, title: &str, screen: Screen, fonts: &Fonts) {
    parent
        .spawn((
            Node {
                width: percent(100),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: px(12),
                padding: UiRect::axes(px(12), px(8)),
                ..default()
            },
            BackgroundColor(colors::PANEL),
        ))
        .with_children(|bar| {
            if screen != Screen::Menu {
                bar.spawn(button("< Back", fonts, colors::BUTTON)).observe(
                    move |_: On<PointerClick>, mut next: ResMut<NextState<Screen>>| {
                        next.set(screen.back());
                    },
                );
            }
            bar.spawn(label(title, fonts.bold(22.0), colors::TEXT));
            bar.spawn(Node { flex_grow: 1.0, ..default() });
            bar.spawn((label("", fonts.text(15.0), colors::DIM), MicText));
            bar.spawn((
                Node { width: px(70), height: px(8), border_radius: BorderRadius::all(px(4)), overflow: Overflow::clip(), ..default() },
                BackgroundColor(colors::BUTTON),
                Pickable::IGNORE,
            ))
            .with_children(|track| {
                track.spawn((
                    Node { width: percent(0), height: percent(100), ..default() },
                    BackgroundColor(colors::OK),
                    MicLevel,
                    Pickable::IGNORE,
                ));
            });
        });
}

fn update_mic(
    mic: Res<Mic>,
    mut texts: Query<&mut Text, With<MicText>>,
    mut levels: Query<&mut Node, With<MicLevel>>,
    added: Query<(), Added<MicText>>,
) {
    if !mic.is_changed() && added.is_empty() {
        return;
    }
    let shown = match (&mic.status, mic.note) {
        (MicStatus::Listening, Some(note)) => format!("Hearing {}", midi_name(note)),
        (MicStatus::Listening, None) => "Listening".to_string(),
        (status, _) => status.describe(),
    };
    for mut text in &mut texts {
        if text.0 != shown {
            text.0 = shown.clone();
        }
    }
    // A logarithmic meter from -60 dB.
    let db = 20.0 * mic.level.max(1e-6).log10();
    let fill = ((db + 60.0) / 60.0).clamp(0.0, 1.0) * 100.0;
    for mut node in &mut levels {
        node.width = percent(fill);
    }
}

/// The wheel scrolls the [`Scrollable`] under the pointer, a drag (a finger) too.
pub fn scroll_on_wheel(
    mut scroll: On<PointerScroll>,
    ratio: Res<MouseScrollPixelsPerLine>,
    mut areas: Query<(&ComputedNode, &mut ScrollPosition), With<Scrollable>>,
) {
    scroll.propagate(false);
    let delta = scroll.to_pixels(&ratio);
    if let Ok((node, mut position)) = areas.get_mut(scroll.entity) {
        scroll_by(node, &mut position, -delta.y);
    }
}

pub fn scroll_on_drag(mut drag: On<PointerDrag>, mut areas: Query<(&ComputedNode, &mut ScrollPosition), With<Scrollable>>) {
    drag.propagate(false);
    let dy = -drag.delta.y;
    if let Ok((node, mut position)) = areas.get_mut(drag.entity) {
        scroll_by(node, &mut position, dy);
    }
}

fn scroll_by(node: &ComputedNode, position: &mut ScrollPosition, dy: f32) {
    let visible = node.size().y * node.inverse_scale_factor;
    let content = node.content_size().y * node.inverse_scale_factor;
    let y = (position.y + dy).clamp(0.0, (content - visible).max(0.0));
    if position.y != y {
        position.y = y;
    }
}

/// A scrolling column filling the rest of the screen.
pub fn scroll_area() -> impl Bundle {
    (
        Node {
            width: percent(100),
            flex_grow: 1.0,
            min_height: px(0),
            flex_direction: FlexDirection::Column,
            row_gap: px(12),
            padding: UiRect::all(px(16)),
            overflow: Overflow::scroll_y(),
            ..default()
        },
        ScrollPosition::default(),
        Scrollable,
    )
}

/// Formats a ratio as a percentage.
pub fn percent_text(ratio: f32) -> String {
    format!("{:.0}%", ratio * 100.0)
}
