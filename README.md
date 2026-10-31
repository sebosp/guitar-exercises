# Guitar Exercises

Finger drills and note reading for the guitar and the bass, made with [Bevy](https://bevyengine.org)
0.20. It listens to the instrument through the microphone to check what is played, or the notes can
be tapped on the fretboard on the screen. It runs on Linux, Windows, macOS, Android and in the
browser: <https://sebosp.github.io/guitar-exercises/>.

## The games

### Finger drills

The four fingers of the fretting hand (1 index, 2 middle, 3 ring, 4 pinky) in every order, up the
strings, one fret higher, and back down in the reverse order. The tablature shows the notes coming,
the fretboard the four notes of the current shape with the finger numbers and colors.

The levels are grouped by hand shape, from the easiest:

| Group | Shape                          | Frets of the fingers                                   |
| ----- | ------------------------------ | ------------------------------------------------------ |
| 1     | One finger per fret            | 1 2 3 4                                                |
| 2     | Index stretch                  | 1 3 4 5 (nothing on fret 2)                            |
| 3     | Pinky stretch                  | 1 2 3 5                                                |
| 4     | Middle-ring stretch            | 1 2 4 5                                                |
| 5     | Index anchor, next string      | index on a string, the others on the next one          |
| 6     | Index anchor, skip a string    | index on a string, the others two strings higher       |
| 7     | Wide span                      | 5 7 9 10, higher on the neck where the frets are close |
| 8     | Index anchor, skip two strings | index on a string, the others three strings higher     |

Each group has the 6 orders of the fingers that are not rotations of each other (1-2-3-4, 2-3-4-1,
3-4-1-2 and 4-1-2-3 loop into the same pattern), 1-2-3-4 and 4-3-2-1 first. A small diagram shows
each shape: the frets, and a dot of the finger's color on each one played (the fingers have the
same colors everywhere, in every theme). The accuracy and the speed (notes per minute) are recorded. A pause
longer than the hint delay names the finger, the fret and the string, and the note pulses. Next to
the tablature the current note is on the staff with its name, and a tuner shows what the
microphone hears.

With the metronome on (the default), each drill has its own tempo and a target tempo, 60 and 100
BPM at first, both changed with - and + in the drill, with 1 to 4 notes per beat. A round is clean
when 90% of the notes are right and 80% are in time: one slot after the previous note, near the
player's usual place in the slot (a steady delay of the microphone is fine, drifting from the
clicks is not). After a clean round the tempo goes up 5 BPM, up to the target; a clean round at the
target completes the lesson, and the target can be raised. After a round under the goal the tempo
stays, after a second one in a row it goes down 5 BPM. The clicks are short bursts of noise, without
a pitch for the microphone to hear as a note.

### Scales

A scale chosen on the page (the root, the family: major, harmonic minor, melodic minor or
pentatonic, and the mode: Dorian, Phrygian dominant, Lydian augmented...) played three notes per
string (two for the pentatonics) from its root on the lowest string, the fingers of the position
shown. Each row has a run (1 2 3 4 5...) and the sequences in groups of 2 to 6 notes (1 2, 2 3,
3 4... then 1 2 3, 2 3 4...), up and back down (6 5 4, 5 4 3...). The rows play the position on
adjacent strings, then skipping 1 string, then 2 strings (the low E, D and B, then the A, G and
high e). Each scale and mode keeps its own progress.

### Stretches

Growing gaps: the four fingers 2 frets apart on one string. Growing shifts: one finger per fret
moving up the neck by jumps that grow (fret 1, 3, 6...), back down on the next string.

The scales have their own page (from the menu, or the drills page): the runs and sequences, and
the notes of the scale in every order, like the fingers: windows of 2, 3 or 4 notes played as
2-1 (2 1, 3 2, 4 3...), 1-3-2 (1 3 2, 2 4 3...), 1-3-2-4 and the 23 orders of 4 notes.

### Stars, tiers and the extra exercises

A round earns a star for 80% of the notes right, two for 90%, three for all of them; a star opens
the next exercise of the row, three in a row open the next row. The border of a tile is the tier
of the fastest clean round with the metronome, in notes per minute (the tempo times the notes per
beat), with the rarity colors of the games: Basic (gray), Medium (green, 160), Advanced (purple,
240), Legendary (orange, 360). Two buttons: practice the weakest exercise (the fewest stars, then
the lowest accuracy), unlock a level at random.

The extra exercises appear at 30 stars: the scales skipping 3 strings or more (more than four
strings apart), the wider gaps, the bigger shifts.

### Learn notes (staff to fretboard)

A note on the staff (treble clef for the guitar, bass clef for the bass, written one octave above
the sound as usual), to play on the instrument or tap on the fretboard. A new note is introduced
with its name, its line or space with the mnemonic, and every place it is on the fretboard; then it
is asked again and again, with a review of an older note between two questions on it, until it is
answered right 3 times in a row (setting) without a hint. Then the next note comes.

The notes come from the middle of the staff outwards: the lines and spaces first, the ledger lines
later, the sharps and flats last when they are on in the settings. A pause shows the hints step by
step: the letters of the lines and spaces, then the name, then the places on the fretboard.

The mnemonics, from the bottom:

| Clef   | Lines                               | Spaces                       |
| ------ | ----------------------------------- | ---------------------------- |
| Treble | E G B D F, Every Good Boy Does Fine | F A C E, FACE                |
| Bass   | G B D F A, Good Boys Do Fine Always | A C E G, All Cows Eat Grass  |

### Find them all

A note on the staff: tap every place it is on the fretboard. The notes are the ones learned in
"Learn notes" (the first five at least), the ones with mistakes or slow answers more often. A pause
shows the letters of the staff, then the places left one by one.

### Settings and statistics

The theme (dark, light, Catppuccin Mocha and Latte, Tokyo Night, Storm and Day, Gruvbox, Nord,
Dracula), the instrument (guitar 6 to 8 strings, bass 4 to 6), the tuning (standard, drop, half step down,
or each string a semitone up or down, a 5-string bass with its low string in A for one), the frets
(12 by default, up to 24), left-handed, the microphone with its noise gate and a tuner to the open
strings, the hint delay, the drill length, the note names in the drills, the sharps and flats, the
right answers that clear a note. The statistics show every drill played and every note with its
accuracy, its time to answer and the hints.

The window is scaled to fit its size (a big screen or the full screen shows everything bigger, a
phone not under 75%), and zoomed over that with Ctrl (Cmd on macOS) and the mouse wheel, a pinch
of the trackpad or of two fingers on a touch screen, Ctrl+= Ctrl+- Ctrl+0 or the buttons of the
settings: to read it from the instrument a few steps away. F11 (or Cmd+Ctrl+F on macOS) toggles
the full screen; in the browser F11 and Ctrl are the browser's. A setting changed on a screen
keeps its scroll.

The settings and the progress are saved on the device: `~/.config/guitar-exercises/` on Linux,
`~/Library/Application Support/guitar-exercises/` on macOS, `%APPDATA%\guitar-exercises\` on
Windows, the app storage on Android, the `localStorage` of the browser on the web. The progress is
kept per instrument and number of strings (`guitar-6`, `bass-4`...).

## The microphone

The samples of the default input are decimated to about 12 kHz and their pitch is found with YIN;
a pitch held for about 50 ms is a note, a new pluck of the same note counts again. Every note from
A0 (the low string of a 5-string bass tuned a whole tone down) up is heard. A clip-on pickup or a plugged-in audio
interface works best; with a microphone, turn up the noise gate in a noisy room.

- Desktop: the default input device of the system (cpal: ALSA, CoreAudio, WASAPI). On macOS the
  system asks for the microphone for the Terminal the first time.
- Android: the app asks for the `RECORD_AUDIO` permission at the first start.
- Web: the browser asks for the microphone; the echo cancellation, noise suppression and automatic
  gain of the browsers are turned off, they eat the notes. The audio starts after the first click
  or touch on the page.

## Building

```sh
cargo run --release           # native
cargo dev                     # native, bevy as a shared library for fast incremental builds
cargo test
trunk serve                   # the web version, http://127.0.0.1:8080
cd mobile && cargo apk build --release --lib   # Android, needs the SDK, the NDK and cargo-apk
```

Linux needs `libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev`, and `mold` and `clang`
for the linker set in `.cargo/config.toml`.

The GitHub workflows build the web version for GitHub Pages (`pages.yml`, the Pages source must be
"GitHub Actions") and the Linux, Windows, macOS and Android builds (`builds.yml`), attached to a
release for the tags `v*`. The APK is signed with the release key of the repository secrets
`ANDROID_KEYSTORE_BASE64` and `ANDROID_KEYSTORE_PASSWORD`, with a debug key without them.

### Checking the screens without an instrument

`GE_SCREENSHOT=out.png` opens a screen, plays a few notes by itself, saves a screenshot and exits:

```sh
XDG_CONFIG_HOME=/tmp/ge GE_SCREENSHOT=drill.png GE_SCREEN=drill GE_PLAY=6 GE_MISS=1 cargo run
XDG_CONFIG_HOME=/tmp/ge GE_SCREENSHOT=find.png GE_SCREEN=find GE_TAP=1 GE_WAIT=11 GE_SIZE=800x400 cargo run
```

See `src/main.rs` for the variables.

## Plans

See [TODO.md](TODO.md).

## Licenses

The code is under the Apache License 2.0. The fonts are under the SIL Open Font License 1.1: Fira
Sans (Mozilla, the Bevy feathers copy), Barlow Condensed (the Barlow Project Authors, the titles
and the numbers) and Noto Music (Google, the clefs, sharps and flats), see
`assets/fonts`.
