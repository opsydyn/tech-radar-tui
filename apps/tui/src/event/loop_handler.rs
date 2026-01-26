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
                eprintln!("Starting file generation...");
                app.status_message = "Generating file...".to_string();
                Ok(FileGenState::Generating.next_state())
            }
            (FileGenState::Generating, FileGenEvent::Success(path)) => {
                // Extract just the filename for a cleaner status message
                let filename = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("unknown");

                eprintln!("File generated successfully: {filename}");
                app.status_message = format!("File generated: {filename}");
                app.input_state = InputState::Completed;
                Ok(FileGenState::Success.next_state())
            }
            (FileGenState::Generating, FileGenEvent::Error(error)) => {
                eprintln!("Error generating file: {error}");
                app.status_message = format!("Error: {error}");
                app.input_state = InputState::ChoosingRing;
                Ok(FileGenState::Error.next_state())
            }
            (FileGenState::Success | FileGenState::Error, FileGenEvent::Reset) => {
                eprintln!("Resetting file generation state");
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
pub async fn run_headless(app: &mut App) -> Result<()> {
    eprintln!("Running in headless mode...");

    // Initialize database
    eprintln!("Initializing database...");
    app.initialize_db().await?;

    eprintln!("Headless mode is limited to database initialization only.");
    eprintln!("To use full TUI features, run in an interactive terminal.");

    Ok(())
}

/// Run the main application event loop
pub async fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>, app: &mut App) -> Result<()> {
    // Configure event poll timeout (ms)
    const EVENT_POLL_TIMEOUT: u64 = 50;

    // Create our file generation state machine
    let mut file_gen_machine = FileGenMachine::new(FileGenState::Idle);

    eprintln!("Entering application main loop");

    loop {
        // Update animations
        app.update();

        // Draw the UI with better error context
        if let Err(e) = terminal.draw(|f| ui::ui(app, f)) {
            eprintln!("Error drawing terminal UI: {e}");
            eprintln!("Error details: {e:?}");
            return Err(color_eyre::eyre::eyre!("Terminal draw error: {e}"));
        }

        // Handle events with improved error context
        match event::poll(std::time::Duration::from_millis(EVENT_POLL_TIMEOUT)) {
            Ok(true) => {
                // Event is available
                match event::read() {
                    Ok(Event::Key(key)) => {
                        // Remove or conditionally log key events
                        // eprintln!("Key event received: {:?}", key.code);
                        handle_input(app, key.code).await;
                        if !app.running {
                            eprintln!("App requested exit");
                            break;
                        }
                    }
                    Ok(Event::Resize(width, height)) => {
                        eprintln!("Terminal resized to {width}x{height}");
                        // Force a redraw after resize
                        if let Err(e) = terminal.draw(|f| ui::ui(app, f)) {
                            eprintln!("Error redrawing after resize: {e}");
                        }
                    }
                    Ok(
                        Event::Mouse(_) | Event::FocusGained | Event::FocusLost | Event::Paste(_),
                    ) => {
                        // Ignore non-key events for now
                    }
                    Err(e) => {
                        eprintln!("Error reading event: {e}");
                        // Non-fatal error, continue the loop
                    }
                }
            }
            Ok(false) => {
                // No event available, continue with application logic
            }
            Err(e) => {
                eprintln!("Error polling for events: {e}");
                // Non-fatal error, continue the loop
            }
        }

        // Handle file generation with state machine
        if app.input_state == InputState::GeneratingFile
            && file_gen_machine.state() == FileGenState::Idle
        {
            // Transition to generating state
            if let Err(e) = file_gen_machine.process_event(&FileGenEvent::StartGeneration, app) {
                eprintln!("Error starting file generation: {e}");
                continue;
            }

            // Generate file in a controlled async context
            match app.generate_file().await {
                Ok(path) => {
                    // Transition to success state
                    if let Err(e) =
                        file_gen_machine.process_event(&FileGenEvent::Success(path), app)
                    {
                        eprintln!("Error processing success event: {e}");
                    }
                }
                Err(e) => {
                    // Transition to error state
                    let error_msg = format!("{e}");
                    if let Err(e) =
                        file_gen_machine.process_event(&FileGenEvent::Error(error_msg), app)
                    {
                        eprintln!("Error processing error event: {e}");
                    }
                }
            }

            // Reset the state machine for next use
            if let Err(e) = file_gen_machine.process_event(&FileGenEvent::Reset, app) {
                eprintln!("Error resetting file generation state: {e}");
            }

            // Force a redraw to show the updated state
            if let Err(e) = terminal.draw(|f| ui::ui(app, f)) {
                eprintln!("Error redrawing UI after file generation: {e}");
            }
        }
    }
    eprintln!("Exiting application main loop");
    Ok(())
}
