use color_eyre::Result;
use crossterm::{
    cursor, execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{stdout, Write};

/// Set up the terminal with robust cursor handling and safer state transitions
pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>> {
    // Minimal terminal environment check
    let _size = crossterm::terminal::size().unwrap_or((80, 24));
    // STEP 1: Enable raw mode - simplest operation that modifies terminal state
    if let Err(e) = enable_raw_mode() {
        eprintln!("Failed to enable raw mode: {e}");
        return Err(color_eyre::eyre::eyre!("Failed to enable raw mode: {e}"));
    }

    // STEP 2: Enter alternate screen - create a clean environment
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
    Ok(terminal)
}

/// Restore terminal to initial state with simplified cursor handling
/// Clean up terminal state, handling any errors
pub fn cleanup_terminal_state(raw_mode: bool, alternate_screen: bool) {
    // Create a new stdout handle each time to avoid borrowing issues
    let mut stdout_handle = stdout();

    // Always try to show cursor first (works in both normal and alternate screen)
    let _ = execute!(stdout_handle, cursor::Show);

    // Leave alternate screen if we entered it
    if alternate_screen {
        let _ = execute!(stdout_handle, LeaveAlternateScreen);
    }

    // Disable raw mode if we enabled it
    if raw_mode {
        let _ = disable_raw_mode();
    }

    // Force a newline to ensure the prompt appears correctly
    let _ = execute!(stdout_handle, cursor::MoveToNextLine(1));

    // Flush the output to ensure all commands are processed
    let _ = stdout_handle.flush();
}
