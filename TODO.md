# TODO

Future enhancements, to decide if and when. The first section is what was asked and is not done
yet or not verified, the rest are suggestions.

## Not done or not verified yet

- [ ] Try the microphone on a real guitar and bass on every platform: the noise gate, the onset of
      a re-plucked note, the low B of a 5-string bass, the octave errors of the low strings.
- [ ] Android: run the APK on a phone; check that the `RECORD_AUDIO` request through JNI shows the
      dialog and that cpal's AAudio input starts after it is granted.
- [ ] Web: check the microphone in Firefox, Chrome and Safari (the audio context starts suspended
      until a click) and on a phone browser.
- [ ] macOS: an `.app` bundle with `NSMicrophoneUsageDescription` in its `Info.plist`, the bare
      binary only works from the Terminal.
- [ ] Windows: an icon and a manifest for the executable.

## Drills

- [ ] A metronome (needs the `bevy_audio` feature): a target tempo per level, the speed goal raised
      little by little, the notes checked against the beat (early, late).
- [ ] Positions up the neck: the same level at frets 5, 7, 9, 12, the spider walk all the way up and
      back.
- [ ] More shapes: three fingers per string (scales), the pinky anchor, legato (hammer-ons and
      pull-offs: one pluck, several notes), string skipping with every finger.
- [ ] Alternate picking: down and up strokes shown in the tablature.
- [ ] A free mode: pick the fingers, the frets and the strings.
- [ ] The two-handed view: the picking hand fingers (p i m a) for the bass and classical guitar.

## Notes and reading

- [ ] Accidentals by key: a key signature on the staff, the notes of a scale.
- [ ] Rhythm reading: quarter notes, eighths, rests, to play in time with the metronome.
- [ ] Short melodies: a few bars to sight-read, then whole songs in the public domain.
- [ ] Intervals and chords: two or three notes on the staff, find the shapes.
- [ ] The tablature to the staff the other way: a fret and a string, write the note on the staff
      (drag the note head up and down).
- [ ] 8va for the notes above 3 ledger lines (frets 13 to 24 on the high strings).
- [ ] Spaced repetition across days: the review weights decay with the time since the last answer.
- [ ] Ear training: a note played by the game, find it on the fretboard.

## Interface

- [ ] A light theme and larger text options; the colors of the fingers are already color blind safe.
- [ ] A keyboard and gamepad way to tap the fretboard.
- [ ] Animations: the notes sliding in the tablature, confetti when a level is done.
- [ ] A tuner screen with a needle (the settings have a text one).
- [ ] Custom tunings (any note per string), the capo.
- [ ] A history graph of the accuracy and the speed per day in the statistics.
- [ ] Export and import of the progress (a JSON file), and sync between devices.

## Technical

- [ ] The web build is about 28 MB: try `wasm-opt -Oz`, fewer Bevy features, a smaller font subset.
- [ ] A polyphonic pitch detector for chords.
- [ ] iOS: the same mobile crate with an Xcode project, as claudralingo does.
- [ ] Tests of the screens in CI with the screenshot mode (needs a GPU or a software renderer).
