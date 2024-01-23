use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use super::{
    app::{AppState, Preview},
    color_theme::ColorTheme,
    components::table::StatefulTable,
};

pub struct Renderer {
    color_theme: ColorTheme,
}

impl Renderer {
    pub fn new(color_theme: ColorTheme) -> Self {
        Self { color_theme }
    }

    /// Renders the user interface.
    pub fn render(&self, state: &mut AppState, frame: &mut Frame) {
        // Split frame into 3 horizontal chunks.
        let main_chunks = self.get_main_layout(frame.size());
        // Split the middle horizontal chunks into 2 equal chunks.
        let middle_chunks = self.get_middle_layout(main_chunks[1]);

        let [top_block, bottom_block, left_block, right_block] = self.get_blocks();

        self.render_top_panel(frame, main_chunks[0], state, top_block);
        self.render_bottom_panel(frame, main_chunks[2], state, bottom_block);

        self.render_left_panel(frame, middle_chunks[0], state, left_block);
        self.render_right_panel(frame, middle_chunks[1], state, right_block);
    }

    fn get_main_layout(&self, area: Rect) -> Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(3),
                Constraint::Length(1),
            ])
            .split(area)
    }

    fn get_middle_layout(&self, area: Rect) -> Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Horizontal)
            // use a 49/51 split instead of 50/50 to ensure that any extra space is on the right
            // side of the screen which renders the border between the two areas.
            .constraints([Constraint::Percentage(49), Constraint::Percentage(51)])
            .split(area)
    }

    fn get_blocks<'a>(&self) -> [Block<'a>; 4] {
        [
            // The top block
            Block::default()
                .borders(Borders::NONE)
                .border_style(Style::default().fg(self.color_theme.border_fg)),
            // The bottom block
            Block::default()
                .borders(Borders::NONE)
                .border_style(Style::default().fg(self.color_theme.border_fg)),
            // The left block: Renders top and bottom borders.
            Block::default()
                .borders(Borders::TOP | Borders::BOTTOM)
                .border_style(Style::default().fg(self.color_theme.border_fg)),
            // The right block: renders top, bottom and left borders. Creates the connection with the left block's right
            // borders.
            Block::default()
                .borders(Borders::LEFT | Borders::TOP | Borders::BOTTOM)
                .border_set(symbols::border::Set {
                    top_left: symbols::line::NORMAL.horizontal_down,
                    bottom_left: symbols::line::NORMAL.horizontal_up,
                    ..symbols::border::PLAIN
                })
                .border_style(Style::default().fg(self.color_theme.border_fg)),
        ]
    }

    fn get_progress_bar_cell<'a>(&self, rate: f64) -> Cell<'a> {
        let fill = (rate * 10.0) as usize;
        let percentage = (rate * 100.0) as usize;
        let color = if percentage < 25 {
            self.color_theme.progress_bar_fg_25
        } else if percentage < 50 {
            self.color_theme.progress_bar_fg_50
        } else if percentage < 75 {
            self.color_theme.progress_bar_fg_75
        } else {
            self.color_theme.progress_bar_fg_100
        };

        Cell::from(Line::from(vec![
            Span::from("\u{25AC}".repeat(fill)).set_style(Style::default().fg(color)),
            Span::from("\u{25AC}".repeat(10 - fill))
                .set_style(Style::default().fg(self.color_theme.progress_bar_bg)),
        ]))
    }

    fn get_file_name_cell<'a>(&self, path: &'a Path, is_focused: bool) -> Cell<'a> {
        Cell::from(Text::from(path.file_name().unwrap().to_str().unwrap())).style(if is_focused {
            Style::default().fg(self.color_theme.highlighted_file_fg)
        } else {
            Style::default().fg(self.color_theme.file_fg)
        })
    }

    fn render_left_panel(
        &self,
        frame: &mut Frame,
        area: Rect,
        state: &mut AppState,
        block: Block<'_>,
    ) {
        let rows = state.main_table.items.iter().enumerate().map(|(i, data)| {
            let is_focused = Some(i) == state.main_table.state.selected();
            let rate = (i % 10) as f64 / 10.0;

            Row::new(vec![
                self.get_file_name_cell(data, is_focused),
                self.get_progress_bar_cell(rate),
            ])
            .set_style(if is_focused {
                Style::default().bg(self.color_theme.highlighted_file_bg)
            } else {
                Style::default()
            })
        });

        let table = Table::new(
            rows,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .block(block);

        frame.render_stateful_widget(table, area, &mut state.main_table.state);
    }

    fn render_right_panel(
        &self,
        frame: &mut Frame,
        area: Rect,
        state: &mut AppState,
        block: Block<'_>,
    ) {
        match &mut state.preview {
            Preview::Table(table) => self.render_preview_table(frame, area, table, block),
            Preview::Text(text) => self.render_preview_paragraph(frame, area, text, block),
            Preview::EmptyDirectory => self.render_preview_empty(frame, area, block),
        }
    }

    fn render_preview_table(
        &self,
        frame: &mut Frame,
        area: Rect,
        state: &mut StatefulTable<PathBuf>,
        block: Block<'_>,
    ) {
        let rows = state
            .items
            .iter()
            .map(|data| Row::new(vec![self.get_file_name_cell(data, false)]));

        let table = Table::new(
            rows,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .block(block);

        frame.render_stateful_widget(table, area, &mut state.state);
    }

    fn render_preview_empty(&self, frame: &mut Frame, area: Rect, block: Block<'_>) {
        let text = Text::styled(
            "Empty directory",
            Style::new()
                .bg(self.color_theme.empty_dir_bg)
                .fg(self.color_theme.empty_dir_fg)
                .add_modifier(Modifier::UNDERLINED),
        );

        let paragraph = Paragraph::new(text).block(block);
        frame.render_widget(paragraph, area);
    }

    fn render_preview_paragraph(
        &self,
        frame: &mut Frame,
        area: Rect,
        state: &str,
        block: Block<'_>,
    ) {
        let text = Paragraph::new(state).block(block);
        frame.render_widget(text, area);
    }

    fn render_top_panel(
        &self,
        frame: &mut Frame,
        area: Rect,
        _state: &mut AppState,
        block: Block<'_>,
    ) {
        let path = Paragraph::new("~/example")
            .block(block)
            .set_style(Style::default().fg(self.color_theme.cwd_fg));
        frame.render_widget(path, area);
    }

    fn render_bottom_panel(
        &self,
        frame: &mut Frame,
        area: Rect,
        _state: &mut AppState,
        block: Block<'_>,
    ) {
        let commands = Paragraph::new("Commands: q(uit)").block(block);
        frame.render_widget(commands, area);
    }
}
