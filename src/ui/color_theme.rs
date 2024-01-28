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
    pub primary_bg: Color,
    /// Background color used as a background for the progress bar.
    pub secondary_bg: Color,
    /// Used for highlighting text.
    pub highlight: Color,
    /// Whether to use a colorful progress bar and percentages.
    pub colorful_bar: bool,
}

impl ColorTheme {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        primary: Color,
        secondary: Color,
        tertiary: Color,
        fg: Color,
        primary_bg: Color,
        secondary_bg: Color,
        highlight: Color,
        colorful_bar: bool,
    ) -> Self {
        Self {
            primary,
            secondary,
            tertiary,
            fg,
            primary_bg,
            secondary_bg,
            highlight,
            colorful_bar,
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
            primary_bg: Color::Black,
            secondary_bg: Color::DarkGray,
            highlight: Color::Red,
            colorful_bar: true,
        }
    }
}
