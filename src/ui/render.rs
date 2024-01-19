use ratatui::{
    prelude::{Alignment, Frame},
    widgets::Paragraph,
};

use super::app::AppState;

/// Renders the user interface.
pub fn render(_state: &mut AppState, frame: &mut Frame) {
    frame.render_widget(
        Paragraph::new("Hello, World!").alignment(Alignment::Center),
        frame.size(),
    )
}
