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

    if has_flag("--help") || has_flag("-h") {
        print_help();
        return Ok(());
    }

    if has_flag("--version") || has_flag("-V") {
        print_version();
        return Ok(());
    }

    apply_overrides()?;

    // Initialize application state
    let mut app = App::new();

    // Check if we're running in a terminal or forced headless mode
    if !is_terminal() || has_flag("--headless") {
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

fn has_flag(flag: &str) -> bool {
    std::env::args().any(|arg| arg == flag)
}

fn print_help() {
    println!("Tech Radar TUI\n");
    println!("USAGE:");
    println!("  ratatui_adr-gen [OPTIONS]\n");
    println!("OPTIONS:");
    println!("  --headless       Print stats and exit");
    println!("  --json           Print headless stats as JSON");
    println!("  --db <path>      Override database path");
    println!("  --adr-dir <path> Override ADR output directory");
    println!("  --blip-dir <path> Override Blip output directory");
    println!("  -h, --help       Print help information");
    println!("  -V, --version    Print version information");
}

fn print_version() {
    println!("ratatui_adr-gen {}", env!("CARGO_PKG_VERSION"));
}

fn apply_overrides() -> Result<()> {
    let mut args = std::env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--db" => {
                if let Some(value) = args.next() {
                    std::env::set_var("DATABASE_NAME", value);
                } else {
                    return Err(color_eyre::eyre::eyre!("--db requires a value"));
                }
            }
            "--adr-dir" => {
                if let Some(value) = args.next() {
                    std::env::set_var("ADR_DIR", value);
                } else {
                    return Err(color_eyre::eyre::eyre!("--adr-dir requires a value"));
                }
            }
            "--blip-dir" => {
                if let Some(value) = args.next() {
                    std::env::set_var("BLIP_DIR", value);
                } else {
                    return Err(color_eyre::eyre::eyre!("--blip-dir requires a value"));
                }
            }
            _ => {}
        }
    }

    Ok(())
}
