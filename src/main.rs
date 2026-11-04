use bevy::prelude::*;
use bevy::render::{
    RenderPlugin,
    settings::{PowerPreference, WgpuSettings},
};
use guitar_exercises::GuitarExercisesPlugin;

fn main() {
    #[cfg(target_os = "linux")]
    leave_nvidia_asleep();
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Guitar Exercises".into(),
                    // The app id of Wayland and the class of X11.
                    name: Some("guitar-exercises".into()),
                    resolution: (1280, 860).into(),
                    // The web build fills the page, see index.html.
                    canvas: Some("#guitar-exercises".into()),
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            })
            .set(RenderPlugin {
                // The integrated GPU: a discrete one awake costs a laptop about 20 W, and nothing
                // here needs it. `WGPU_POWER_PREF=high` still picks the discrete one.
                render_creation: WgpuSettings {
                    power_preference: PowerPreference::from_env().unwrap_or(PowerPreference::LowPower),
                    ..default()
                }
                .into(),
                ..default()
            }),
    )
    .add_plugins(GuitarExercisesPlugin);
    #[cfg(not(target_arch = "wasm32"))]
    snapshot::add(&mut app);
    app.run();
}

/// Keeps a discrete NVIDIA GPU asleep when another GPU can draw: the Vulkan loader starts every
/// driver to list the GPUs, and the NVIDIA one stays awake (about 20 W on a laptop) even with the
/// integrated GPU picked. `WGPU_POWER_PREF=high` or any choice of Vulkan driver in the environment
/// keeps all of them.
#[cfg(target_os = "linux")]
fn leave_nvidia_asleep() {
    let chosen = ["VK_LOADER_DRIVERS_DISABLE", "VK_LOADER_DRIVERS_SELECT", "VK_DRIVER_FILES", "VK_ICD_FILENAMES"]
        .iter()
        .any(|name| std::env::var_os(name).is_some());
    if chosen || std::env::var("WGPU_POWER_PREF").is_ok_and(|pref| pref == "high") {
        return;
    }
    let vendors: Vec<String> = std::fs::read_dir("/sys/class/drm")
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|card| std::fs::read_to_string(card.path().join("device/vendor")).ok())
        .map(|vendor| vendor.trim().to_string())
        .collect();
    let drivers: Vec<String> = ["/usr/share/vulkan/icd.d", "/etc/vulkan/icd.d"]
        .iter()
        .filter_map(|dir| std::fs::read_dir(dir).ok())
        .flatten()
        .flatten()
        .map(|driver| driver.file_name().to_string_lossy().to_string())
        .collect();
    let has_driver = |name: &str| drivers.iter().any(|driver| driver.contains(name));
    let another_can_draw = vendors.iter().any(|vendor| vendor == "0x1002") && has_driver("radeon")
        || vendors.iter().any(|vendor| vendor == "0x8086") && has_driver("intel");
    if vendors.iter().any(|vendor| vendor == "0x10de") && another_can_draw {
        // SAFETY: before any thread is started.
        unsafe { std::env::set_var("VK_LOADER_DRIVERS_DISABLE", "*nvidia*") };
    }
}

/// `GE_SCREENSHOT=out.png` opens a screen, plays a few notes by itself, saves a screenshot and
/// exits, to check the screens without a person at the instrument:
///
/// - `GE_SCREEN`: menu, drills, scales, drill, learn, find, shapes, settings, stats, help or songs;
/// - `GE_LEVEL`: the drill, by its place on the drills page, or `GE_EXERCISE` by its key;
/// - `GE_PLAY`: the notes played right (heard by the "microphone"), `GE_TAP` tapped right,
///   `GE_MISS` wrong notes heard;
/// - `GE_WAIT`: seconds to wait after them, for the hints;
/// - `GE_SCROLL`: mouse wheel lines down over the middle of the window, through the picking;
/// - `GE_IMPORT`: a Guitar Pro file imported as if dropped on the window;
/// - `GE_REBUILD=1`: the screen built again after the scroll, as a setting changed on it does;
/// - `GE_SIZE`: the window size, `1280x860` by default;
/// - `GE_SOUND=1`: the clicks of the metronome out loud.
///
/// The microphone is off, and the clicks of the metronome unless `GE_SOUND=1`. Use `XDG_CONFIG_HOME` to keep the saves away from the real ones.
#[cfg(not(target_arch = "wasm32"))]
mod snapshot {
    use bevy::camera::NormalizedRenderTarget;
    use bevy::input::mouse::MouseScrollUnit;
    use bevy::input::touch::TouchPhase;
    use bevy::picking::pointer::{Location, PointerAction, PointerId, PointerInput};
    use bevy::prelude::*;
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};
    use bevy::window::{PrimaryWindow, WindowRef, WindowResolution};
    use guitar_exercises::audio::NoteHeard;
    use guitar_exercises::fretboard::FretTapped;
    use guitar_exercises::metronome::Metronome;
    use guitar_exercises::screens::Expected;
    use guitar_exercises::screens::drills::SelectedLevel;
    use guitar_exercises::settings::Settings;
    use guitar_exercises::ui::Screen;

    #[derive(Resource)]
    struct Snapshot {
        path: String,
        screen: Screen,
        play: u32,
        tap: u32,
        miss: u32,
        wait: f32,
        scroll: f32,
        rebuild: bool,
    }

    fn var<T: std::str::FromStr>(name: &str, default: T) -> T {
        std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
    }

    pub fn add(app: &mut App) {
        let Ok(path) = std::env::var("GE_SCREENSHOT") else {
            return;
        };
        let screen = match std::env::var("GE_SCREEN").unwrap_or_default().as_str() {
            "drills" => Screen::Drills,
            "scales" => Screen::Scales,
            "songs" => Screen::Songs,
            "drill" => Screen::Drill,
            "learn" => Screen::Learn,
            "find" => Screen::Find,
            "shapes" => Screen::FindTutorial,
            "settings" => Screen::Settings,
            "stats" => Screen::Stats,
            "help" => Screen::Help,
            _ => Screen::Menu,
        };
        app.insert_resource(Snapshot {
            path,
            screen,
            play: var("GE_PLAY", 0),
            tap: var("GE_TAP", 0),
            miss: var("GE_MISS", 0),
            wait: var("GE_WAIT", 0.0),
            scroll: var("GE_SCROLL", 0.0),
            rebuild: var("GE_REBUILD", 0) == 1,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, run);
    }

    fn setup(
        mut window: Single<&mut Window, With<PrimaryWindow>>,
        mut settings: ResMut<Settings>,
        mut metronome: ResMut<Metronome>,
        mut selected: ResMut<SelectedLevel>,
        mut incoming: ResMut<guitar_exercises::library::Incoming>,
    ) {
        if let Ok(path) = std::env::var("GE_IMPORT")
            && let Ok(bytes) = std::fs::read(&path)
        {
            let name = std::path::Path::new(&path).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            incoming.0.push((name, bytes));
        }
        let catalog = settings.catalog();
        selected.0 = std::env::var("GE_EXERCISE")
            .ok()
            .or_else(|| catalog.exercises().nth(var("GE_LEVEL", 0)).map(|e| e.key.clone()))
            .unwrap_or_default();
        // A window in the background is throttled by the compositor with vsync.
        window.present_mode = bevy::window::PresentMode::AutoNoVsync;
        let size = std::env::var("GE_SIZE").unwrap_or_default();
        if let Some((w, h)) = size.split_once('x').and_then(|(w, h)| Some((w.parse::<u32>().ok()?, h.parse::<u32>().ok()?))) {
            window.resolution = WindowResolution::new(w, h);
        }
        settings.microphone = false;
        settings.help_seen = true;
        metronome.sound = var("GE_SOUND", 0) == 1;
    }

    #[allow(clippy::too_many_arguments)]
    fn run(
        mut frame: Local<u32>,
        mut sent: Local<u32>,
        mut done_at: Local<Option<f32>>,
        mut shot: Local<Option<u32>>,
        time: Res<Time<Real>>,
        snapshot: Res<Snapshot>,
        expected: Res<Expected>,
        mut next: ResMut<NextState<Screen>>,
        mut heard: MessageWriter<NoteHeard>,
        mut tapped: MessageWriter<FretTapped>,
        mut pointer: MessageWriter<PointerInput>,
        window: Single<(Entity, &Window), With<PrimaryWindow>>,
        mut commands: Commands,
        mut exit: MessageWriter<AppExit>,
    ) {
        *frame += 1;
        if *frame == 5 || (snapshot.rebuild && *frame == 34) {
            next.set(snapshot.screen);
        }
        let start = 30;
        // The pointer over the middle, then the wheel one line at a time.
        let (window_entity, window) = *window;
        if snapshot.scroll > 0.0 && *frame >= start - 10 && *frame < start {
            let target = WindowRef::Primary.normalize(Some(window_entity)).map(NormalizedRenderTarget::Window);
            if let Some(target) = target {
                let location = Location { target, position: Vec2::new(window.width() / 2.0, window.height() / 2.0) };
                let action = if *frame == start - 10 {
                    PointerAction::Move { delta: Vec2::ZERO }
                } else {
                    PointerAction::Scroll { unit: MouseScrollUnit::Line, x: 0.0, y: -snapshot.scroll / 9.0, phase: TouchPhase::Moved }
                };
                pointer.write(PointerInput::new(PointerId::Mouse, location, action));
            }
        }
        let total = snapshot.play + snapshot.tap + snapshot.miss;
        // One answer every 8 frames; the note games ignore the answers while they show the
        // previous one.
        if *frame > start && (*frame - start).is_multiple_of(8) && *sent < total {
            let i = *sent;
            if i < snapshot.play {
                if let Some(midi) = expected.midi {
                    heard.write(NoteHeard { midi, cents: 0.0 });
                }
            } else if i < snapshot.play + snapshot.tap {
                if let Some(position) = expected.position {
                    tapped.write(FretTapped(position));
                }
            } else if let Some(midi) = expected.midi {
                heard.write(NoteHeard { midi: midi + 1, cents: 0.0 });
            }
            *sent += 1;
        }
        if *frame > start && *sent >= total && done_at.is_none() {
            *done_at = Some(time.elapsed_secs());
        }
        if let Some(at) = *done_at
            && shot.is_none()
            && time.elapsed_secs() - at >= snapshot.wait + 0.5
        {
            commands.spawn(Screenshot::primary_window()).observe(save_to_disk(snapshot.path.clone()));
            *shot = Some(*frame);
        }
        if shot.is_some_and(|f| *frame > f + 20) {
            exit.write(AppExit::Success);
        }
    }
}
