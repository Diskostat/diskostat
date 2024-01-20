/// The front end of the application.
pub mod ui;

use anyhow::Result;
use ui::app::App;

const DEFAULT_TICK_RATE: f64 = 30.0;
const DEFAULT_RENDER_RATE: f64 = 30.0;

fn main() -> Result<()> {
    // Create and start the application.
    let mut app = App::new(DEFAULT_TICK_RATE, DEFAULT_RENDER_RATE)?;
    app.run()?;
    Ok(())
}
