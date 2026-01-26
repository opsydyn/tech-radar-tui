// Terminal module for ratatui_adr-gen
// Handles terminal setup and cleanup

pub mod setup;

pub use setup::{cleanup_terminal_state as cleanup, setup_terminal as setup};
