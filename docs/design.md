# Design

The game is played at the instrument, the screen a meter away on a stand: its job is to show the
note to play and how the practice goes, readable from there. The look comes from the instrument.

## Tokens

- Ebony `#16120F`, the background; rosewood `#221B17`, the panels; nickel buttons `#302823`.
- Bone `#EDE5D6`, the text; dust `#A89D8D`, the secondary text.
- Nickel blue `#A9BCC8`, the headings and the values; rosewood red `#8C3B2B`, a choice selected.
- The fingers keep their validated colors in every theme (yellow, aqua, violet, deep blue, see
  `ui::colors::FINGERS`); no other element uses those hues: the tiers are rank pips, the accent is
  nickel, not amber.
- The other themes (light, Catppuccin, Tokyo Night, Gruvbox, Nord, Dracula) keep their own
  palettes; the contrasts are checked by the tests of `theme.rs`.

## Type

- Barlow Condensed SemiBold: the titles, the labels and every number (BPM, stars, the note to
  play). It was drawn for road signs read from afar.
- Fira Sans: the running text.
- The scale, about 1.25 apart: 13, 15, 17, 21, 27, 34, 44 (`ui::size`).

## Layout

- Left aligned. Panels are rounded 10 px, buttons and tiles 6 px, chips are pills.
- The menu is a setlist: the tuning drawn as the strings, then rows with hairlines, each with
  where the player is.
- The drill reads top to bottom as it is used: the note to play, big, and how the round goes;
  the fretboard; then the tablature, the staff and the tuner.
- Plain sentences, no `A · B · C` strings; no explanations of what the players learn by
  playing (the stars, the tiers).
