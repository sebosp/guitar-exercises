//! The microphone: the samples of the default input device (cpal on the desktops and Android,
//! the Web Audio API in the browser), their pitch, and the notes played as [`NoteHeard`].
//!
//! The capture starts when the microphone is on in the settings. On Android the app asks for the
//! `RECORD_AUDIO` permission first; in the browser the page asks for the microphone and the audio
//! starts with the first click or touch.
use bevy::prelude::*;

use crate::pitch::{self, NoteTracker};
use crate::settings::Settings;

/// The samples analyzed at a time, 85 ms at 48 kHz: two periods of B0 and some.
const WINDOW: usize = 4096;

#[derive(Debug, Clone, PartialEq, Default)]
pub enum MicStatus {
    #[default]
    Off,
    Starting,
    Listening,
    /// The permission was refused, or not given yet.
    NoPermission,
    Failed(String),
}

impl MicStatus {
    pub fn describe(&self) -> String {
        match self {
            MicStatus::Off => "Microphone off".into(),
            MicStatus::Starting => "Starting the microphone...".into(),
            MicStatus::Listening => "Listening".into(),
            MicStatus::NoPermission => "Allow the microphone to play the notes".into(),
            MicStatus::Failed(err) => format!("Microphone: {err}"),
        }
    }
}

/// What the microphone hears now.
#[derive(Resource, Default)]
pub struct Mic {
    pub status: MicStatus,
    /// The RMS level of the last window.
    pub level: f32,
    /// The pitch heard, as a fractional MIDI number.
    pub pitch: Option<f32>,
    /// The note sounding, once stable.
    pub note: Option<i32>,
}

/// A note played on the instrument.
#[derive(Message, Debug, Clone, Copy)]
pub struct NoteHeard {
    pub midi: i32,
    pub cents: f32,
}

#[derive(Resource)]
struct Listener {
    capture: Option<capture::Capture>,
    tracker: NoteTracker,
    samples: Vec<f32>,
}

pub struct AudioInputPlugin;

impl Plugin for AudioInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Mic>()
            .add_message::<NoteHeard>()
            .insert_resource(Listener { capture: None, tracker: NoteTracker::new(0.006), samples: vec![0.0; WINDOW] })
            .add_systems(Update, (start_stop, listen).chain());
    }
}

fn start_stop(settings: Res<Settings>, mut listener: ResMut<Listener>, mut mic: ResMut<Mic>) {
    listener.tracker.gate = settings.gate_level();
    match (settings.microphone, listener.capture.is_some()) {
        (true, false) => {
            listener.capture = Some(capture::Capture::start());
            mic.status = MicStatus::Starting;
        }
        (false, true) => {
            if let Some(capture) = listener.capture.take() {
                capture.stop();
            }
            *mic = Mic::default();
        }
        _ => {}
    }
}

fn listen(mut listener: ResMut<Listener>, mut mic: ResMut<Mic>, mut heard: MessageWriter<NoteHeard>) {
    let Listener { capture, tracker, samples } = &mut *listener;
    let Some(capture) = capture.as_mut() else {
        return;
    };
    let status = capture.status();
    if mic.status != status {
        mic.status = status;
    }
    let Some(rate) = capture.latest(samples) else {
        return;
    };
    let (decimated, rate) = pitch::decimate(samples, rate);
    let level = pitch::rms(&decimated);
    let found = (level >= tracker.gate)
        .then(|| pitch::yin(&decimated, rate, pitch::MIN_HZ, pitch::MAX_HZ))
        .flatten()
        .map(|p| crate::music::hz_to_midi(p.hz));
    if let Some(onset) = tracker.feed(found, level) {
        heard.write(NoteHeard { midi: onset.midi, cents: onset.cents });
    }
    mic.level = level;
    mic.pitch = found;
    mic.note = tracker.current();
}

#[cfg(not(target_arch = "wasm32"))]
mod capture {
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use bevy::prelude::*;
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use cpal::{FromSample, Sample, SampleFormat, SizedSample};

    use super::{MicStatus, WINDOW};

    #[derive(Default)]
    struct Shared {
        ring: Mutex<Ring>,
        status: Mutex<MicStatus>,
    }

    #[derive(Default)]
    struct Ring {
        samples: VecDeque<f32>,
        rate: u32,
        /// The samples received since the last analysis.
        fresh: usize,
    }

    /// The capture runs on its own thread, which owns the cpal stream (not `Send` everywhere).
    pub struct Capture {
        shared: Arc<Shared>,
        stop: Arc<AtomicBool>,
    }

    impl Capture {
        pub fn start() -> Self {
            let shared = Arc::new(Shared::default());
            let stop = Arc::new(AtomicBool::new(false));
            *shared.status.lock().unwrap() = MicStatus::Starting;
            let (s, st) = (shared.clone(), stop.clone());
            let spawned = std::thread::Builder::new().name("microphone".into()).spawn(move || {
                if let Err(err) = run(&s, &st) {
                    warn!("Microphone: {err}");
                    *s.status.lock().unwrap() = MicStatus::Failed(err);
                }
            });
            if let Err(err) = spawned {
                *shared.status.lock().unwrap() = MicStatus::Failed(err.to_string());
            }
            Capture { shared, stop }
        }

        pub fn stop(self) {
            self.stop.store(true, Ordering::Relaxed);
        }

        pub fn status(&self) -> MicStatus {
            self.shared.status.lock().unwrap().clone()
        }

        /// The last [`WINDOW`] samples and their rate, when enough new ones came in.
        pub fn latest(&mut self, out: &mut [f32]) -> Option<f32> {
            let mut ring = self.shared.ring.lock().unwrap();
            // About 100 analyses per second at most.
            if ring.samples.len() < out.len() || ring.fresh < (ring.rate as usize / 100).max(1) {
                return None;
            }
            ring.fresh = 0;
            let start = ring.samples.len() - out.len();
            for (o, s) in out.iter_mut().zip(ring.samples.range(start..)) {
                *o = *s;
            }
            Some(ring.rate as f32)
        }
    }

    impl Drop for Capture {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::Relaxed);
        }
    }

    fn run(shared: &Arc<Shared>, stop: &AtomicBool) -> Result<(), String> {
        #[cfg(target_os = "android")]
        if !super::android::wait_for_permission(stop, || *shared.status.lock().unwrap() = MicStatus::NoPermission) {
            return Ok(());
        }
        let host = cpal::default_host();
        let device = host.default_input_device().ok_or("no microphone found")?;
        let config = device.default_input_config().map_err(|e| e.to_string())?;
        let channels = config.channels() as usize;
        shared.ring.lock().unwrap().rate = config.sample_rate();
        let stream_config = config.config();
        let stream = match config.sample_format() {
            SampleFormat::F32 => build::<f32>(&device, &stream_config, channels, shared),
            SampleFormat::I16 => build::<i16>(&device, &stream_config, channels, shared),
            SampleFormat::U16 => build::<u16>(&device, &stream_config, channels, shared),
            SampleFormat::I32 => build::<i32>(&device, &stream_config, channels, shared),
            SampleFormat::U8 => build::<u8>(&device, &stream_config, channels, shared),
            other => Err(format!("unsupported sample format {other}")),
        }?;
        stream.play().map_err(|e| e.to_string())?;
        *shared.status.lock().unwrap() = MicStatus::Listening;
        while !stop.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_millis(50));
        }
        Ok(())
    }

    fn build<T>(device: &cpal::Device, config: &cpal::StreamConfig, channels: usize, shared: &Arc<Shared>) -> Result<cpal::Stream, String>
    where
        T: SizedSample + Send + 'static,
        f32: FromSample<T>,
    {
        let data_shared = shared.clone();
        let error_shared = shared.clone();
        device
            .build_input_stream(
                config,
                move |data: &[T], _: &cpal::InputCallbackInfo| {
                    let mut ring = data_shared.ring.lock().unwrap();
                    for frame in data.chunks(channels.max(1)) {
                        let mono = frame.iter().map(|&s| f32::from_sample(s)).sum::<f32>() / frame.len() as f32;
                        ring.samples.push_back(mono);
                        ring.fresh += 1;
                    }
                    let excess = ring.samples.len().saturating_sub(2 * WINDOW);
                    ring.samples.drain(..excess);
                },
                move |err| {
                    *error_shared.status.lock().unwrap() = MicStatus::Failed(err.to_string());
                },
                None,
            )
            .map_err(|e| e.to_string())
    }
}

/// The `RECORD_AUDIO` runtime permission through JNI: Android 6 and later ask the player.
#[cfg(target_os = "android")]
mod android {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    use jni::JavaVM;
    use jni::objects::{JObject, JValue};

    const PERMISSION: &str = "android.permission.RECORD_AUDIO";

    fn with_activity<R>(f: impl FnOnce(&mut jni::JNIEnv, &JObject) -> jni::errors::Result<R>) -> Option<R> {
        let app = bevy::android::ANDROID_APP.get()?;
        // SAFETY: the pointers of the running activity, valid while the app runs.
        let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr() as *mut jni::sys::JavaVM) }.ok()?;
        let mut env = vm.attach_current_thread().ok()?;
        let activity = unsafe { JObject::from_raw(app.activity_as_ptr() as jni::sys::jobject) };
        f(&mut env, &activity).ok()
    }

    fn granted() -> bool {
        with_activity(|env, activity| {
            let permission = env.new_string(PERMISSION)?;
            let result = env.call_method(activity, "checkSelfPermission", "(Ljava/lang/String;)I", &[JValue::Object(&permission)])?;
            // PackageManager.PERMISSION_GRANTED
            Ok(result.i()? == 0)
        })
        .unwrap_or(false)
    }

    fn request() {
        with_activity(|env, activity| {
            let permission = env.new_string(PERMISSION)?;
            let permissions = env.new_object_array(1, "java/lang/String", &permission)?;
            env.call_method(activity, "requestPermissions", "([Ljava/lang/String;I)V", &[JValue::Object(&permissions), JValue::Int(1)])?;
            Ok(())
        });
    }

    /// Asks once and waits for the answer; false when stopped before it was granted.
    pub fn wait_for_permission(stop: &AtomicBool, waiting: impl Fn()) -> bool {
        let mut asked = false;
        while !stop.load(Ordering::Relaxed) {
            if granted() {
                return true;
            }
            if !asked {
                request();
                asked = true;
            }
            waiting();
            std::thread::sleep(Duration::from_millis(500));
        }
        false
    }
}

#[cfg(target_arch = "wasm32")]
mod capture {
    use std::cell::RefCell;

    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{AnalyserNode, AudioContext, AudioContextState, MediaStream};

    use super::{MicStatus, WINDOW};

    struct WebMic {
        context: AudioContext,
        analyser: AnalyserNode,
        stream: MediaStream,
        last_resume: f64,
    }

    thread_local! {
        static MIC: RefCell<Option<WebMic>> = const { RefCell::new(None) };
        static STATUS: RefCell<MicStatus> = const { RefCell::new(MicStatus::Off) };
        /// Bumped by each start and stop: a late answer of the browser is dropped.
        static GENERATION: RefCell<u32> = const { RefCell::new(0) };
    }

    /// The browser objects stay in the thread locals of the page, the resource is a token.
    pub struct Capture;

    impl Capture {
        pub fn start() -> Self {
            let generation = GENERATION.with_borrow_mut(|g| {
                *g += 1;
                *g
            });
            STATUS.set(MicStatus::Starting);
            wasm_bindgen_futures::spawn_local(async move {
                let result = open().await;
                if GENERATION.with_borrow(|g| *g) != generation {
                    if let Ok(mic) = result {
                        close(mic);
                    }
                    return;
                }
                match result {
                    Ok(mic) => {
                        MIC.set(Some(mic));
                        STATUS.set(MicStatus::Listening);
                    }
                    Err(err) => {
                        let name = err.dyn_ref::<web_sys::DomException>().map(|e| e.name());
                        let status = match name.as_deref() {
                            Some("NotAllowedError") | Some("SecurityError") => MicStatus::NoPermission,
                            Some(name) => MicStatus::Failed(name.to_string()),
                            None => MicStatus::Failed(format!("{err:?}")),
                        };
                        STATUS.set(status);
                    }
                }
            });
            Capture
        }

        pub fn stop(self) {
            GENERATION.with_borrow_mut(|g| *g += 1);
            if let Some(mic) = MIC.take() {
                close(mic);
            }
            STATUS.set(MicStatus::Off);
        }

        pub fn status(&self) -> MicStatus {
            STATUS.with_borrow(|s| s.clone())
        }

        pub fn latest(&mut self, out: &mut [f32]) -> Option<f32> {
            MIC.with_borrow_mut(|mic| {
                let mic = mic.as_mut()?;
                // The browsers start the audio suspended until the page is clicked.
                if mic.context.state() == AudioContextState::Suspended {
                    let now = js_sys::Date::now();
                    if now - mic.last_resume > 1000.0 {
                        mic.last_resume = now;
                        let _ = mic.context.resume();
                    }
                    return None;
                }
                mic.analyser.get_float_time_domain_data(out);
                Some(mic.context.sample_rate())
            })
        }
    }

    fn close(mic: WebMic) {
        let _ = mic.context.close();
        for track in mic.stream.get_tracks() {
            if let Ok(track) = track.dyn_into::<web_sys::MediaStreamTrack>() {
                track.stop();
            }
        }
    }

    async fn open() -> Result<WebMic, JsValue> {
        let window = web_sys::window().ok_or("no window")?;
        let devices = window.navigator().media_devices()?;
        // The raw sound of the instrument: the voice processing of the browsers eats the notes.
        let audio = js_sys::Object::new();
        for key in ["echoCancellation", "noiseSuppression", "autoGainControl"] {
            js_sys::Reflect::set(&audio, &key.into(), &JsValue::FALSE)?;
        }
        let constraints = web_sys::MediaStreamConstraints::new();
        constraints.set_audio(&audio);
        let stream: MediaStream = JsFuture::from(devices.get_user_media_with_constraints(&constraints)?).await?.dyn_into()?;
        let context = AudioContext::new()?;
        let source = context.create_media_stream_source(&stream)?;
        let analyser = context.create_analyser()?;
        analyser.set_fft_size(WINDOW as u32);
        source.connect_with_audio_node(&analyser)?;
        Ok(WebMic { context, analyser, stream, last_resume: 0.0 })
    }
}
