mod helpers;
pub mod screens;

use crate::app::state::App;
use crossterm::event::KeyCode;

pub async fn handle_input(app: &mut App, key: KeyCode) {
    screens::dispatch_input(app, key).await;
}
