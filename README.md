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

Each group has the 24 orders of the fingers, 1-2-3-4 and 4-3-2-1 first, then the orders with the
smallest jumps between fingers. 90% accuracy completes a level and opens the next order; three
orders done open the next group. The accuracy and the speed (notes per minute) are recorded. A pause
longer than the hint delay names the finger, the fret and the string, and the note pulses.

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

The instrument (guitar 6 to 8 strings, bass 4 to 6), the tuning (standard, drop, half step down),
the frets (12 by default, up to 24), left-handed, the microphone with its noise gate and a tuner to
check it, the hint delay, the drill length, the note names in the drills, the sharps and flats, the
right answers that clear a note. The statistics show every drill played and every note with its
accuracy, its time to answer and the hints.

The settings and the progress are saved on the device: `~/.config/guitar-exercises/` on Linux,
`~/Library/Application Support/guitar-exercises/` on macOS, `%APPDATA%\guitar-exercises\` on
Windows, the app storage on Android, the `localStorage` of the browser on the web. The progress is
kept per instrument and number of strings (`guitar-6`, `bass-4`...).

## The microphone

The samples of the default input are decimated to about 12 kHz and their pitch is found with YIN;
a pitch held for about 50 ms is a note, a new pluck of the same note counts again. Every note from
B0 (the low string of a 5-string bass) up is heard. A clip-on pickup or a plugged-in audio
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
Sans (Mozilla, the Bevy feathers copy) and Noto Music (Google, the clefs, sharps and flats), see
`assets/fonts`.
