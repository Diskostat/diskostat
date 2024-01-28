#[allow(dead_code)]
mod backend;
/// The front end of the application.
pub mod ui;

use std::path::PathBuf;

use anyhow::{bail, Result};
use ui::app::App;

use clap::Parser;

use crate::backend::disko_tree::DiskoTree;

const DEFAULT_RENDER_RATE: u64 = 30;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Arguments {
    /// The path to the directory to be analyzed
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Do not open a terminal UI, only print the summary
    #[arg(short, long)]
    summary: bool,

    /// The number of threads to use for the file system traversal.
    #[arg(short = 'n', long, default_value_t = 4)]
    threads: usize,

    /// How many times the application ticks per second.
    #[arg(short, long, default_value_t = 4)]
    tick_rate: u64,
}

fn main() -> Result<()> {
    let mut arguments = Arguments::parse();

    arguments.path = dunce::canonicalize(arguments.path)?;

    if !arguments.path.is_dir() {
        bail!("{} is not a directory", arguments.path.display());
    }

    if arguments.threads == 0 {
        bail!("threads must be greater than 0");
    }

    let mut tree = DiskoTree::new(arguments.path, arguments.threads);

    if arguments.summary {
        tree.traverse();
        println!("{tree}");
        return Ok(());
    }

    // Create and start the application.
    let mut app = App::new(arguments.tick_rate, DEFAULT_RENDER_RATE, tree)?;
    app.run()?;
    Ok(())
}
