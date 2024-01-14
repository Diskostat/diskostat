use std::sync::mpsc;

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};

use crate::{
    action::Action,
    event::{Event, EventHandler},
    tui::Tui,
    utils::AppResult,
};

pub type CrosstermTerminal = ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stdout>>;

/// Application state
pub struct AppState {
    pub should_quit: bool,
}

/// Application.
pub struct App {
    state: AppState,
    tui: Tui,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(tick_rate: f64, render_rate: f64) -> AppResult<Self> {
        // Initialize the terminal user interface.
        let backend = CrosstermBackend::new(std::io::stdout());
        let terminal = Terminal::new(backend)?;

        let (sender, receiver) = mpsc::channel();
        let events = EventHandler::new(tick_rate, render_rate, sender, receiver);

        let tui = Tui::new(terminal, events);
        let state = AppState { should_quit: false };

        Ok(Self { state, tui })
    }

    /// Runs the main loop of the application.
    pub fn run(&mut self) -> AppResult<()> {
        self.tui.enter()?;

        // Start the main loop.
        while !self.state.should_quit {
            let event = self.tui.events.next()?;

            // Render the user interface.
            if let Event::Init | Event::Render = event {
                self.tui.draw(&mut self.state)?;
            }

            // Handle events.
            let action = Self::get_action(event);

            self.update(action)?;
        }

        // Exit the user interface.
        self.tui.exit()?;
        Ok(())
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {}

    /// Handles the resize event of the terminal.
    pub fn resize(&mut self, width: u16, height: u16) -> AppResult<()> {
        self.tui.terminal.resize(Rect::new(0, 0, width, height))?;
        Ok(())
    }

    /// Set the application to quit.
    pub fn quit(&mut self) {
        self.state.should_quit = true;
    }

    /// Map the terminal event to an application action.
    fn get_action(event: Event) -> Option<Action> {
        let action = match event {
            Event::Tick => Action::Tick,
            Event::Key(key) => match key.code {
                KeyCode::Esc | KeyCode::Char('q') => Action::Quit,
                KeyCode::Char('c') | KeyCode::Char('C')
                    if key.modifiers == KeyModifiers::CONTROL =>
                {
                    Action::Quit
                }
                _ => return None,
            },
            Event::Mouse(_) => return None,
            Event::Resize(w, h) => Action::Resize(w, h),
            _ => return None,
        };
        Some(action)
    }

    /// Handle the application actions.
    pub fn update(&mut self, action: Option<Action>) -> AppResult<()> {
        if let Some(action) = action {
            match action {
                Action::Tick => self.tick(),
                Action::Quit => self.quit(),
                Action::Resize(w, h) => self.resize(w, h)?,
            }
        }
        Ok(())
    }
}
