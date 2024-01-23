use std::{path::PathBuf, rc::Rc};

use ratatui::{
    prelude::*,
    widgets::{block::Title, Block, Borders, Cell, Clear, Paragraph, Row, Table, Wrap},
};

use super::{
    app::{AppFocus, AppState, Preview},
    color_theme::ColorTheme,
    components::{confirm_delete::ConfirmDeletePopup, table::StatefulTable},
};

pub struct Renderer {
    colors: ColorTheme,
}

/// The renderer is responsible for rendering widgets in the user interface.
impl Renderer {
    pub fn new(colors: ColorTheme) -> Self {
        Self { colors }
    }

    /// Renders the user interface.
    pub fn render(&self, state: &mut AppState, frame: &mut Frame) {
        // Split frame into 3 horizontal chunks.
        let main_chunks = Self::get_main_layout(frame.size());
        // Split the middle horizontal chunks into 2 equal chunks.
        let middle_chunks = Self::get_middle_layout(main_chunks[1]);

        let [top_block, bottom_block, left_block, right_block] = self.get_blocks();

        self.render_top_panel(frame, main_chunks[0], top_block, state);
        self.render_bottom_panel(frame, main_chunks[2], bottom_block, state);

        self.render_left_panel(frame, middle_chunks[0], left_block, state);
        self.render_right_panel(frame, middle_chunks[1], right_block, state);

        match &state.focus {
            AppFocus::ConfirmDeletePopup(popup) => {
                let popup_area = Self::get_centered_rect(25, 25, frame.size());
                self.render_confirm_delete_popup(frame, popup_area, popup);
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

    fn get_centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(area);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }

    fn get_blocks<'a>(&self) -> [Block<'a>; 4] {
        [
            // The top block
            Block::default()
                .borders(Borders::NONE)
                .border_style(Style::default().fg(self.colors.secondary)),
            // The bottom block
            Block::default()
                .borders(Borders::NONE)
                .border_style(Style::default().fg(self.colors.secondary)),
            // The left block: Renders top and bottom borders.
            Block::default()
                .borders(Borders::TOP | Borders::BOTTOM)
                .border_style(Style::default().fg(self.colors.secondary)),
            // The right block: renders top, bottom and left borders. Creates the connection with the left block's right
            // borders.
            Block::default()
                .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
                .border_set(symbols::border::Set {
                    top_left: symbols::line::NORMAL.horizontal_down,
                    bottom_left: symbols::line::NORMAL.horizontal_up,
                    ..symbols::border::PLAIN
                })
                .border_style(Style::default().fg(self.colors.secondary)),
        ]
    }

    fn render_top_panel(
        &self,
        frame: &mut Frame,
        area: Rect,
        block: Block<'_>,
        _state: &mut AppState,
    ) {
        let path = Paragraph::new("~/example")
            .block(block)
            .style(Style::default().fg(self.colors.tertiary));
        frame.render_widget(path, area);
    }

    fn render_bottom_panel(
        &self,
        frame: &mut Frame,
        area: Rect,
        block: Block<'_>,
        _state: &mut AppState,
    ) {
        let commands = Paragraph::new("Commands: q(uit), s(elect)")
            .block(block)
            .style(Style::default().fg(self.colors.fg));
        frame.render_widget(commands, area);
    }

    fn render_left_panel(
        &self,
        frame: &mut Frame,
        area: Rect,
        block: Block<'_>,
        state: &mut AppState,
    ) {
        self.render_table(frame, area, block, &mut state.main_table, &state.focus);
    }

    fn render_right_panel(
        &self,
        frame: &mut Frame,
        area: Rect,
        block: Block<'_>,
        state: &mut AppState,
    ) {
        match &mut state.preview {
            Preview::Table(table) => {
                self.render_preview_table(frame, area, block, table, &state.focus)
            }
            Preview::Text(text) => self.render_preview_paragraph(frame, area, block, text),
            Preview::EmptyDirectory => self.render_preview_empty(frame, area, block),
        }
    }

    fn render_table(
        &self,
        frame: &mut Frame,
        area: Rect,
        block: Block<'_>,
        table_state: &mut StatefulTable<PathBuf>,
        focus: &AppFocus,
    ) {
        let rows = table_state.items.iter().enumerate().map(|(index, data)| {
            Row::new(vec![
                Cell::from(Span::styled(
                    if table_state.is_selected(index) {
                        "▌"
                    } else {
                        ""
                    },
                    Style::default().fg(self.colors.tertiary),
                )),
                Cell::from(Text::from(data.file_name().unwrap().to_str().unwrap())),
            ])
            .style(self.get_row_style(index, table_state, focus))
        });

        let table = Table::new(rows, [Constraint::Length(1), Constraint::Min(10)]).block(block);

        frame.render_stateful_widget(table, area, &mut table_state.state);
    }

    fn get_row_style(
        &self,
        index: usize,
        table_state: &StatefulTable<PathBuf>,
        focus: &AppFocus,
    ) -> Style {
        if table_state.is_focused(index) {
            match focus {
                AppFocus::MainScreen => Style::default().bg(self.colors.primary).fg(self.colors.bg),
                AppFocus::ConfirmDeletePopup(_) => Style::default().hidden(),
            }
        } else {
            Style::default()
        }
    }

    fn render_preview_table(
        &self,
        frame: &mut Frame,
        area: Rect,
        block: Block<'_>,
        table_state: &mut StatefulTable<PathBuf>,
        focus: &AppFocus,
    ) {
        self.render_table(frame, area, block, table_state, focus);
    }

    fn render_preview_empty(&self, frame: &mut Frame, area: Rect, block: Block<'_>) {
        let text = Text::styled(
            "Empty directory",
            Style::new()
                .bg(self.colors.fg)
                .fg(self.colors.bg)
                .add_modifier(Modifier::UNDERLINED),
        );

        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
    }

    fn render_preview_paragraph(
        &self,
        frame: &mut Frame,
        area: Rect,
        block: Block<'_>,
        state: &str,
    ) {
        let text = Paragraph::new(state)
            .block(block)
            .style(Style::default().fg(self.colors.fg));
        frame.render_widget(text, area);
    }

    fn render_confirm_delete_popup(
        &self,
        frame: &mut Frame,
        area: Rect,
        confirm_delete_popup: &ConfirmDeletePopup,
    ) {
        frame.render_widget(Clear, area);
        let block = Block::default()
            .title(Title::from("Confirm delete"))
            .title_style(Style::default().fg(self.colors.primary))
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.colors.secondary));

        frame.render_widget(block, area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1), Constraint::Length(1)])
            .margin(1)
            .split(area);
        let text_area = layout[0];

        let text = Paragraph::new("Are you sure you want to delete ...?")
            .style(Style::default().fg(self.colors.fg))
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

        let mut yes = Span::from("Yes").style(Style::default().fg(self.colors.fg));
        let mut no = Span::from("No").style(Style::default().fg(self.colors.fg));

        if confirm_delete_popup.confirmed() {
            yes = yes.fg(self.colors.primary);
        } else {
            no = no.fg(self.colors.primary);
        }

        frame.render_widget(Paragraph::new(yes).alignment(Alignment::Center), yes_area);
        frame.render_widget(Paragraph::new(no).alignment(Alignment::Center), no_area);
    }
}
