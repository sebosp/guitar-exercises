# guitar-exercises

A Bevy 0.20 game: finger drills, scales, songs and note reading for the guitar and the bass,
played at the instrument with the screen a meter away. Linux, Windows, macOS, Android and the web.

## Every session

- Load the `frontend-design:frontend-design` skill before touching any screen, color, font or
  layout, so the game stays a good experience for the players: clear from the instrument,
  accessible (contrast, color blindness, reduced motion), a balanced color scheme. The design
  tokens and the layout decisions are in `docs/design.md`; keep it in step with the screens.
- The research on gamification, reinforcement, pleasant palettes and gentle error signals is in
  `reports/Gamified music practice feedback design.md` (notes under `research_notes/`). Read it
  before adding a reward, a counter, a notice or an error state.
- Compile, test and run clippy on corvid-zt (`rsync` the working tree, `cargo check`, `cargo test`,
  `cargo clippy --all-targets`, `cargo fmt`); only light checks on wormgat. Screenshots of a screen
  with the harness in `src/main.rs` (`GE_SCREENSHOT`, `GE_SCREEN`, `GE_EXERCISE`, `GE_PLAY`...).
- No commits until the user asks; the changes are tested on portia first.

## The players' rules

- Treat the players as smart: no legends, explanations or reminders inside an exercise. The
  finger colors are explained once in the help.
- Signal a mistake without aggression: a short direction ("a bit sharp, a little lower"), never
  "wrong"; a change of tempo is a temporary pill, not a permanent sentence.
- Numbers that change during a round keep a fixed width and place.
- The finger colors (`ui::colors::FINGERS`) belong to the fingers only; no middle-dot strings; no
  duplicated titles; no explanations of what the players learn by playing.
