//! The songs: Guitar Pro files (GP3, GP4, GP5, GP6 `.gpx`, GP7 and later `.gp`) read with the
//! `guitarpro` crate, made into what the drills need: the tracks of strings with their tuning,
//! the measures with their tempo and markers, and the notes of each beat on their strings and
//! frets, with their time in quarter notes. Without Bevy.
//!
//! A section is a run of measures: from a marker of the file to the next one, or 4 measures at a
//! time when the file has no marker, and the whole song. A section to play is a list of steps, a
//! note or a chord each (any note of a chord is right, the microphone hears one note at a time),
//! with the time of each from the start of the section, for the timing with the metronome.
use guitarpro::{Beat, BeatStatus, NoteType, Song};

use crate::drills::Step;
use crate::music::{Instrument, InstrumentKind, Position, Spelled};

/// The measures of a section when the file has no marker, more for a long song: about this many
/// sections at most.
pub const BARS_PER_SECTION: usize = 4;
const MAX_SECTIONS: usize = 24;

#[derive(Debug, Clone, PartialEq)]
pub struct SongData {
    pub title: String,
    pub artist: String,
    pub tracks: Vec<TrackData>,
    pub measures: Vec<MeasureData>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasureData {
    /// The start, in quarter notes from the start of the song.
    pub start: f32,
    pub length: f32,
    pub tempo: f32,
    pub marker: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrackData {
    pub name: String,
    /// The open strings, from the lowest to the highest.
    pub open: Vec<i32>,
    pub frets: u8,
    /// The beats with notes, in order.
    pub beats: Vec<BeatData>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BeatData {
    pub measure: usize,
    /// In quarter notes from the start of the song.
    pub time: f32,
    /// The notes plucked, the lowest string first.
    pub notes: Vec<Position>,
}

/// A run of measures to practice.
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub name: String,
    /// The first measure and the one after the last, from 0.
    pub measures: std::ops::Range<usize>,
}

/// The steps of a section and their times.
#[derive(Debug, Clone, PartialEq)]
pub struct Practice {
    pub instrument: Instrument,
    pub steps: Vec<Step>,
    /// Every note of the chord of each step, as MIDI numbers.
    pub chords: Vec<Vec<i32>>,
    /// The time of each step, in quarter notes from the first one.
    pub times: Vec<f32>,
    /// The tempo of the score at the start of the section.
    pub tempo: f32,
}

/// Reads a file by its extension, or tries every format.
pub fn read(name: &str, bytes: &[u8]) -> Result<SongData, String> {
    let lower = name.to_lowercase();
    let mut song = Song::default();
    let ext = lower.rsplit('.').next().unwrap_or_default();
    let result = match ext {
        "gp3" => song.read_gp3(bytes),
        "gp4" => song.read_gp4(bytes),
        "gp5" => song.read_gp5(bytes),
        "gpx" => song.read_gpx(bytes),
        "gp" => song.read_gp(bytes),
        _ => {
            let mut last = Err(guitarpro::GpError::FormatError("unknown format".into()));
            for read in [Song::read_gp, Song::read_gpx, Song::read_gp5, Song::read_gp4, Song::read_gp3] {
                song = Song::default();
                last = read(&mut song, bytes);
                if last.is_ok() {
                    break;
                }
            }
            last
        }
    };
    result.map_err(|e| format!("{name} is not a Guitar Pro file this game reads ({e})"))?;
    Ok(from_song(&song, name))
}

/// The duration of a beat in quarter notes.
fn quarters(beat: &Beat) -> f32 {
    let d = &beat.duration;
    let mut q = 4.0 / d.value.max(1) as f32;
    if d.dotted {
        q *= 1.5;
    } else if d.double_dotted {
        q *= 1.75;
    }
    if d.tuplet_enters > 0 && d.tuplet_times > 0 {
        q *= d.tuplet_times as f32 / d.tuplet_enters as f32;
    }
    q
}

pub fn from_song(song: &Song, file_name: &str) -> SongData {
    let default_tempo = if song.tempo > 0 { song.tempo as f32 } else { 120.0 };
    let mut measures = Vec::with_capacity(song.measure_headers.len());
    let mut start = 0.0;
    let mut tempo = default_tempo;
    for header in &song.measure_headers {
        let ts = &header.time_signature;
        let length = ts.numerator.max(1) as f32 * 4.0 / ts.denominator.value.max(1) as f32;
        if header.tempo > 0 {
            tempo = header.tempo as f32;
        }
        let marker = header.marker.as_ref().map(|m| m.title.trim().to_string()).filter(|t| !t.is_empty());
        measures.push(MeasureData { start, length, tempo, marker });
        start += length;
    }
    let tracks = song
        .tracks
        .iter()
        .filter(|t| !t.percussion_track && !t.strings.is_empty())
        .map(|track| {
            // The strings are numbered from the highest, 1.
            let mut strings: Vec<(i8, i8)> = track.strings.clone();
            strings.sort_by_key(|(number, _)| std::cmp::Reverse(*number));
            let open: Vec<i32> = strings.iter().map(|(_, tuning)| *tuning as i32).collect();
            let count = open.len();
            let mut beats = Vec::new();
            for (index, measure) in track.measures.iter().enumerate() {
                let Some(info) = measures.get(index) else { break };
                // The first voice with notes.
                let Some(voice) = measure.voices.iter().find(|v| v.beats.iter().any(|b| !b.notes.is_empty())) else {
                    continue;
                };
                let mut time = info.start;
                for beat in &voice.beats {
                    let notes: Vec<Position> = beat
                        .notes
                        .iter()
                        .filter(|n| {
                            n.kind == NoteType::Normal && beat.status != BeatStatus::Rest && n.string >= 1 && (n.string as usize) <= count
                        })
                        .map(|n| Position { string: count - n.string as usize, fret: n.value.clamp(0, 36) as u8 })
                        .collect();
                    if !notes.is_empty() {
                        let mut notes = notes;
                        notes.sort_by_key(|p| p.string);
                        beats.push(BeatData { measure: index, time, notes });
                    }
                    time += quarters(beat);
                }
            }
            let frets =
                beats.iter().flat_map(|b| b.notes.iter().map(|p| p.fret)).max().unwrap_or(12).max(track.fret_count.min(24)).clamp(12, 24);
            TrackData { name: track.name.trim().to_string(), open, frets, beats }
        })
        .filter(|t| !t.beats.is_empty())
        .collect();
    let title = if song.name.trim().is_empty() {
        file_name.rsplit(['/', '\\']).next().unwrap_or(file_name).to_string()
    } else {
        song.name.trim().to_string()
    };
    SongData { title, artist: song.artist.trim().to_string(), tracks, measures }
}

impl SongData {
    /// The sections: from each marker to the next one, or every 4 measures without markers;
    /// the whole song last.
    pub fn sections(&self) -> Vec<Section> {
        let count = self.measures.len();
        let mut sections = Vec::new();
        let marked: Vec<(usize, &str)> =
            self.measures.iter().enumerate().filter_map(|(i, m)| m.marker.as_deref().map(|t| (i, t))).collect();
        if marked.is_empty() {
            let bars = count.div_ceil(MAX_SECTIONS).div_ceil(BARS_PER_SECTION).max(1) * BARS_PER_SECTION;
            for first in (0..count).step_by(bars) {
                let end = (first + bars).min(count);
                sections.push(Section { name: format!("Bars {}-{}", first + 1, end), measures: first..end });
            }
        } else {
            if marked[0].0 > 0 {
                sections.push(Section { name: "Start".to_string(), measures: 0..marked[0].0 });
            }
            for (k, (first, title)) in marked.iter().enumerate() {
                let end = marked.get(k + 1).map_or(count, |(next, _)| *next);
                sections.push(Section { name: title.to_string(), measures: *first..end });
            }
        }
        if sections.len() != 1 {
            sections.push(Section { name: "Whole song".to_string(), measures: 0..count });
        }
        sections
    }

    /// The track for an instrument: the first one with as many strings and a bass range for a
    /// bass, else the first one.
    pub fn track_for(&self, kind: InstrumentKind) -> usize {
        let bass_like = |t: &TrackData| t.open.first().is_some_and(|low| *low < 36);
        self.tracks.iter().position(|t| bass_like(t) == (kind == InstrumentKind::Bass)).unwrap_or(0)
    }

    /// A section of a track to play.
    pub fn practice(&self, track: usize, section: &Section, kind: InstrumentKind) -> Option<Practice> {
        let track = self.tracks.get(track)?;
        let instrument = Instrument { kind, open: track.open.clone(), frets: track.frets };
        let beats: Vec<&BeatData> = track.beats.iter().filter(|b| section.measures.contains(&b.measure)).collect();
        let first_time = beats.first()?.time;
        let mut steps = Vec::new();
        let mut chords = Vec::new();
        let mut times = Vec::new();
        for beat in beats {
            // The highest note of a chord shows where to play; any of them is right.
            let top = *beat.notes.last()?;
            let midi = instrument.midi_at(top);
            steps.push(Step { position: top, finger: 0, note: Some(Spelled::from_midi(midi, false)) });
            chords.push(beat.notes.iter().map(|p| instrument.midi_at(*p)).collect());
            times.push(beat.time - first_time);
        }
        let tempo = self.measures.get(section.measures.start).map_or(120.0, |m| m.tempo);
        Some(Practice { instrument, steps, chords, times, tempo })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use guitarpro::{Measure, MeasureHeader, Note, Track, Voice};

    /// A 4-string bass in standard tuning, 6 measures of 4/4 at 100 BPM: quarter notes on the
    /// open E then A, a marker on the third measure.
    fn bass_song() -> Song {
        let mut song = Song { tempo: 100, ..Default::default() };
        let mut track = Track { name: "Bass".into(), strings: vec![(1, 43), (2, 38), (3, 33), (4, 28)], ..Default::default() };
        for m in 0..6 {
            let mut header = MeasureHeader { tempo: 100, ..Default::default() };
            if m == 2 {
                header.marker = Some(guitarpro::model::legacy::headers::Marker { title: "Verse".into(), color: 0 });
            }
            song.measure_headers.push(header);
            let mut voice = Voice::default();
            for b in 0..4 {
                let mut beat = Beat::default();
                // Fret 0 of the E string (4), then fret 2 of the A string (3), then a chord.
                let note = |string: i8, value: i16| Note { string, value, kind: NoteType::Normal, ..Default::default() };
                beat.notes = match b {
                    0 => vec![note(4, 0)],
                    1 => vec![note(3, 2)],
                    2 => vec![note(4, 3), note(3, 2)],
                    _ => vec![],
                };
                if b == 3 {
                    beat.status = BeatStatus::Rest;
                }
                voice.beats.push(beat);
            }
            track.measures.push(Measure { voices: vec![voice], ..Default::default() });
        }
        song.tracks.push(track);
        song
    }

    #[test]
    fn reads_a_song() {
        let data = from_song(&bass_song(), "song.gp5");
        assert_eq!(data.title, "song.gp5");
        let track = &data.tracks[0];
        assert_eq!(track.open, [28, 33, 38, 43]);
        assert_eq!(track.beats.len(), 6 * 3);
        assert_eq!(track.beats[1].time, 1.0);
        assert_eq!(track.beats[3].time, 4.0);
        // The A string, fret 2: a B.
        assert_eq!(track.beats[1].notes, [Position { string: 1, fret: 2 }]);
        assert_eq!(data.measures[3].start, 12.0);
        assert_eq!(data.track_for(InstrumentKind::Bass), 0);
    }

    #[test]
    fn sections_and_practice() {
        let data = from_song(&bass_song(), "song.gp5");
        let sections = data.sections();
        let names: Vec<&str> = sections.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, ["Start", "Verse", "Whole song"]);
        assert_eq!(sections[1].measures, 2..6);
        let practice = data.practice(0, &sections[1], InstrumentKind::Bass).unwrap();
        assert_eq!(practice.steps.len(), 4 * 3);
        assert_eq!(practice.times[..4], [0.0, 1.0, 2.0, 4.0]);
        assert_eq!(practice.tempo, 100.0);
        // The chord: the highest note shows, the G of the E string fret 3 is right too.
        assert_eq!(practice.steps[2].position, Position { string: 1, fret: 2 });
        assert_eq!(practice.chords[2], [31, 35]);
        // Without markers: 4 bars at a time, then the whole song.
        let mut plain = bass_song();
        plain.measure_headers.iter_mut().for_each(|h| h.marker = None);
        let names: Vec<String> = from_song(&plain, "x").sections().into_iter().map(|s| s.name).collect();
        assert_eq!(names, ["Bars 1-4", "Bars 5-6", "Whole song"]);
        // A long song: 24 sections at most, by 4 bars.
        let mut long = plain.clone();
        for _ in 0..200 {
            long.measure_headers.push(MeasureHeader::default());
        }
        let sections = from_song(&long, "x").sections();
        assert!(sections.len() <= 25, "{}", sections.len());
        assert_eq!(sections[0].measures.len() % 4, 0);
    }

    /// The Guitar Pro files of `GE_SONGS` (a folder) read and give sections to play.
    #[test]
    #[ignore]
    fn reads_real_files() {
        let Ok(dir) = std::env::var("GE_SONGS") else { return };
        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else { continue };
            if !name.to_lowercase().contains(".gp") {
                continue;
            }
            let bytes = std::fs::read(&path).unwrap();
            match read(name, &bytes) {
                Ok(song) => {
                    let sections = song.sections();
                    let track = song.track_for(InstrumentKind::Bass);
                    let practice = sections.last().and_then(|s| song.practice(track, s, InstrumentKind::Bass));
                    println!(
                        "{name}: {:?} by {:?}, {} tracks {:?}, {} measures, {} sections, {} notes at {} BPM",
                        song.title,
                        song.artist,
                        song.tracks.len(),
                        song.tracks.iter().map(|t| (t.name.clone(), t.open.clone())).collect::<Vec<_>>(),
                        song.measures.len(),
                        sections.len(),
                        practice.as_ref().map_or(0, |p| p.steps.len()),
                        practice.as_ref().map_or(0.0, |p| p.tempo),
                    );
                }
                Err(e) => println!("{e}"),
            }
        }
    }
}
