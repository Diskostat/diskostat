use std::rc::Rc;

use byte_unit::Byte;
use ratatui::{
    prelude::*,
    widgets::{block::Title, Block, Borders, Cell, Clear, Paragraph, Row, Table, Wrap},
};

use crate::backend::model::entry_node::EntryNodeView;

use super::{
    app::{AppFocus, AppState, Preview},
    color_theme::ColorTheme,
    components::{confirm_delete::ConfirmDeletePopup, table::StatefulTable},
};

const BAR_SIZE: usize = 10;

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
            AppFocus::BufferingInput => (),
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
        state: &mut AppState,
    ) {
        let path = Paragraph::new(state.current_directory.path.display().to_string())
            .block(block)
            .style(Style::default().fg(self.colors.tertiary));
        frame.render_widget(path, area);
    }

    fn render_bottom_panel(
        &self,
        frame: &mut Frame,
        area: Rect,
        block: Block<'_>,
        state: &mut AppState,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        frame.render_widget(block, area);

        let input =
            Paragraph::new(state.message.clone()).style(Style::default().fg(self.colors.fg));
        frame.render_widget(input, chunks[0]);

        let commands = Paragraph::new("Commands: q(uit), s(elect), b(ar)")
            .style(Style::default().fg(self.colors.fg));
        frame.render_widget(commands, chunks[1]);
    }

    fn render_left_panel(
        &self,
        frame: &mut Frame,
        area: Rect,
        block: Block<'_>,
        state: &mut AppState,
    ) {
        self.render_table(
            frame,
            area,
            block,
            &state.current_directory,
            &mut state.main_table,
            &state.focus,
            state.show_bar,
        );
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
                self.render_preview_table(
                    frame,
                    area,
                    block,
                    state
                        .main_table
                        .focused()
                        .expect("a directory should be focused"),
                    table,
                    &state.focus,
                    state.show_bar,
                );
            }
            Preview::Text(text) => self.render_preview_paragraph(frame, area, block, text),
            Preview::EmptyDirectory => self.render_preview_empty_directory(frame, area, block),
            Preview::Empty => frame.render_widget(block, area),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_table(
        &self,
        frame: &mut Frame,
        area: Rect,
        block: Block<'_>,
        parent: &EntryNodeView,
        table_state: &mut StatefulTable<EntryNodeView>,
        app_focus: &AppFocus,
        show_bar: bool,
    ) {
        let rows = table_state.items.iter().enumerate().map(|(index, data)| {
            let is_focused = table_state.is_focused(index);
            let is_selected = table_state.is_selected(index);

            Row::new(vec![
                self.get_selection_cell(is_selected),
                self.get_name_cell(data.name.clone(), is_focused, app_focus),
                self.get_size_progress_cell(
                    data.size,
                    parent.size,
                    show_bar,
                    is_focused,
                    app_focus,
                ),
                self.get_size_cell(Byte::from_u64(data.size), is_focused, app_focus),
            ])
            .style(self.get_row_style(is_focused, app_focus))
        });

        let table = Table::default()
            .rows(rows)
            .widths([
                Constraint::Length(1),
                Constraint::Min(10),
                // + 2 for padding
                Constraint::Length(BAR_SIZE as u16 + 2),
                // + 3 for padding (example: 123.45 KB)
                Constraint::Length(12),
            ])
            .block(block);

        frame.render_stateful_widget(table, area, &mut table_state.state);
    }

    fn get_row_style(&self, is_focused: bool, app_focus: &AppFocus) -> Style {
        match app_focus {
            AppFocus::MainScreen | AppFocus::BufferingInput if is_focused => {
                Style::default().bg(self.colors.primary)
            }
            _ => Style::default(),
        }
    }

    fn get_selection_cell<'a>(&self, is_selected: bool) -> Cell<'a> {
        Cell::from(Span::styled(
            if is_selected { "▌" } else { "" },
            Style::default().fg(self.colors.tertiary),
        ))
    }

    fn get_name_cell<'a>(&self, name: String, is_focused: bool, app_focus: &AppFocus) -> Cell<'a> {
        let style = match app_focus {
            AppFocus::MainScreen | AppFocus::BufferingInput if is_focused => {
                Style::default().fg(self.colors.primary_bg)
            }
            _ => Style::default(),
        };

        Cell::from(Text::from(name)).style(style)
    }

    fn get_size_progress_cell<'a>(
        &self,
        size: u64,
        total_size: u64,
        show_bar: bool,
        is_focused: bool,
        app_focus: &AppFocus,
    ) -> Cell<'a> {
        let rate = (size as f64 / total_size as f64).min(1.0);
        let filled = (rate * BAR_SIZE as f64) as usize;
        let empty = BAR_SIZE - filled;

        let red = (rate * 255.0) as u8;
        let green = 255 - (rate * 255.0) as u8;
        let blue = 50 - (rate * 50.0) as u8;
        let color = Color::Rgb(red, green, blue);

        let fg = match app_focus {
            AppFocus::MainScreen | AppFocus::BufferingInput if is_focused => self.colors.primary_bg,
            _ => color,
        };

        if show_bar {
            Cell::from(Line::from(vec![
                Span::from("\u{25AC}".repeat(filled)).set_style(Style::default().fg(color)),
                Span::from("\u{25AC}".repeat(empty))
                    .set_style(Style::default().fg(self.colors.secondary_bg)),
            ]))
        } else {
            Cell::from(Line::from(vec![Span::from(format!(
                "{:.1}%",
                rate * 100.0
            ))
            .set_style(Style::default().fg(fg))]))
        }
    }

    fn get_size_cell<'a>(&self, size: Byte, is_focused: bool, app_focus: &AppFocus) -> Cell<'a> {
        let fg = match app_focus {
            AppFocus::MainScreen | AppFocus::BufferingInput if is_focused => self.colors.primary_bg,
            _ => self.colors.fg,
        };

        let appropriate_size = size.get_appropriate_unit(byte_unit::UnitType::Decimal);

        Cell::from(Line::from(vec![Span::from(format!(
            "{:>10.2}",
            appropriate_size
        ))
        .set_style(Style::default().fg(fg))]))
    }

    #[allow(clippy::too_many_arguments)]
    fn render_preview_table(
        &self,
        frame: &mut Frame,
        area: Rect,
        block: Block<'_>,
        parent: &EntryNodeView,
        table_state: &mut StatefulTable<EntryNodeView>,
        app_focus: &AppFocus,
        show_bar: bool,
    ) {
        self.render_table(frame, area, block, parent, table_state, app_focus, show_bar);
    }

    fn render_preview_empty_directory(&self, frame: &mut Frame, area: Rect, block: Block<'_>) {
        let text = Text::styled(
            "Empty directory",
            Style::new()
                .bg(self.colors.fg)
                .fg(self.colors.primary_bg)
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
