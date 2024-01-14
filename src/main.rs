/// Application.
pub mod app;

/// Application actions.
pub mod action;

/// Terminal events handler.
pub mod event;

/// Widget renderer.
pub mod ui;

/// Terminal user interface.
pub mod tui;

/// Utility functions.
pub mod utils;

use app::App;
use utils::AppResult;

const DEFAULT_TICK_RATE: f64 = 4.0;
const DEFAULT_RENDER_RATE: f64 = 30.0;

fn main() -> AppResult<()> {
    // Create and start the application.
    let mut app = App::new(DEFAULT_TICK_RATE, DEFAULT_RENDER_RATE)?;
    app.run()?;
    Ok(())
}
