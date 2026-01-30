use color_eyre::Result;
use crossterm::event::{self, Event};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::convert::TryFrom;
use std::fmt;
use std::io::Stdout;
use std::path::PathBuf;

use crate::app::{handle_input, App, InputState};
use crate::ui;

// Define states for file generation
#[derive(Clone, Copy, PartialEq, Debug)]
enum FileGenState {
    Idle,
    Generating,
    Success,
    Error,
}

impl fmt::Display for FileGenState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Idle => write!(f, "Idle"),
            Self::Generating => write!(f, "Generating"),
            Self::Success => write!(f, "Success"),
            Self::Error => write!(f, "Error"),
        }
    }
}

// Define events for file generation
#[derive(Clone, Debug)]
enum FileGenEvent {
    StartGeneration,
    Success(PathBuf),
    Error(String),
    Reset,
}

impl fmt::Display for FileGenEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StartGeneration => write!(f, "StartGeneration"),
            Self::Success(path) => write!(f, "Success({path})", path = path.display()),
            Self::Error(msg) => write!(f, "Error({msg})"),
            Self::Reset => write!(f, "Reset"),
        }
    }
}

// Define a custom error type for state transitions
#[derive(Debug)]
struct StateTransitionError {
    from: FileGenState,
    event: FileGenEvent,
}

impl fmt::Display for StateTransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Invalid transition from {} with event {}",
            self.from, self.event
        )
    }
}

impl std::error::Error for StateTransitionError {}

// Create a custom state machine for file generation
struct FileGenMachine {
    state: FileGenState,
}

impl FileGenMachine {
    const fn new(initial_state: FileGenState) -> Self {
        Self {
            state: initial_state,
        }
    }

    const fn state(&self) -> FileGenState {
        self.state
    }

    // Process an event and update the state machine and app
    fn process_event(
        &mut self,
        event: &FileGenEvent,
        app: &mut App,
    ) -> std::result::Result<(), StateTransitionError> {
        // Calculate the next state based on current state and event
        let next_state = NextState::try_from((self.state, event, app))?;

        // Update the state machine
        self.state = next_state.0;

        Ok(())
    }
}

// Helper struct for state transitions
struct NextState(FileGenState);

impl NextState {
    const fn new(state: FileGenState) -> Self {
        Self(state)
    }
}

impl FileGenState {
    const fn next_state(self) -> NextState {
        NextState::new(self)
    }
}

impl TryFrom<(FileGenState, &FileGenEvent, &mut App)> for NextState {
    type Error = StateTransitionError;

    fn try_from(
        value: (FileGenState, &FileGenEvent, &mut App),
    ) -> std::result::Result<Self, Self::Error> {
        let (current_state, event, app) = value;

        match (current_state, event) {
            (FileGenState::Idle, FileGenEvent::StartGeneration) => {
                app.status_message = "Generating file...".to_string();
                Ok(FileGenState::Generating.next_state())
            }
            (FileGenState::Generating, FileGenEvent::Success(path)) => {
                // Extract just the filename for a cleaner status message
                let filename = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown");

                app.status_message = format!("File generated: {filename}");
                app.input_state = InputState::Completed;
                Ok(FileGenState::Success.next_state())
            }
            (FileGenState::Generating, FileGenEvent::Error(error)) => {
                app.status_message = format!("Error: {error}");

                if error.contains("already exists") {
                    app.input_state = InputState::EnteringTechnology;
                    app.current_input = app.blip_data.name.clone();
                } else {
                    app.input_state = InputState::ChoosingRing;
                }

                Ok(FileGenState::Error.next_state())
            }
            (FileGenState::Success | FileGenState::Error, FileGenEvent::Reset) => {
                Ok(FileGenState::Idle.next_state())
            }
            _ => Err(StateTransitionError {
                from: current_state,
                event: event.clone(),
            }),
        }
    }
}

/// Run the application in headless mode (no UI)
pub async fn run_headless(app: &mut App, json: bool) -> Result<()> {
    // Initialize database
    app.initialize_db().await?;

    if json {
        render_headless_json(app).await?;
    } else {
        render_headless_stats(app).await?;
    }

    Ok(())
}

async fn render_headless_stats(app: &App) -> Result<()> {
    let stats = build_headless_stats(app).await?;

    println!("\nTech Radar Stats");
    println!("=================");
    println!("Total blips: {}", stats.total_blips);
    println!("Total ADRs: {}", stats.total_adrs);

    if let Some(coverage) = stats.adr_coverage {
        println!("ADR coverage: {coverage:.1}%");
    }

    println!("\nBlips by Quadrant:");
    for (quadrant, count) in stats.by_quadrant {
        println!("- {quadrant}: {count}");
    }

    println!("\nBlips by Ring:");
    for (ring, count) in stats.by_ring {
        println!("- {ring}: {count}");
    }

    println!("\nRecent Blips:");
    for blip in stats.recent_blips {
        println!(
            "- {} | {} | {} | {}",
            blip.name, blip.quadrant, blip.ring, blip.created
        );
    }

    Ok(())
}

async fn render_headless_json(app: &App) -> Result<()> {
    let stats = build_headless_stats(app).await?;
    let json = serde_json::to_string_pretty(&stats)?;
    println!("{json}");
    Ok(())
}

async fn build_headless_stats(app: &App) -> Result<HeadlessStats> {
    let total_blips = app.actions.count_blips().await?;
    let total_adrs = app.actions.count_adrs().await?;
    let by_quadrant = app.actions.count_blips_by_quadrant().await?;
    let by_ring = app.actions.count_blips_by_ring().await?;
    let recent = app.actions.recent_blips(5).await?;

    let adr_coverage = if total_blips > 0 {
        #[allow(clippy::cast_precision_loss)]
        Some((total_adrs as f64 / total_blips as f64) * 100.0)
    } else {
        None
    };

    let by_quadrant = by_quadrant
        .into_iter()
        .map(|(quadrant, count)| (quadrant.as_str().to_string(), count))
        .collect();

    let by_ring = by_ring
        .into_iter()
        .map(|(ring, count)| (ring.as_str().to_string(), count))
        .collect();

    let recent_blips = recent
        .into_iter()
        .map(|blip| {
            let ring = blip
                .ring
                .map_or_else(|| "(none)".to_string(), |ring| ring.as_str().to_string());
            let quadrant = blip.quadrant.map_or_else(
                || "(none)".to_string(),
                |quadrant| quadrant.as_str().to_string(),
            );
            HeadlessBlip {
                name: blip.name,
                quadrant,
                ring,
                created: blip.created,
            }
        })
        .collect();

    Ok(HeadlessStats {
        total_blips,
        total_adrs,
        adr_coverage,
        by_quadrant,
        by_ring,
        recent_blips,
    })
}

#[derive(serde::Serialize)]
struct HeadlessStats {
    total_blips: i64,
    total_adrs: i64,
    adr_coverage: Option<f64>,
    by_quadrant: Vec<(String, i64)>,
    by_ring: Vec<(String, i64)>,
    recent_blips: Vec<HeadlessBlip>,
}

#[derive(serde::Serialize)]
struct HeadlessBlip {
    name: String,
    quadrant: String,
    ring: String,
    created: String,
}

/// Run the main application event loop
pub async fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> Result<()> {
    // Configure event poll timeout (ms)
    const EVENT_POLL_TIMEOUT: u64 = 50;

    // Create our file generation state machine
    let mut file_gen_machine = FileGenMachine::new(FileGenState::Idle);

    loop {
        // Update animations
        app.update();

        // Draw the UI with better error context
        if let Err(e) = terminal.draw(|f| ui::ui(app, f)) {
            return Err(color_eyre::eyre::eyre!("Terminal draw error: {e}"));
        }

        // Handle events with improved error context
        if matches!(
            event::poll(std::time::Duration::from_millis(EVENT_POLL_TIMEOUT)),
            Ok(true)
        ) {
            match event::read() {
                Ok(Event::Key(key)) => {
                    handle_input(app, key.code).await;
                    if !app.running {
                        break;
                    }
                }
                Ok(Event::Resize(_, _)) => {
                    // Force a redraw after resize
                    if terminal.draw(|f| ui::ui(app, f)).is_err() {
                        // Non-fatal redraw error
                    }
                }
                Ok(Event::Mouse(_) | Event::FocusGained | Event::FocusLost | Event::Paste(_))
                | Err(_) => {
                    // Ignore non-key events for now
                }
            }
        }

        // Handle file generation with state machine
        if app.input_state == InputState::GeneratingFile
            && file_gen_machine.state() == FileGenState::Idle
        {
            // Transition to generating state
            if file_gen_machine
                .process_event(&FileGenEvent::StartGeneration, app)
                .is_err()
            {
                continue;
            }

            // Generate file in a controlled async context
            match app.generate_file().await {
                Ok(path) => {
                    // Transition to success state
                    if file_gen_machine
                        .process_event(&FileGenEvent::Success(path), app)
                        .is_err()
                    {
                        // Non-fatal state transition error
                    }
                }
                Err(e) => {
                    // Transition to error state
                    let error_msg = format!("{e}");
                    if file_gen_machine
                        .process_event(&FileGenEvent::Error(error_msg), app)
                        .is_err()
                    {
                        // Non-fatal state transition error
                    }
                }
            }

            // Reset the state machine for next use
            if file_gen_machine
                .process_event(&FileGenEvent::Reset, app)
                .is_err()
            {
                // Non-fatal reset error
            }

            // Force a redraw to show the updated state
            if terminal.draw(|f| ui::ui(app, f)).is_err() {
                // Non-fatal redraw error
            }
        }
    }
    Ok(())
}
