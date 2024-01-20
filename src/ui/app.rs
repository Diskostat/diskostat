use std::{fs, path::PathBuf, sync::mpsc};

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
    EnterFocusedDirectory,
    EnterParentDirectory,
    Resize(u16, u16),
    Quit,
}

/// Possible application preview states.
pub enum Preview {
    Text(String),
    Table(StatefulTable<PathBuf>),
}

/// Application state.
pub struct AppState {
    pub should_quit: bool,
    pub parent_dir: Option<PathBuf>,
    pub main_table: StatefulTable<PathBuf>,
    pub preview: Preview,
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

        let parent = PathBuf::from(".");
        let paths = fs::read_dir(&parent)?
            .map(|file| file.unwrap().path())
            .collect::<Vec<PathBuf>>();

        let first = paths.first().unwrap();
        let preview = Self::get_preview(first)?;

        let tui = Tui::new(terminal, events);
        let state = AppState {
            should_quit: false,
            parent_dir: Some(parent),
            main_table: StatefulTable::with_focused(paths, Some(0)),
            preview,
        };

        Ok(Self { state, tui })
    }

    fn get_preview(path: &PathBuf) -> Result<Preview> {
        if path.is_dir() {
            Ok(Preview::Table(StatefulTable::with_items(Self::get_paths(
                path,
            )?)))
        } else {
            Ok(Preview::Text(fs::read_to_string(path)?))
        }
    }

    fn get_paths(path: &PathBuf) -> Result<Vec<PathBuf>> {
        if !path.is_dir() {
            return Ok(Vec::new());
        }

        Ok(fs::read_dir(path)?
            .map(|file| file.unwrap().path())
            .collect::<Vec<PathBuf>>())
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

    fn update_focus(&mut self) -> Result<()> {
        let focused = self.state.main_table.focused().unwrap().clone();
        self.state.preview = Self::get_preview(&focused)?;
        Ok(())
    }

    /// Handle the application actions.
    pub fn update(&mut self, action: Option<Action>) -> Result<()> {
        if let Some(action) = action {
            match action {
                Action::Tick => self.tick(),
                Action::Quit => self.quit(),
                Action::FocusNextItem => {
                    self.state.main_table.focus_next();
                    self.update_focus()?;
                }
                Action::FocusPreviousItem => {
                    self.state.main_table.focus_previous();
                    self.update_focus()?;
                }
                Action::FocusFirstItem => {
                    self.state.main_table.focus_first();
                    self.update_focus()?;
                }
                Action::FocusLastItem => {
                    self.state.main_table.focus_last();
                    self.update_focus()?;
                }
                Action::EnterFocusedDirectory => (),
                Action::EnterParentDirectory => (),
                Action::Resize(w, h) => self.resize(w, h)?,
            }
        }
        Ok(())
    }
}
