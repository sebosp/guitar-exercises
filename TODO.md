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
- [ ] Since the scales page, the zoom and the full screen (2026-10-31): the macOS build (Cmd zoom,
      Cmd+Ctrl+F; only built by the GitHub workflow, no macOS SDK on the desktops), the web page in a
      real browser (built with trunk, headless Firefox does not draw the canvas), the two-finger
      pinch on an Android phone.
- [ ] macOS: an `.app` bundle with `NSMicrophoneUsageDescription` in its `Info.plist`, the bare
      binary only works from the Terminal.
- [ ] Windows: an icon and a manifest for the executable.
- [ ] The metronome through a speaker with the microphone on: that the clicks (noise bursts) are
      never heard as notes and do not break the onsets of a ringing note; the clicks in the
      browser (Web Audio, scheduled ahead) and on Android (cpal AAudio output).
- [ ] The scale fingerings with real hands, the wide ones of the harmonic minor modes; the repeated
      notes of the sequences (1 2, 2 3) heard as two plucks by the microphone.
- [ ] The timing check of the drills with real playing: the tolerance (20% of a slot, 60 to 120 ms)
      and the 80% goal, with the microphone delay and with taps.

## Songs

- [ ] Choose any range of bars, not only the sections.
- [ ] The repeats and the endings of the score, the tempo changes inside a section, the fingering
      of the file when it has one, a play-along sound of the score.
- [ ] An open dialog on Android (the songs folder only for now); the drop of a file in a real
      browser, not tested yet.

## Drills

- [ ] The metronome: show each note early or late against the beat, a count-in bar, accents per
      group of notes per beat.
- [ ] Positions up the neck: the same level at frets 5, 7, 9, 12, the spider walk all the way up and
      back; the scales from each degree (the 7 positions of three notes per string).
- [ ] The scales: the box (CAGED) fingerings, intervals (thirds, fourths), the arpeggios.
- [ ] More shapes: the pinky anchor, legato (hammer-ons and
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

- [ ] Larger text options; the colors of the fingers are already color blind safe.
- [ ] A keyboard and gamepad way to tap the fretboard.
- [ ] Animations: the notes sliding in the tablature, confetti when a level is done.
- [ ] A tuner screen of its own (the settings and the drills have one).
- [ ] The capo.
- [ ] A history graph of the accuracy and the speed per day in the statistics.
- [ ] Export and import of the progress (a JSON file), and sync between devices.

## Technical

- [ ] The web build is about 28 MB: try `wasm-opt -Oz`, fewer Bevy features, a smaller font subset.
- [ ] A polyphonic pitch detector for chords.
- [ ] iOS: the same mobile crate with an Xcode project, as claudralingo does.
- [ ] Tests of the screens in CI with the screenshot mode (needs a GPU or a software renderer).
