// Event module for ratatui_adr-gen
// Handles event loop and event processing

mod loop_handler;

pub use loop_handler::{run, run_headless};
