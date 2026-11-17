//! App themes: built-in presets (styled by classes in `themes/*.css`) and
//! user-defined custom palettes (applied as CSS variables in a style attribute).

use dioxus::prelude::*;
use std::fmt;

pub type ThemeSignal = Signal<Theme>;

/// A `#rrggbb` color.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rgb(u32);

impl Rgb {
    pub const fn new(hex: u32) -> Self {
        Rgb(hex & 0xff_ffff)
    }

    /// Accepts `#rrggbb` or `#rgb`, with or without the leading `#`.
    pub fn parse(s: &str) -> Option<Self> {
        let hex = s.trim().trim_start_matches('#');
        if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        match hex.len() {
            6 => u32::from_str_radix(hex, 16).ok().map(Rgb),
            3 => {
                let expanded: String = hex.chars().flat_map(|c| [c, c]).collect();
                u32::from_str_radix(&expanded, 16).ok().map(Rgb)
            }
            _ => None,
        }
    }
}

impl fmt::Display for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:06x}", self.0)
    }
}

/// The colors a theme controls. Field order matches [`Palette::FIELDS`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Palette {
    pub bg: Rgb,
    pub fg: Rgb,
    pub dim: Rgb,
    pub faint: Rgb,
    pub danger: Rgb,
}

impl Palette {
    /// Labels (also the `--color-*` variable suffixes), in storage order.
    pub const FIELDS: [&'static str; 5] = ["bg", "fg", "dim", "faint", "danger"];

    /// Same values as the defaults in `tailwind.css`.
    pub const DARK: Palette = Palette {
        bg: Rgb::new(0x000000),
        fg: Rgb::new(0xe5e5e5),
        dim: Rgb::new(0x7a7a7a),
        faint: Rgb::new(0x262626),
        danger: Rgb::new(0xcc3333),
    };

    pub fn colors(&self) -> [Rgb; 5] {
        [self.bg, self.fg, self.dim, self.faint, self.danger]
    }

    pub fn from_colors([bg, fg, dim, faint, danger]: [Rgb; 5]) -> Self {
        Palette { bg, fg, dim, faint, danger }
    }

    /// Storage format: comma-separated colors in [`Palette::FIELDS`] order.
    pub fn serialize(&self) -> String {
        self.colors().map(|c| c.to_string()).join(",")
    }

    pub fn deserialize(s: &str) -> Option<Self> {
        let colors: Vec<Rgb> = s.split(',').map(Rgb::parse).collect::<Option<_>>()?;
        Some(Palette::from_colors(colors.try_into().ok()?))
    }

    /// `style` attribute value that overrides the Tailwind color variables.
    pub fn style(&self) -> String {
        let mut style = String::new();
        for (name, color) in Self::FIELDS.iter().zip(self.colors()) {
            style.push_str(&format!("--color-{name}: {color}; "));
        }
        // Keep hover close to bg, like the presets do.
        style.push_str(&format!("--color-hover: {};", self.bg));
        style
    }
}

impl Default for Palette {
    fn default() -> Self {
        Palette::DARK
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Theme {
    #[default]
    Dark,
    Light,
    Custom(Palette),
}

impl Theme {
    /// Stable key stored in the settings table.
    pub fn key(&self) -> &'static str {
        match self {
            Theme::Dark => "dark",
            Theme::Light => "light",
            Theme::Custom(_) => "custom",
        }
    }

    /// Preset class from `themes/*.css`; dark is the default palette and needs none.
    pub fn class(&self) -> &'static str {
        match self {
            Theme::Light => "light",
            Theme::Dark | Theme::Custom(_) => "",
        }
    }

    /// Style attribute for custom palettes; presets are styled by their class.
    pub fn style(&self) -> String {
        match self {
            Theme::Custom(palette) => palette.style(),
            Theme::Dark | Theme::Light => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_parses_and_formats() {
        assert_eq!(Rgb::parse("#1a2B3c"), Some(Rgb::new(0x1a2b3c)));
        assert_eq!(Rgb::parse("fff"), Some(Rgb::new(0xffffff)));
        assert_eq!(Rgb::parse(" #000000 "), Some(Rgb::new(0)));
        assert_eq!(Rgb::parse("#12345"), None);
        assert_eq!(Rgb::parse("#gggggg"), None);
        assert_eq!(Rgb::parse("+12345"), None);
        assert_eq!(Rgb::new(0xab).to_string(), "#0000ab");
    }

    #[test]
    fn palette_round_trips() {
        let p = Palette::DARK;
        assert_eq!(p.serialize(), "#000000,#e5e5e5,#7a7a7a,#262626,#cc3333");
        assert_eq!(Palette::deserialize(&p.serialize()), Some(p));
        assert_eq!(Palette::deserialize("#000000,#ffffff"), None);
        assert_eq!(Palette::deserialize("#000000,#ffffff,#000000,#000000,nope"), None);
    }

    #[test]
    fn palette_style_sets_variables() {
        let style = Palette::DARK.style();
        assert!(style.contains("--color-bg: #000000;"));
        assert!(style.contains("--color-danger: #cc3333;"));
        assert!(style.contains("--color-hover: #000000;"));
    }
}
