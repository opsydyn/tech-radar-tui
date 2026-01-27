mod app;
mod config;
mod db;
mod domain;
mod event;
mod terminal;
mod ui;

use app::App;
use color_eyre::Result;

pub use domain::{Quadrant, Ring};

#[tokio::main]
async fn main() -> Result<()> {
    // Setup error handling
    color_eyre::install()?;

    // Initialize application state
    let mut app = App::new();

    // Check if we're running in a terminal
    if !is_terminal() {
        // Run in headless mode
        return event::run_headless(&mut app).await;
    }

    // Initialize database
    if let Err(e) = app.initialize_db().await {
        eprintln!("Error initializing database: {e}");
        eprintln!("Will continue with limited functionality");
    } else {
        eprintln!("Database initialization successful");
    }

    // Setup terminal
    let mut terminal = terminal::setup()?;

    // Run the application
    let result = event::run(&mut terminal, &mut app).await;

    // Restore terminal
    terminal::cleanup(true, true);

    // Return the result
    result
}

// Check if we're running in a terminal
fn is_terminal() -> bool {
    atty::is(atty::Stream::Stdout)
}
