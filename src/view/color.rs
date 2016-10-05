pub use termion::color::Rgb as RGBColor;

/// A convenience type used to represent a foreground/background
/// color combination. Provides generic/convenience variants to
/// discourage color selection outside of the theme, whenever possible.
#[derive(Clone)]
pub enum Colors {
    Blank,    // blank/blank
    Default,  // default/background
    Focused,  // default/alt background
    Inverted, // background/default
    Insert,   // white/green
    Modified, // white/yellow
    Select,   // white/blue
    CustomForeground(RGBColor),
    CustomFocusedForeground(RGBColor),
    Custom(RGBColor, RGBColor),
}
