use std::{io, panic};

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use super::{
    app::{AppState, CrosstermTerminal},
    event_handling::EventHandler,
    render,
};

/// Representation of a terminal user interface.
///
/// It is responsible for setting up the terminal, event handler,
/// initializing the interface and handling the draw events.
pub struct Tui {
    /// Interface to the Terminal.
    pub terminal: CrosstermTerminal,
    /// Terminal event handler.
    pub events: EventHandler,
}

impl Tui {
    /// Constructs a new instance of [`Tui`].
    pub fn new(terminal: CrosstermTerminal, events: EventHandler) -> Self {
        Self { terminal, events }
    }

    /// Initializes the terminal interface.
    ///
    /// It enables the raw mode, sets terminal properties,
    /// and starts the event handler.
    pub fn enter(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

        // Define a custom panic hook to reset the terminal properties.
        // This way, you won't have your terminal messed up if an unexpected error happens.
        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic| {
            Self::reset().expect("Failed to reset the terminal.");
            panic_hook(panic);
        }));

        self.terminal.hide_cursor()?;
        self.terminal.clear()?;

        self.events.start()?;
        Ok(())
    }

    // Resets the terminal interface.
    ///
    /// This function is also used for the panic hook to revert
    /// the terminal properties if unexpected errors occur.
    fn reset() -> Result<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture)?;
        Ok(())
    }

    /// Exits the terminal interface.
    ///
    /// It disables the raw mode, reverts back the terminal properties,
    /// and stops the event handler.
    pub fn exit(&mut self) -> Result<()> {
        self.events.stop()?;

        Self::reset()?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    /// Draw the terminal interface by rendering the widgets.
    pub fn draw(&mut self, state: &mut AppState) -> Result<()> {
        self.terminal.draw(|frame| render::render(state, frame))?;
        Ok(())
    }
}
