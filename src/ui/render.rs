use std::{path::PathBuf, rc::Rc};

use ratatui::{
    prelude::*,
    widgets::{block::Title, Block, Borders, Cell, Clear, Paragraph, Row, Table, Wrap},
};

use super::{
    app::{AppFocus, AppState, Preview},
    components::{confirm_delete::ConfirmDeletePopup, table::StatefulTable},
};

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

    match &state.focus {
        AppFocus::ConfirmDeletePopup(popup) => {
            let popup_area = centered_rect(25, 25, frame.size());
            render_confirm_delete_popup(frame, popup_area, popup);
        }
        AppFocus::MainScreen => (),
    }
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
        // use a 49/51 split instead of 50/50 to ensure that any extra space is on the right
        // side of the screen which renders the border between the two areas.
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
    let highlight_style = match &mut state.focus {
        AppFocus::MainScreen => Style::default().bg(Color::Yellow).fg(Color::Black),
        AppFocus::ConfirmDeletePopup(_) => Style::default().hidden(),
    };

    let rows = state
        .main_table
        .items
        .iter()
        .enumerate()
        .map(|(index, data)| {
            Row::new(vec![
                Cell::from(Span::styled(
                    if state.main_table.is_selected(index) {
                        "▌"
                    } else {
                        ""
                    },
                    Style::default().fg(Color::LightGreen),
                )),
                Cell::from(Text::from(data.file_name().unwrap().to_str().unwrap())),
            ])
        });

    let table = Table::new(rows, [Constraint::Length(1), Constraint::Min(10)])
        .highlight_style(highlight_style)
        .block(block);

    frame.render_stateful_widget(table, area, &mut state.main_table.state);
}

fn render_right_panel(frame: &mut Frame, area: Rect, state: &mut AppState, block: Block<'_>) {
    match &mut state.preview {
        Preview::Table(table) => render_preview_table(frame, area, table, block),
        Preview::Text(text) => render_preview_paragraph(frame, area, text, block),
        Preview::EmptyDirectory => render_preview_empty(frame, area, block),
    }
}

fn render_preview_table(
    frame: &mut Frame,
    area: Rect,
    state: &mut StatefulTable<PathBuf>,
    block: Block<'_>,
) {
    let rows = state.items.iter().map(|data| {
        Row::new(vec![Cell::from(Text::from(
            data.file_name().unwrap().to_str().unwrap(),
        ))])
    });

    let table = Table::new(
        rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .block(block);

    frame.render_stateful_widget(table, area, &mut state.state);
}

fn render_preview_empty(frame: &mut Frame, area: Rect, block: Block<'_>) {
    let text = Text::styled(
        "Empty directory",
        Style::new()
            .bg(Color::White)
            .fg(Color::Black)
            .add_modifier(Modifier::UNDERLINED),
    );

    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

fn render_preview_paragraph(frame: &mut Frame, area: Rect, state: &str, block: Block<'_>) {
    let text = Paragraph::new(state).block(block);
    frame.render_widget(text, area);
}

fn render_top_panel(frame: &mut Frame, area: Rect, _state: &mut AppState, block: Block<'_>) {
    let path = Paragraph::new("~/example").block(block);
    frame.render_widget(path, area);
}

fn render_bottom_panel(frame: &mut Frame, area: Rect, _state: &mut AppState, block: Block<'_>) {
    let commands = Paragraph::new("Commands: q(uit), s(elect)").block(block);
    frame.render_widget(commands, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_confirm_delete_popup(
    frame: &mut Frame,
    area: Rect,
    confirm_delete_popup: &ConfirmDeletePopup,
) {
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(Title::from("Confirm delete"))
        .title_style(Style::default().fg(Color::Yellow))
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL);

    frame.render_widget(block, area);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .margin(1)
        .split(area);
    let text_area = layout[0];

    let text = Paragraph::new("Are you sure you want to delete ...?")
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center);

    frame.render_widget(text, text_area);

    let tabs_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(layout[1]);

    let yes_area = tabs_area[1];
    let no_area = tabs_area[2];

    let mut yes = Span::from("Yes");
    let mut no = Span::from("No");

    if confirm_delete_popup.selected_yes() {
        yes = yes.fg(Color::Yellow);
    } else {
        no = no.fg(Color::Yellow);
    }

    frame.render_widget(Paragraph::new(yes).alignment(Alignment::Center), yes_area);
    frame.render_widget(Paragraph::new(no).alignment(Alignment::Center), no_area);
}
