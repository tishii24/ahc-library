use num_traits::Num;

use crate::utils::env::env_is_one;

pub const AHC_DUMP_ENABLED: bool = env_is_one(option_env!("AHC_DUMP"));

pub const ANSI_RESET: &str = "\x1b[0m";
pub const ANSI_BOLD: &str = "\x1b[1m";

/// `eprintln!` + ANSI style (`ColoredText` or `Color`).
/// `dump!` is the same but without the newline. Both macros do nothing if the "dump" feature is disabled.
///
/// Set `AHC_DUMP=1` in the environment to enable dumping.
///
/// Examples:
/// - `dumpln!("a={}", a);` (no color)
/// - `dumpln!(RED, "a: {}, b: {}", a, b);` (Color as fg)
/// - `dumpln!(ColoredText::new().fg(RED).bg(BLUE), "{}", "hello");`
/// - `dumpln!(ColoredText::new().fg(RED).bold(), "bold!");`
#[macro_export]
macro_rules! dumpln {
    // no color
    ($fmt:literal $(, $arg:expr)*) => {
        if const { $crate::utils::dump::AHC_DUMP_ENABLED } {
            eprintln!($fmt $(, $arg)*);
        }
    };
    // with style (Color or ColoredText)
    ($style:expr, $fmt:literal $(, $arg:expr)*) => {{
        if const { $crate::utils::dump::AHC_DUMP_ENABLED } {
            let __style: $crate::utils::dump::ColoredText = $style.into();
            let __prefix = $crate::utils::dump::ansi_prefix(__style);
            eprintln!("{}{}{}", __prefix, format_args!($fmt $(, $arg)*), $crate::utils::dump::ANSI_RESET);
        }
    }};
}

#[macro_export]
macro_rules! dump {
    // no color
    ($fmt:literal $(, $arg:expr)*) => {{
        if const { $crate::utils::dump::AHC_DUMP_ENABLED } {
            eprint!($fmt $(, $arg)*);
        }
    }};
    // with style (Color or ColoredText)
    ($style:expr, $fmt:literal $(, $arg:expr)*) => {{
        if const { $crate::utils::dump::AHC_DUMP_ENABLED } {
            let __style: $crate::utils::dump::ColoredText = $style.into();
            let __prefix = $crate::utils::dump::ansi_prefix(__style);
            eprint!("{}{}{}", __prefix, format_args!($fmt $(, $arg)*), $crate::utils::dump::ANSI_RESET);
        }
    }};
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub br: f64,
}

pub mod color {
    use super::Color;

    pub const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        br: 1.0,
    };
    pub const GREEN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        br: 1.0,
    };
    pub const BLUE: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        br: 1.0,
    };
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        br: 1.0,
    };
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        br: 1.0,
    };
    pub const YELLOW: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 0.0,
        br: 1.0,
    };
    pub const CYAN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 1.0,
        br: 1.0,
    };
    pub const MAGENTA: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 1.0,
        br: 1.0,
    };
}

impl Default for Color {
    fn default() -> Self {
        color::BLACK
    }
}

impl Color {
    pub fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b, br: 1.0 }
    }

    pub fn with_br(self, br: f64) -> Self {
        Self { br, ..self }
    }

    /// Convert to (r, g, b) in 0..=255 (without br).
    pub fn to_rgb8(self) -> (u8, u8, u8) {
        let to_u8 = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        (to_u8(self.r), to_u8(self.g), to_u8(self.b))
    }

    /// Linearly interpolate between `self` (t=0) and `other` (t=1).
    pub fn lerp(self, other: Color, t: f64) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            br: self.br + (other.br - self.br) * t,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColoredText {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub is_bold: bool,
}

impl ColoredText {
    pub fn new() -> Self {
        Self {
            fg: None,
            bg: None,
            is_bold: false,
        }
    }

    pub fn fg(self, color: Color) -> Self {
        Self {
            fg: Some(color),
            ..self
        }
    }

    pub fn bg(self, color: Color) -> Self {
        Self {
            bg: Some(color),
            ..self
        }
    }

    pub fn bold(self) -> Self {
        Self {
            is_bold: true,
            ..self
        }
    }
}

impl Default for ColoredText {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Color> for ColoredText {
    fn from(color: Color) -> Self {
        Self {
            fg: Some(color),
            ..Self::new()
        }
    }
}

/// ANSI truecolor foreground escape sequence with br applied.
pub fn ansi_fg(color: Color) -> String {
    let b = color.br.clamp(0.0, 1.0);
    let to_u8 = |v: f64| (v.clamp(0.0, 1.0) * b * 255.0).round() as u8;
    format!(
        "\x1b[38;2;{};{};{}m",
        to_u8(color.r),
        to_u8(color.g),
        to_u8(color.b)
    )
}

/// ANSI truecolor background escape sequence with br applied.
pub fn ansi_bg(color: Color) -> String {
    let b = color.br.clamp(0.0, 1.0);
    let to_u8 = |v: f64| (v.clamp(0.0, 1.0) * b * 255.0).round() as u8;
    format!(
        "\x1b[48;2;{};{};{}m",
        to_u8(color.r),
        to_u8(color.g),
        to_u8(color.b)
    )
}

pub fn ansi_prefix(style: ColoredText) -> String {
    let mut s = String::new();
    if style.is_bold {
        s.push_str(ANSI_BOLD);
    }
    if let Some(fg) = style.fg {
        s.push_str(&ansi_fg(fg));
    }
    if let Some(bg) = style.bg {
        s.push_str(&ansi_bg(bg));
    }
    s
}

/// Dump a 2D matrix to stderr, coloring each cell by its normalized value.
///
/// Each value is normalized to [0, 1] over the whole matrix, then the fg color
/// is interpolated from `low_color` (min) to `high_color` (max).
pub fn dump_2d<T>(v: &[impl AsRef<[T]>], low_color: Color, high_color: Color)
where
    T: std::fmt::Display + Copy + Num + num_traits::ToPrimitive + PartialOrd,
{
    // Collect all values as f64 to find min/max.
    let vals: Vec<f64> = v
        .iter()
        .flat_map(|row| row.as_ref().iter())
        .filter_map(|x| x.to_f64())
        .collect();

    let min_val = vals.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_val = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let range = max_val - min_val;

    for row in v {
        for val in row.as_ref() {
            let f = val.to_f64().unwrap_or(0.0);
            let t = if range > 0.0 {
                (f - min_val) / range
            } else {
                0.5
            };
            let color = low_color.lerp(high_color, t);
            let style = ColoredText::new().fg(color);
            dump!(style, "{:>4} ", val);
        }
        eprintln!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use color::*;

    #[test]
    fn test_color_to_rgb8() {
        assert_eq!(Color::new(0.0, 0.0, 0.0).to_rgb8(), (0, 0, 0));
        assert_eq!(RED.to_rgb8(), (255, 0, 0));
        assert_eq!(GREEN.to_rgb8(), (0, 255, 0));
        assert_eq!(BLUE.to_rgb8(), (0, 0, 255));
    }

    #[test]
    fn test_ansi_fg_br() {
        // br is now on Color
        let red_dim = RED.with_br(0.8);
        let v = (0.8_f64 * 255.0).round() as u8;
        assert_eq!(ansi_fg(red_dim), format!("\x1b[38;2;{};0;0m", v));
        assert_eq!(ansi_fg(RED.with_br(1.0)), "\x1b[38;2;255;0;0m");
        assert_eq!(ansi_fg(RED.with_br(0.0)), "\x1b[38;2;0;0;0m");

        // From<Color> for ColoredText keeps the color's br
        let style: ColoredText = RED.into();
        assert_eq!(style.fg.unwrap().br, 1.0);
    }

    #[test]
    fn test_dump() {
        dumpln!(RED, "This is red (br=1.0)");
        dumpln!(GREEN, "This is green (br=1.0)");
        dumpln!(ColoredText::new().fg(RED), "Full br red");
        dumpln!(ColoredText::new().fg(RED.with_br(0.3)), "Dim red");
        dumpln!(
            ColoredText::new().fg(YELLOW).bg(BLUE.with_br(0.4)),
            "Yellow fg bright, blue bg dim"
        );
        dumpln!(ColoredText::new().fg(CYAN).bold(), "Bold cyan");
        dump!(
            ColoredText::new()
                .fg(MAGENTA.with_br(0.9))
                .bg(WHITE.with_br(0.5))
                .bold(),
            "Magenta fg bright, white bg dim, bold\n"
        );
        dumpln!(ColoredText::new(), "Default style");

        let matrix = vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]];
        dump_2d(&matrix, BLUE, RED);
        dump_2d(&matrix, Color::default(), RED);

        dumpln!("Test dumpln without color: {}", 42);
        dump!("Test dump without color: {}\n", 42);
    }
}
