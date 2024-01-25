use ratatui::style::Color;

pub struct ColorTheme {
    pub primary: Color,
    pub secondary: Color,
    pub tertiary: Color,
    pub fg: Color,
    pub bg: Color,
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
        Self::new(
            Color::Yellow,
            Color::Blue,
            Color::Green,
            Color::White,
            Color::Black,
            Color::Red,
        )
    }
}
