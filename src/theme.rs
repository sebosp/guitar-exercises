//! The color themes of the interface. The fretboard keeps its wood and the fingers keep their
//! colors in every theme, so a finger looks the same everywhere.
use std::sync::atomic::{AtomicU8, Ordering};

use bevy::prelude::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum Theme {
    #[default]
    Dark,
    Light,
    CatppuccinMocha,
    CatppuccinLatte,
    TokyoNight,
    TokyoNightStorm,
    TokyoNightDay,
    GruvboxDark,
    Nord,
    Dracula,
}

/// The colors of a theme, see [`crate::ui::colors`] for their use.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub bg: Color,
    pub panel: Color,
    pub button: Color,
    pub button_hover: Color,
    pub selected: Color,
    pub text: Color,
    pub dim: Color,
    pub accent: Color,
    pub ok: Color,
    pub bad: Color,
    pub reveal: Color,
    pub locked: Color,
}

fn hex(rgb: u32) -> Color {
    Color::srgb_u8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

impl Theme {
    pub const ALL: [Theme; 10] = [
        Theme::Dark,
        Theme::Light,
        Theme::CatppuccinMocha,
        Theme::CatppuccinLatte,
        Theme::TokyoNight,
        Theme::TokyoNightStorm,
        Theme::TokyoNightDay,
        Theme::GruvboxDark,
        Theme::Nord,
        Theme::Dracula,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Theme::Dark => "Dark",
            Theme::Light => "Light",
            Theme::CatppuccinMocha => "Catppuccin Mocha",
            Theme::CatppuccinLatte => "Catppuccin Latte",
            Theme::TokyoNight => "Tokyo Night",
            Theme::TokyoNightStorm => "Tokyo Night Storm",
            Theme::TokyoNightDay => "Tokyo Night Day",
            Theme::GruvboxDark => "Gruvbox Dark",
            Theme::Nord => "Nord",
            Theme::Dracula => "Dracula",
        }
    }

    pub fn palette(self) -> Palette {
        let p = |bg, panel, button, button_hover, selected, text, dim, accent, ok, bad, reveal, locked| Palette {
            bg: hex(bg),
            panel: hex(panel),
            button: hex(button),
            button_hover: hex(button_hover),
            selected: hex(selected),
            text: hex(text),
            dim: hex(dim),
            accent: hex(accent),
            ok: hex(ok),
            bad: hex(bad),
            reveal: hex(reveal),
            locked: hex(locked),
        };
        match self {
            Theme::Dark => {
                p(0x1c1f24, 0x292b36, 0x383d4a, 0x4a5163, 0xa87321, 0xebe8e3, 0x99a1a8, 0xf2ad40, 0x40ba80, 0xe6544a, 0x4d8feb, 0x2b2e33)
            }
            Theme::Light => {
                p(0xf4f2ee, 0xe6e3dd, 0xd6d2ca, 0xc6c1b8, 0xf0c98a, 0x1f1f24, 0x5f646b, 0xb5670a, 0x1a8a52, 0xc8322b, 0x2563c9, 0xe9e7e3)
            }
            Theme::CatppuccinMocha => {
                p(0x11111b, 0x1e1e2e, 0x313244, 0x45475a, 0x7c4fc4, 0xcdd6f4, 0xa6adc8, 0xfab387, 0xa6e3a1, 0xf38ba8, 0x89b4fa, 0x181825)
            }
            Theme::CatppuccinLatte => {
                p(0xeff1f5, 0xe6e9ef, 0xccd0da, 0xbcc0cc, 0xdcc6f5, 0x4c4f69, 0x6c6f85, 0xfe640b, 0x40a02b, 0xd20f39, 0x1e66f5, 0xe6e9ef)
            }
            Theme::TokyoNight => {
                p(0x16161e, 0x1a1b26, 0x292e42, 0x3b4261, 0x3d59a1, 0xc0caf5, 0x9aa5ce, 0xff9e64, 0x9ece6a, 0xf7768e, 0x7aa2f7, 0x1f2335)
            }
            Theme::TokyoNightStorm => {
                p(0x1f2335, 0x24283b, 0x2f354f, 0x3b4261, 0x3d59a1, 0xc0caf5, 0x9aa5ce, 0xff9e64, 0x9ece6a, 0xf7768e, 0x7aa2f7, 0x272b40)
            }
            Theme::TokyoNightDay => {
                p(0xe1e2e7, 0xd4d6e4, 0xc8cce0, 0xb6bfe2, 0xa8b7e8, 0x1f3070, 0x4a5890, 0xb15c00, 0x587539, 0xf52a65, 0x2e7de9, 0xd8d9de)
            }
            Theme::GruvboxDark => {
                p(0x1d2021, 0x282828, 0x3c3836, 0x504945, 0xaf3a03, 0xebdbb2, 0xa89984, 0xfe8019, 0xb8bb26, 0xfb4934, 0x83a598, 0x262626)
            }
            Theme::Nord => {
                p(0x2e3440, 0x3b4252, 0x434c5e, 0x4c566a, 0x5e81ac, 0xeceff4, 0xaeb7c6, 0xebcb8b, 0xa3be8c, 0xbf616a, 0x88c0d0, 0x343a47)
            }
            Theme::Dracula => {
                p(0x21222c, 0x282a36, 0x44475a, 0x565970, 0x6272a4, 0xf8f8f2, 0xa0a3b8, 0xffb86c, 0x50fa7b, 0xff5555, 0x8be9fd, 0x2a2c38)
            }
        }
    }
}

/// The theme in use, an index in [`Theme::ALL`]: read by the color functions everywhere.
static CURRENT: AtomicU8 = AtomicU8::new(0);

pub fn set(theme: Theme) {
    let index = Theme::ALL.iter().position(|t| *t == theme).unwrap_or(0);
    CURRENT.store(index as u8, Ordering::Relaxed);
}

pub fn current() -> Theme {
    Theme::ALL.get(CURRENT.load(Ordering::Relaxed) as usize).copied().unwrap_or_default()
}

/// White or black, the one that reads best on a background.
pub fn text_on(background: Color) -> Color {
    let l = background.to_linear();
    let luminance = 0.2126 * l.red + 0.7152 * l.green + 0.0722 * l.blue;
    if luminance > 0.36 { Color::srgb(0.08, 0.08, 0.10) } else { Color::WHITE }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The contrast ratio of two colors (WCAG).
    fn contrast(a: Color, b: Color) -> f32 {
        let lum = |c: Color| {
            let l = c.to_linear();
            0.2126 * l.red + 0.7152 * l.green + 0.0722 * l.blue
        };
        let (x, y) = (lum(a), lum(b));
        (x.max(y) + 0.05) / (x.min(y) + 0.05)
    }

    #[test]
    fn readable_themes() {
        for theme in Theme::ALL {
            let p = theme.palette();
            assert!(contrast(p.text, p.bg) >= 7.0, "{} text", theme.name());
            assert!(contrast(p.text, p.button) >= 4.5, "{} button", theme.name());
            assert!(contrast(p.text, p.selected) >= 3.0, "{} selected", theme.name());
            assert!(contrast(p.dim, p.panel) >= 3.0, "{} dim", theme.name());
        }
        set(Theme::Nord);
        assert_eq!(current(), Theme::Nord);
        set(Theme::Dark);
        assert_eq!(text_on(Color::WHITE), Color::srgb(0.08, 0.08, 0.10));
        assert_eq!(text_on(Color::BLACK), Color::WHITE);
    }
}
