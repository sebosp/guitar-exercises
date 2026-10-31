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
    /// The scale exercises.
    Scales,
    /// The Guitar Pro songs.
    Songs,
    /// A drill being played.
    Drill,
    Learn,
    Find,
    /// The shapes of the fretboard, the first level of "Find them all".
    FindTutorial,
    Settings,
    Stats,
    Help,
}

impl Screen {
    /// Where Escape and the back button go: a drill back to the page it was opened from.
    pub fn back(self, drill_from: &ReturnTo) -> Screen {
        match self {
            Screen::Drill => drill_from.0,
            Screen::FindTutorial => Screen::Find,
            _ => Screen::Menu,
        }
    }
}

/// The page a drill was opened from: the drills or the scales.
#[derive(Resource, Debug, Clone, Copy)]
pub struct ReturnTo(pub Screen);

impl Default for ReturnTo {
    fn default() -> Self {
        ReturnTo(Screen::Drills)
    }
}

pub mod colors {
    use bevy::prelude::{Alpha, Color};

    use crate::theme::current;

    /// The colors of the theme in use, see [`crate::theme`].
    pub fn bg() -> Color {
        current().palette().bg
    }
    pub fn panel() -> Color {
        current().palette().panel
    }
    pub fn button() -> Color {
        current().palette().button
    }
    pub fn button_hover() -> Color {
        current().palette().button_hover
    }
    pub fn selected() -> Color {
        current().palette().selected
    }
    pub fn text() -> Color {
        current().palette().text
    }
    pub fn dim() -> Color {
        current().palette().dim
    }
    pub fn accent() -> Color {
        current().palette().accent
    }
    pub fn ok() -> Color {
        current().palette().ok
    }
    pub fn bad() -> Color {
        current().palette().bad
    }
    pub fn reveal() -> Color {
        current().palette().reveal
    }
    pub fn locked() -> Color {
        current().palette().locked
    }
    /// The staff: the panel with lines of the text color.
    pub fn paper() -> Color {
        panel()
    }
    pub fn ink() -> Color {
        text()
    }

    /// The fretboard, the same in every theme.
    pub const WOOD: Color = Color::srgb(0.24, 0.16, 0.11);
    pub const FRET: Color = Color::srgb(0.72, 0.72, 0.70);
    pub const NUT: Color = Color::srgb(0.90, 0.87, 0.78);
    pub const STRING: Color = Color::srgb(0.85, 0.81, 0.72);
    pub const INLAY: Color = Color::srgb(0.88, 0.85, 0.78);

    /// Open, index, middle, ring and pinky, the same in every theme: yellow, aqua, violet and
    /// deep blue, every pair apart for the color blind (OKLab delta E 8.4 at worst under protan,
    /// 17 at worst for normal vision) on the dark and the light surfaces; the numbers are always
    /// written on them too.
    pub const FINGERS: [Color; 5] = [
        Color::srgb(0.55, 0.55, 0.60),
        Color::srgb_u8(0xc9, 0x85, 0x00),
        Color::srgb_u8(0x19, 0x9e, 0x70),
        Color::srgb_u8(0x90, 0x85, 0xe9),
        Color::srgb_u8(0x00, 0x72, 0xb2),
    ];

    /// A note already played on the fretboard and in the tablature: neutral, not a finger color.
    pub fn played() -> Color {
        dim().with_alpha(0.7)
    }
}

#[derive(Resource, Clone)]
pub struct Fonts {
    pub regular: Handle<Font>,
    pub bold: Handle<Font>,
    /// Noto Music: the clefs, the sharps and the flats.
    pub music: Handle<Font>,
    /// Barlow Condensed SemiBold: the titles, the labels and the numbers, drawn for road signs
    /// read from afar as the screen is from the instrument.
    pub display: Handle<Font>,
}

impl Fonts {
    pub fn text(&self, size: f32) -> TextFont {
        TextFont { font: FontSource::Handle(self.regular.clone()), font_size: FontSize::Px(size), ..default() }
    }

    pub fn bold(&self, size: f32) -> TextFont {
        TextFont { font: FontSource::Handle(self.bold.clone()), font_size: FontSize::Px(size), ..default() }
    }

    pub fn display(&self, size: f32) -> TextFont {
        TextFont { font: FontSource::Handle(self.display.clone()), font_size: FontSize::Px(size), ..default() }
    }

    pub fn music(&self, size: f32) -> TextFont {
        TextFont { font: FontSource::Handle(self.music.clone()), font_size: FontSize::Px(size), ..default() }
    }
}

/// The type scale, about 1.25 apart: the captions, the text, the lead text, the headings, the
/// titles, the numbers, the note to play.
pub mod size {
    pub const CAPTION: f32 = 13.0;
    pub const TEXT: f32 = 15.0;
    pub const LEAD: f32 = 17.0;
    pub const HEADING: f32 = 21.0;
    pub const TITLE: f32 = 27.0;
    pub const NUMBER: f32 = 34.0;
    pub const HERO: f32 = 44.0;
}

/// The icons drawn by the game: a star, white, tinted by [`ImageNode::color`].
#[derive(Resource, Clone)]
pub struct Icons {
    pub star: Handle<Image>,
    pub gear: Handle<Image>,
}

impl FromWorld for Icons {
    fn from_world(world: &mut World) -> Self {
        let mut images = world.resource_mut::<Assets<Image>>();
        Icons { star: images.add(star_image(64)), gear: images.add(gear_image(64)) }
    }
}

/// A shape in white with soft edges (4x4 samples per pixel), from a test of the points of the
/// unit square.
fn shape_image(size: u32, inside: impl Fn(Vec2) -> bool) -> Image {
    use bevy::asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let hits = (0..16)
                .filter(|s| {
                    let p = Vec2::new(x as f32 + (s % 4) as f32 / 4.0 + 0.125, y as f32 + (s / 4) as f32 / 4.0 + 0.125) / size as f32;
                    inside(p)
                })
                .count();
            data.extend_from_slice(&[255, 255, 255, (hits * 255 / 16) as u8]);
        }
    }
    Image::new(
        Extent3d { width: size, height: size, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

/// A five-pointed star.
fn star_image(size: u32) -> Image {
    // The ten corners, the outer ones on a circle, the inner ones on a smaller one.
    let corners: Vec<Vec2> = (0..10)
        .map(|i| {
            let angle = std::f32::consts::PI * (i as f32 / 5.0 - 0.5);
            let radius = if i % 2 == 0 { 0.5 } else { 0.2 };
            Vec2::new(0.5 + radius * angle.cos(), 0.53 + radius * angle.sin())
        })
        .collect();
    shape_image(size, |p| {
        let mut inside = false;
        for (i, a) in corners.iter().enumerate() {
            let b = corners[(i + 1) % corners.len()];
            if (a.y > p.y) != (b.y > p.y) && p.x < (b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x {
                inside = !inside;
            }
        }
        inside
    })
}

/// A gear: a ring with eight teeth around a hole.
fn gear_image(size: u32) -> Image {
    shape_image(size, |p| {
        let d = p - Vec2::splat(0.5);
        let r = d.length();
        let teeth = 8.0;
        let phase = (d.y.atan2(d.x) / std::f32::consts::TAU * teeth).rem_euclid(1.0);
        let outer = if (0.25..0.75).contains(&phase) { 0.48 } else { 0.36 };
        r <= outer && r >= 0.15
    })
}

/// Three stars, `earned` of them lit.
pub fn spawn_stars(parent: &mut ChildSpawnerCommands, earned: u8, size: f32, icons: &Icons) {
    parent.spawn((Node { flex_direction: FlexDirection::Row, column_gap: px(1), ..default() }, Pickable::IGNORE)).with_children(|row| {
        for n in 0..3 {
            let color = if n < earned { STAR } else { colors::dim().with_alpha(0.35) };
            row.spawn((
                Node { width: px(size), height: px(size), ..default() },
                ImageNode::new(icons.star.clone()).with_color(color),
                Pickable::IGNORE,
            ));
        }
    });
}

/// The gold of the stars, the same in every theme.
pub const STAR: Color = Color::srgb(1.0, 0.78, 0.16);

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

/// The scroll of the screen shown. A screen built again in place (a setting changed on it, a
/// level opened at random, another theme, the next level) comes back where it was; another
/// screen starts at the top.
#[derive(Resource, Default)]
struct ScrollMemory {
    y: f32,
    /// The scroll to put back, for a few frames: the first layout of the new screen may not
    /// have all its height yet and would clamp it.
    restore: Option<(f32, u8)>,
}

/// The frames a scroll is put back for.
const RESTORE_FRAMES: u8 = 6;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<Screen>()
            .init_resource::<ReturnTo>()
            .init_resource::<ScrollMemory>()
            .add_systems(Update, (screen_changed, restore_scroll, remember_scroll).chain())
            .add_systems(Startup, setup)
            .add_systems(Update, (hover_buttons, update_mic, escape_goes_back, apply_theme, fill_gears));
    }

    /// The fonts are there before the first screen is entered, which happens before `Startup`.
    fn finish(&self, app: &mut App) {
        app.init_resource::<Fonts>().init_resource::<Icons>();
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
            display: load(include_bytes!("../assets/fonts/BarlowCondensed-SemiBold.ttf")),
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((Camera2d, Camera { clear_color: ClearColorConfig::Custom(colors::bg()), ..default() }));
}

/// Another theme: the background, and the screen spawned again in its colors.
fn apply_theme(
    settings: Res<crate::settings::Settings>,
    mut cameras: Query<&mut Camera>,
    screen: Res<State<Screen>>,
    mut next: ResMut<NextState<Screen>>,
) {
    if !settings.is_changed() || settings.theme == crate::theme::current() {
        return;
    }
    crate::theme::set(settings.theme);
    for mut camera in &mut cameras {
        camera.clear_color = ClearColorConfig::Custom(colors::bg());
    }
    next.set(*screen.get());
}

fn hover_buttons(mut buttons: Query<(&Hovered, &Btn, &mut BackgroundColor), Changed<Hovered>>) {
    for (hovered, btn, mut bg) in &mut buttons {
        bg.0 = if hovered.get() { btn.hover } else { btn.color };
    }
}

fn escape_goes_back(keys: Res<ButtonInput<KeyCode>>, screen: Res<State<Screen>>, from: Res<ReturnTo>, mut next: ResMut<NextState<Screen>>) {
    if keys.just_pressed(KeyCode::Escape) && *screen.get() != Screen::Menu {
        next.set(screen.get().back(&from));
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
            border_radius: BorderRadius::all(px(6)),
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
    (button_frame(color), children![label(text, fonts.text(18.0), colors::text())])
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
        BackgroundColor(colors::button()),
        Btn { color: colors::button(), hover: colors::button_hover() },
        Hovered::default(),
        children![label(text, fonts.bold(20.0), colors::text())],
    )
}

/// The root of a screen: the whole window, a column, despawned when the screen is left. In
/// viewport units: the percentages of a root node grow with the content, and the scroll areas
/// under it would never scroll.
pub fn screen_root(screen: Screen) -> impl Bundle {
    (Node { width: vw(100), height: vh(100), flex_direction: FlexDirection::Column, ..default() }, DespawnOnExit(screen))
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
            BackgroundColor(colors::panel()),
        ))
        .with_children(|bar| {
            if screen != Screen::Menu {
                bar.spawn(button("< Back", fonts, colors::button())).observe(
                    move |_: On<PointerClick>, from: Res<ReturnTo>, mut next: ResMut<NextState<Screen>>| {
                        next.set(screen.back(&from));
                    },
                );
            }
            bar.spawn(label(title, fonts.display(size::TITLE), colors::text()));
            bar.spawn(Node { flex_grow: 1.0, ..default() });
            bar.spawn((label("", fonts.text(15.0), colors::dim()), MicText));
            bar.spawn((
                Node { width: px(70), height: px(8), border_radius: BorderRadius::all(px(4)), overflow: Overflow::clip(), ..default() },
                BackgroundColor(colors::button()),
                Pickable::IGNORE,
            ))
            .with_children(|track| {
                track.spawn((
                    Node { width: percent(0), height: percent(100), ..default() },
                    BackgroundColor(colors::ok()),
                    MicLevel,
                    Pickable::IGNORE,
                ));
            });
            // The statistics, the help and the settings, from anywhere.
            for (target, kind) in [(Screen::Stats, BarIcon::Stats), (Screen::Help, BarIcon::Help), (Screen::Settings, BarIcon::Settings)] {
                if target == screen {
                    continue;
                }
                bar.spawn((
                    Node {
                        width: px(40),
                        height: px(36),
                        border_radius: BorderRadius::all(px(6)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::End,
                        column_gap: px(3),
                        padding: UiRect::bottom(px(if kind == BarIcon::Help { 2.0 } else { 9.0 })),
                        ..default()
                    },
                    BackgroundColor(colors::button()),
                    Btn { color: colors::button(), hover: colors::button_hover() },
                    Hovered::default(),
                ))
                .with_children(|b| match kind {
                    BarIcon::Stats => {
                        for height in [9.0, 15.0, 12.0] {
                            b.spawn((
                                Node { width: px(5), height: px(height), ..default() },
                                BackgroundColor(colors::text()),
                                Pickable::IGNORE,
                            ));
                        }
                    }
                    BarIcon::Help => {
                        b.spawn(label("?", fonts.display(size::TITLE), colors::text()));
                    }
                    BarIcon::Settings => {
                        b.spawn((
                            Node { width: px(22), height: px(22), margin: UiRect::bottom(px(-2)), ..default() },
                            GearSlot,
                            Pickable::IGNORE,
                        ));
                    }
                })
                .observe(move |_: On<PointerClick>, mut next: ResMut<NextState<Screen>>| next.set(target));
            }
        });
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BarIcon {
    Stats,
    Help,
    Settings,
}

/// Where the gear of the settings goes, its image added by [`fill_gears`].
#[derive(Component)]
struct GearSlot;

fn fill_gears(mut commands: Commands, slots: Query<Entity, Added<GearSlot>>, icons: Res<Icons>) {
    for slot in &slots {
        commands.entity(slot).insert(ImageNode::new(icons.gear.clone()).with_color(colors::text()));
    }
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

fn screen_changed(mut transitions: MessageReader<StateTransitionEvent<Screen>>, mut memory: ResMut<ScrollMemory>) {
    for transition in transitions.read() {
        if transition.exited == transition.entered {
            memory.restore = Some((memory.y, RESTORE_FRAMES));
        } else {
            *memory = ScrollMemory::default();
        }
    }
}

fn restore_scroll(mut memory: ResMut<ScrollMemory>, mut areas: Query<&mut ScrollPosition, With<Scrollable>>) {
    let Some((y, frames)) = memory.restore else {
        return;
    };
    for mut position in &mut areas {
        if position.y != y {
            position.y = y;
        }
    }
    memory.restore = frames.checked_sub(1).filter(|f| *f > 0).map(|f| (y, f));
}

fn remember_scroll(mut memory: ResMut<ScrollMemory>, areas: Query<&ScrollPosition, (With<Scrollable>, Changed<ScrollPosition>)>) {
    if memory.restore.is_some() {
        return;
    }
    for position in &areas {
        memory.y = position.y;
    }
}

/// The wheel scrolls the [`Scrollable`] under the pointer, a drag (a finger) too; not with Ctrl.
pub fn scroll_on_wheel(
    mut scroll: On<PointerScroll>,
    ratio: Res<MouseScrollPixelsPerLine>,
    keys: Res<ButtonInput<KeyCode>>,
    mut areas: Query<(&ComputedNode, &mut ScrollPosition), With<Scrollable>>,
) {
    scroll.propagate(false);
    // With Ctrl the wheel zooms, see `crate::zoom`.
    if crate::zoom::ctrl_held(&keys) {
        return;
    }
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
