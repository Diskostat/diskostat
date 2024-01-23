use ratatui::style::Color;

pub struct ColorTheme {
    pub file_fg: Color,
    pub dir_fg: Color,
    pub highlighted_file_fg: Color,
    pub highlighted_dir_fg: Color,
    pub highlighted_file_bg: Color,
    pub highlighted_dir_bg: Color,
    pub progress_bar_fg_25: Color,
    pub progress_bar_fg_50: Color,
    pub progress_bar_fg_75: Color,
    pub progress_bar_fg_100: Color,
    pub progress_bar_bg: Color,
    pub border_fg: Color,
    pub empty_dir_fg: Color,
    pub empty_dir_bg: Color,
    pub cwd_fg: Color,
}

impl ColorTheme {
    pub fn new() -> Self {
        Self {
            file_fg: Color::White,
            dir_fg: Color::Blue,
            highlighted_file_fg: Color::Black,
            highlighted_dir_fg: Color::Black,
            highlighted_file_bg: Color::White,
            highlighted_dir_bg: Color::Blue,
            progress_bar_fg_25: Color::Rgb(51, 255, 51),
            progress_bar_fg_50: Color::Rgb(255, 255, 51),
            progress_bar_fg_75: Color::Rgb(255, 153, 51),
            progress_bar_fg_100: Color::Rgb(255, 51, 51),
            progress_bar_bg: Color::DarkGray,
            border_fg: Color::Yellow,
            empty_dir_fg: Color::Black,
            empty_dir_bg: Color::White,
            cwd_fg: Color::Cyan,
        }
    }
}

impl Default for ColorTheme {
    fn default() -> Self {
        Self::new()
    }
}
