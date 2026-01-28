// App module for ratatui_adr-gen
// Handles application state and business logic

pub mod actions;
pub mod input;
pub mod state;

pub use input::handle_input;
pub use state::{AdrStatus, App, InputMode, InputState};
