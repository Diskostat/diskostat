use ratatui::style::Color;

pub struct ColorTheme {
    /// Primary color of the application, used for directories and their focus.
    pub primary: Color,
    /// Secondary color of the application, used to contrast the primary color.
    pub secondary: Color,
    /// Tertiary color of the application, used to contrast the primary color.
    pub tertiary: Color,
    /// Used for default text color.
    pub fg: Color,
    /// Background color used for inverting text colors.
    pub bg: Color,
    /// Used for highlighting text.
    pub highlight: Color,
}

impl ColorTheme {
    pub fn new(
        primary: Color,
        secondary: Color,
        tertiary: Color,
        fg: Color,
        bg: Color,
        highlight: Color,
    ) -> Self {
        Self {
            primary,
            secondary,
            tertiary,
            fg,
            bg,
            highlight,
        }
    }
}

impl Default for ColorTheme {
    fn default() -> Self {
        Self {
            primary: Color::Yellow,
            secondary: Color::Blue,
            tertiary: Color::Green,
            fg: Color::White,
            bg: Color::Black,
            highlight: Color::Red,
        }
    }
}
