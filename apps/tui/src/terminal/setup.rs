use color_eyre::Result;
use crossterm::{
    cursor, execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{stdout, Write};

/// Set up the terminal with robust cursor handling and safer state transitions
pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    eprintln!("Setting up terminal...");

    // Minimal terminal environment check
    let size = crossterm::terminal::size().unwrap_or((80, 24));
    let (width, height) = size;
    eprintln!("Terminal size: {width}x{height}");

    // STEP 1: Enable raw mode - simplest operation that modifies terminal state
    eprintln!("Enabling raw mode...");
    if let Err(e) = enable_raw_mode() {
        eprintln!("Failed to enable raw mode: {e}");
        return Err(color_eyre::eyre::eyre!("Failed to enable raw mode: {e}"));
    }

    // STEP 2: Enter alternate screen - create a clean environment
    eprintln!("Entering alternate screen...");
    let mut stdout = stdout();
    if let Err(e) = execute!(stdout, EnterAlternateScreen) {
        // Clean up raw mode
        let _ = disable_raw_mode();
        eprintln!("Failed to enter alternate screen: {e}");
        return Err(color_eyre::eyre::eyre!(
            "Failed to enter alternate screen: {e}"
        ));
    }

    // STEP 3: Create backend and terminal with minimal operations
    eprintln!("Creating terminal...");
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = match Terminal::new(backend) {
        Ok(term) => term,
        Err(e) => {
            // Clean up terminal state
            let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
            let _ = disable_raw_mode();
            eprintln!("Failed to create terminal: {e}");
            return Err(color_eyre::eyre::eyre!("Failed to create terminal: {e}"));
        }
    };

    // STEP 4: Configure terminal appearance (only essential operations)
    // Clear screen and hide cursor - these operations rarely fail
    if let Err(e) = terminal.clear() {
        eprintln!("Warning: Failed to clear terminal: {e}");
        // Not fatal, continue
    }

    // Hide cursor using a new stdout handle
    if let Err(e) = execute!(std::io::stdout(), cursor::Hide) {
        eprintln!("Warning: Failed to hide cursor: {e}");
        // Not fatal, continue
    }

    // Terminal is now successfully initialized
    eprintln!("Terminal setup completed successfully");
    Ok(terminal)
}

/// Restore terminal to initial state with simplified cursor handling
/// Clean up terminal state, handling any errors
pub fn cleanup_terminal_state(raw_mode: bool, alternate_screen: bool) {
    // Create a new stdout handle each time to avoid borrowing issues
    let mut stdout_handle = stdout();

    eprintln!("Cleaning up terminal state...");

    // Always try to show cursor first (works in both normal and alternate screen)
    match execute!(stdout_handle, cursor::Show) {
        Ok(()) => eprintln!("Cursor visibility restored"),
        Err(e) => eprintln!("Warning: Failed to show cursor: {e}"),
    }

    // Leave alternate screen if we entered it
    if alternate_screen {
        match execute!(stdout_handle, LeaveAlternateScreen) {
            Ok(()) => eprintln!("Left alternate screen"),
            Err(e) => eprintln!("Warning: Failed to leave alternate screen: {e}"),
        }
    }

    // Disable raw mode if we enabled it
    if raw_mode {
        match disable_raw_mode() {
            Ok(()) => eprintln!("Disabled raw mode"),
            Err(e) => eprintln!("Warning: Failed to disable raw mode: {e}"),
        }
    }

    // Force a newline to ensure the prompt appears correctly
    let _ = execute!(stdout_handle, cursor::MoveToNextLine(1));

    // Flush the output to ensure all commands are processed
    let _ = stdout_handle.flush();

    eprintln!("Terminal cleanup completed");
}
