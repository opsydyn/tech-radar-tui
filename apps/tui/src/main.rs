mod app;
mod config;
mod db;
mod event;
mod terminal;
mod ui;

use app::App;
use color_eyre::Result;

#[derive(Debug, Clone, Copy)]
enum Quadrant {
    Platforms,
    Languages,
    Tools,
    Techniques,
}

impl Quadrant {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Platforms => "platforms",
            Self::Languages => "languages",
            Self::Tools => "tools",
            Self::Techniques => "techniques",
        }
    }

    const fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Platforms),
            1 => Some(Self::Languages),
            2 => Some(Self::Tools),
            3 => Some(Self::Techniques),
            _ => None,
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Platforms => "Platforms",
            Self::Languages => "Languages",
            Self::Tools => "Tools",
            Self::Techniques => "Techniques",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Ring {
    Hold,
    Assess,
    Trial,
    Adopt,
}

impl Ring {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Hold => "hold",
            Self::Assess => "assess",
            Self::Trial => "trial",
            Self::Adopt => "adopt",
        }
    }

    const fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Hold),
            1 => Some(Self::Assess),
            2 => Some(Self::Trial),
            3 => Some(Self::Adopt),
            _ => None,
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Hold => "Hold",
            Self::Assess => "Assess",
            Self::Trial => "Trial",
            Self::Adopt => "Adopt",
        }
    }
}

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
