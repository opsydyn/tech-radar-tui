mod helpers;
pub mod screens;

use crate::app::state::App;
use crossterm::event::KeyCode;

pub async fn handle_input(app: &mut App, key: KeyCode) {
    if let Err(error) = screens::dispatch_input(app, key).await {
        app.status_message = format!("Settings error: {error}");
    }
}
