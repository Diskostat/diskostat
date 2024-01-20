use std::rc::Rc;

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use super::app::AppState;

/// Renders the user interface.
pub fn render(state: &mut AppState, frame: &mut Frame) {
    // Split frame into 3 horizontal chunks.
    let main_chunks = get_main_layout(frame.size());
    // Split the middle horizontal chunks into 2 equal chunks.
    let middle_chunks = get_middle_layout(main_chunks[1]);

    let [top_block, bottom_block, left_block, right_block] = get_blocks();

    render_top_panel(frame, main_chunks[0], state, top_block);
    render_bottom_panel(frame, main_chunks[2], state, bottom_block);

    render_left_panel(frame, middle_chunks[0], state, left_block);
    render_right_panel(frame, middle_chunks[1], state, right_block);
}

fn get_main_layout(area: Rect) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(area)
}

fn get_middle_layout(area: Rect) -> Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(49), Constraint::Percentage(51)])
        .split(area)
}

fn get_blocks<'a>() -> [Block<'a>; 4] {
    [
        // The top block
        Block::default().borders(Borders::NONE),
        // The bottom block
        Block::default().borders(Borders::NONE),
        // The left block: Renders top and bottom borders.
        Block::default().borders(Borders::TOP | Borders::BOTTOM),
        // The right block: renders top, bottom and left borders. Creates the connection with the left block's right
        // borders.
        Block::default()
            .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
            .border_set(symbols::border::Set {
                top_left: symbols::line::NORMAL.horizontal_down,
                bottom_left: symbols::line::NORMAL.horizontal_up,
                ..symbols::border::PLAIN
            }),
    ]
}

fn render_left_panel(frame: &mut Frame, area: Rect, state: &mut AppState, block: Block<'_>) {
    let highlight_style = Style::default().bg(Color::Yellow).fg(Color::Black);

    let rows = state
        .main_table
        .items
        .iter()
        .map(|data| Row::new(vec![Cell::from(Text::from(data.to_string()))]));

    let table = Table::new(
        rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .highlight_style(highlight_style)
    .block(block);

    frame.render_stateful_widget(table, area, &mut state.main_table.state);
}

fn render_right_panel(frame: &mut Frame, area: Rect, _state: &mut AppState, block: Block<'_>) {
    frame.render_widget(block, area);
}

fn render_top_panel(frame: &mut Frame, area: Rect, _state: &mut AppState, block: Block<'_>) {
    let path = Paragraph::new("~/example").block(block);
    frame.render_widget(path, area);
}

fn render_bottom_panel(frame: &mut Frame, area: Rect, _state: &mut AppState, block: Block<'_>) {
    let commands = Paragraph::new("Commands: q(uit)").block(block);
    frame.render_widget(commands, area);
}
