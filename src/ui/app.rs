use std::{fs, sync::mpsc};

use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};

use super::{
    components::table::StatefulTable,
    event_handling::{Event, EventHandler},
    key_handling::map_key_events,
    tui::Tui,
};

use anyhow::Result;

pub type CrosstermTerminal = ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>;

/// All possible application actions.
pub enum Action {
    Tick,
    FocusNextItem,
    FocusPreviousItem,
    FocusFirstItem,
    FocusLastItem,
    Resize(u16, u16),
    Quit,
}

/// Application state.
pub struct AppState {
    pub should_quit: bool,
    pub main_table: StatefulTable<String>,
}

/// Application.
pub struct App {
    state: AppState,
    tui: Tui,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(tick_rate: f64, render_rate: f64) -> Result<Self> {
        // Initialize the terminal user interface.
        let backend = CrosstermBackend::new(std::io::stdout());
        let terminal = Terminal::new(backend)?;

        let (sender, receiver) = mpsc::channel();
        let events = EventHandler::new(tick_rate, render_rate, sender, receiver);

        let files = fs::read_dir(".")?
            .map(|file| {
                file.unwrap()
                    .path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
            })
            .collect();

        let tui = Tui::new(terminal, events);
        let state = AppState {
            should_quit: false,
            main_table: StatefulTable::with_selected(files, Some(0)),
        };

        Ok(Self { state, tui })
    }

    /// Runs the main loop of the application.
    pub fn run(&mut self) -> Result<()> {
        self.tui.enter()?;

        // Start the main loop.
        while !self.state.should_quit {
            let event = self.tui.events.next()?;

            // Render the user interface.
            if let Event::Init | Event::Render = event {
                self.tui.draw(&mut self.state)?;
            }

            // Handle events.
            let action = map_key_events(event);

            self.update(action)?;
        }

        // Exit the user interface.
        self.tui.exit()?;
        Ok(())
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {}

    /// Handles the resize event of the terminal.
    pub fn resize(&mut self, width: u16, height: u16) -> Result<()> {
        self.tui.terminal.resize(Rect::new(0, 0, width, height))?;
        Ok(())
    }

    /// Set the application to quit.
    pub fn quit(&mut self) {
        self.state.should_quit = true;
    }

    /// Handle the application actions.
    pub fn update(&mut self, action: Option<Action>) -> Result<()> {
        if let Some(action) = action {
            match action {
                Action::Tick => self.tick(),
                Action::Quit => self.quit(),
                Action::FocusNextItem => self.state.main_table.select_next(),
                Action::FocusPreviousItem => self.state.main_table.select_previous(),
                Action::FocusFirstItem => self.state.main_table.select_first(),
                Action::FocusLastItem => self.state.main_table.select_last(),
                Action::Resize(w, h) => self.resize(w, h)?,
            }
        }
        Ok(())
    }
}
