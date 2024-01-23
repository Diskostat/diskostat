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
    pub fn new() -> Self {
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

impl Default for ColorTheme {
    fn default() -> Self {
        Self::new()
    }
}
