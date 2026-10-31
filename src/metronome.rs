//! The metronome of the drills: a click on every beat, louder on the first beat of the bar. The
//! clicks are short bursts of noise, without a pitch for the microphone to take for a note.
//!
//! The beats on the screen follow the real time from [`Metronome::restart`]; the clicks are made
//! by the speaker output with the same tempo (cpal on the desktops and Android, the Web Audio API
//! in the browser), started the first time the metronome ticks.
use bevy::prelude::*;

pub const BEATS_PER_BAR: u64 = 4;

/// The metronome, driven by the drill screen.
#[derive(Resource)]
pub struct Metronome {
    /// Ticking.
    pub on: bool,
    pub bpm: u32,
    /// Clicks out loud; off for the screenshots.
    pub sound: bool,
    /// The real time of beat 0, in seconds.
    start: f64,
    /// Bumped by each restart, for the output.
    generation: u32,
}

impl Default for Metronome {
    fn default() -> Self {
        Metronome { on: false, bpm: crate::tempo::START_BPM, sound: true, start: 0.0, generation: 0 }
    }
}

impl Metronome {
    pub fn beat_seconds(&self) -> f32 {
        60.0 / self.bpm.max(1) as f32
    }

    /// Starts again from the first beat of a bar, now.
    pub fn restart(&mut self, now: f64) {
        self.start = now;
        self.generation = self.generation.wrapping_add(1);
    }

    /// The beat at a real time, from 0.
    pub fn beat_at(&self, now: f64) -> u64 {
        ((now - self.start).max(0.0) / self.beat_seconds() as f64) as u64
    }
}

/// The sound output, started once.
#[derive(Resource, Default)]
struct Speaker(Option<output::Output>);

pub struct MetronomePlugin;

impl Plugin for MetronomePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Metronome>().init_resource::<Speaker>().add_systems(Update, drive);
    }
}

fn drive(metronome: Res<Metronome>, mut speaker: ResMut<Speaker>, time: Res<Time<Real>>) {
    let audible = metronome.on && metronome.sound;
    if audible && speaker.0.is_none() {
        speaker.0 = Some(output::Output::start());
    }
    if let Some(output) = speaker.0.as_mut() {
        let bpm = if audible { metronome.bpm } else { 0 };
        output.sync(bpm, metronome.generation, metronome.start, time.elapsed_secs_f64());
    }
}

/// The samples of the clicks: the beats counted at the sample rate, so the clicks are exactly in
/// time whatever the frame rate.
pub struct Clicker {
    rate: f32,
    bpm: u32,
    generation: u32,
    /// Samples to the next beat.
    until_beat: f32,
    beat: u64,
    /// The samples of the click sounding, its length and its loudness.
    click: (u32, u32, f32),
    noise: u32,
    last_noise: f32,
}

impl Clicker {
    pub fn new(rate: f32) -> Self {
        Clicker { rate, bpm: 0, generation: 0, until_beat: 0.0, beat: 0, click: (0, 0, 0.0), noise: 0x1234_5678, last_noise: 0.0 }
    }

    /// The tempo, 0 for silence; a new generation starts the bar now.
    pub fn set(&mut self, bpm: u32, generation: u32) {
        if generation != self.generation || (self.bpm == 0 && bpm > 0) {
            self.until_beat = 0.0;
            self.beat = 0;
        }
        self.bpm = bpm;
        self.generation = generation;
    }

    /// One click alone, the accent or a beat.
    pub fn click(rate: f32, accent: bool) -> Vec<f32> {
        let mut clicker = Clicker::new(rate);
        // A minute between two beats: one click.
        clicker.set(1, 1);
        clicker.beat = if accent { 0 } else { 1 };
        (0..(rate * 0.03) as usize).map(|_| clicker.next_sample()).collect()
    }

    pub fn next_sample(&mut self) -> f32 {
        if self.bpm == 0 {
            return 0.0;
        }
        if self.until_beat <= 0.0 {
            self.until_beat += self.rate * 60.0 / self.bpm as f32;
            let accent = self.beat.is_multiple_of(BEATS_PER_BAR);
            let length = (self.rate * if accent { 0.018 } else { 0.012 }) as u32;
            self.click = (length, length, if accent { 0.7 } else { 0.4 });
            self.beat += 1;
        }
        self.until_beat -= 1.0;
        let (left, length, loudness) = self.click;
        if left == 0 {
            return 0.0;
        }
        self.click.0 -= 1;
        // White noise, its difference (brighter, a tick), fading fast.
        self.noise ^= self.noise << 13;
        self.noise ^= self.noise >> 17;
        self.noise ^= self.noise << 5;
        let white = self.noise as f32 / u32::MAX as f32 * 2.0 - 1.0;
        let tick = (white - self.last_noise) * 0.5;
        self.last_noise = white;
        let age = (length - left) as f32 / self.rate;
        tick * loudness * (-age / 0.003).exp()
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod output {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::time::Duration;

    use bevy::prelude::*;
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use cpal::{FromSample, SampleFormat, SizedSample};

    use super::Clicker;

    #[derive(Default)]
    struct Shared {
        bpm: AtomicU32,
        generation: AtomicU32,
        stop: AtomicBool,
    }

    /// The output stream runs on its own thread, which owns it (not `Send` everywhere).
    pub struct Output {
        shared: Arc<Shared>,
    }

    impl Output {
        pub fn start() -> Self {
            let shared = Arc::new(Shared::default());
            let s = shared.clone();
            let spawned = std::thread::Builder::new().name("metronome".into()).spawn(move || {
                if let Err(err) = run(&s) {
                    warn!("Metronome: {err}");
                }
            });
            if let Err(err) = spawned {
                warn!("Metronome: {err}");
            }
            Output { shared }
        }

        pub fn sync(&mut self, bpm: u32, generation: u32, _start: f64, _now: f64) {
            self.shared.bpm.store(bpm, Ordering::Relaxed);
            self.shared.generation.store(generation, Ordering::Relaxed);
        }
    }

    impl Drop for Output {
        fn drop(&mut self) {
            self.shared.stop.store(true, Ordering::Relaxed);
        }
    }

    fn run(shared: &Arc<Shared>) -> Result<(), String> {
        let host = cpal::default_host();
        let device = host.default_output_device().ok_or("no speaker found")?;
        let config = device.default_output_config().map_err(|e| e.to_string())?;
        let channels = config.channels() as usize;
        let rate = config.sample_rate() as f32;
        let stream_config = config.config();
        let stream = match config.sample_format() {
            SampleFormat::F32 => build::<f32>(&device, &stream_config, channels, rate, shared),
            SampleFormat::I16 => build::<i16>(&device, &stream_config, channels, rate, shared),
            SampleFormat::U16 => build::<u16>(&device, &stream_config, channels, rate, shared),
            SampleFormat::I32 => build::<i32>(&device, &stream_config, channels, rate, shared),
            SampleFormat::U8 => build::<u8>(&device, &stream_config, channels, rate, shared),
            other => Err(format!("unsupported sample format {other}")),
        }?;
        stream.play().map_err(|e| e.to_string())?;
        while !shared.stop.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_millis(50));
        }
        Ok(())
    }

    fn build<T>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        channels: usize,
        rate: f32,
        shared: &Arc<Shared>,
    ) -> Result<cpal::Stream, String>
    where
        T: SizedSample + FromSample<f32> + Send + 'static,
    {
        let shared = shared.clone();
        let mut clicker = Clicker::new(rate);
        device
            .build_output_stream(
                config,
                move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                    clicker.set(shared.bpm.load(Ordering::Relaxed), shared.generation.load(Ordering::Relaxed));
                    for frame in data.chunks_mut(channels.max(1)) {
                        let sample = T::from_sample(clicker.next_sample());
                        frame.iter_mut().for_each(|s| *s = sample);
                    }
                },
                |err| warn!("Metronome: {err}"),
                None,
            )
            .map_err(|e| e.to_string())
    }
}

/// The clicks are rendered once into two buffers (the accent and the beat), and scheduled a
/// moment ahead on the clock of the audio context.
#[cfg(target_arch = "wasm32")]
mod output {
    use std::cell::RefCell;

    use web_sys::{AudioBuffer, AudioContext, AudioContextState};

    use super::{BEATS_PER_BAR, Clicker};

    /// The seconds scheduled ahead of the audio clock.
    const AHEAD: f64 = 0.2;

    struct WebClicks {
        context: AudioContext,
        clicks: Vec<AudioBuffer>,
        /// The tempo and the generation scheduled, and the next beat to schedule.
        scheduled: (u32, u32, u64),
        last_resume: f64,
    }

    thread_local! {
        static CLICKS: RefCell<Option<WebClicks>> = const { RefCell::new(None) };
    }

    /// The browser objects stay in a thread local of the page, the resource is a token.
    pub struct Output;

    impl Output {
        pub fn start() -> Self {
            let clicks = AudioContext::new().ok().map(|context| {
                let clicks = render(&context);
                WebClicks { context, clicks, scheduled: (0, 0, 0), last_resume: 0.0 }
            });
            CLICKS.set(clicks);
            Output
        }

        pub fn sync(&mut self, bpm: u32, generation: u32, start: f64, now: f64) {
            CLICKS.with_borrow_mut(|web| {
                if let Some(web) = web.as_mut() {
                    web.sync(bpm, generation, start, now);
                }
            });
        }
    }

    impl WebClicks {
        fn sync(&mut self, bpm: u32, generation: u32, start: f64, now: f64) {
            if bpm == 0 || self.clicks.len() < 2 {
                self.scheduled = (0, generation, 0);
                return;
            }
            let context = &self.context;
            // The browsers start the audio suspended until the page is clicked.
            if context.state() == AudioContextState::Suspended {
                let wall = js_sys::Date::now();
                if wall - self.last_resume > 1000.0 {
                    self.last_resume = wall;
                    let _ = context.resume();
                }
                return;
            }
            let beat = 60.0 / bpm as f64;
            let audio_now = context.current_time();
            let elapsed = now - start;
            if (bpm, generation) != (self.scheduled.0, self.scheduled.1) {
                // From the next beat on.
                self.scheduled = (bpm, generation, (elapsed.max(0.0) / beat).ceil() as u64);
            }
            loop {
                let next = self.scheduled.2;
                let at = audio_now + (next as f64 * beat - elapsed);
                if at > audio_now + AHEAD {
                    break;
                }
                if at >= audio_now - 0.01 {
                    let buffer = &self.clicks[if next.is_multiple_of(BEATS_PER_BAR) { 0 } else { 1 }];
                    play(context, buffer, at.max(audio_now));
                }
                self.scheduled.2 += 1;
            }
        }
    }

    fn render(context: &AudioContext) -> Vec<AudioBuffer> {
        let rate = context.sample_rate();
        [true, false]
            .into_iter()
            .filter_map(|accent| {
                let samples = Clicker::click(rate, accent);
                let buffer = context.create_buffer(1, samples.len() as u32, rate).ok()?;
                buffer.copy_to_channel(&samples, 0).ok()?;
                Some(buffer)
            })
            .collect()
    }

    fn play(context: &AudioContext, buffer: &AudioBuffer, at: f64) {
        let Ok(source) = context.create_buffer_source() else {
            return;
        };
        source.set_buffer(Some(buffer));
        if source.connect_with_audio_node(&context.destination()).is_ok() {
            let _ = source.start_with_when(at);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clicks_on_the_beats() {
        let rate = 8000.0;
        let mut clicker = Clicker::new(rate);
        clicker.set(120, 1);
        let samples: Vec<f32> = (0..(rate as usize * 2)).map(|_| clicker.next_sample()).collect();
        // A click starts every half second, the first one now and louder.
        let loud = |range: std::ops::Range<usize>| samples[range].iter().map(|s| s.abs()).fold(0.0f32, f32::max);
        for beat in 0..4 {
            let at = beat * 4000;
            assert!(loud(at..at + 40) > 0.05, "beat {beat}");
            assert!(loud(at + 400..at + 3900) < 1e-6, "silence after beat {beat}");
        }
        let energy = |range: std::ops::Range<usize>| samples[range].iter().map(|s| s * s).sum::<f32>();
        assert!(energy(0..200) > energy(4000..4200));
        // Silence at 0 BPM, and a restart clicks at once.
        clicker.set(0, 1);
        assert!((0..1000).all(|_| clicker.next_sample() == 0.0));
        clicker.set(60, 2);
        let restart: Vec<f32> = (0..40).map(|_| clicker.next_sample()).collect();
        assert!(restart.iter().any(|s| s.abs() > 0.05));
    }

    #[test]
    fn beats() {
        let mut m = Metronome { bpm: 120, ..default() };
        m.restart(10.0);
        assert_eq!(m.beat_at(9.0), 0);
        assert_eq!(m.beat_at(10.2), 0);
        assert_eq!(m.beat_at(11.6), 3);
    }
}
