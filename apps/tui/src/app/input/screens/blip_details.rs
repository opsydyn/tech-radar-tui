use crate::app::state::{App, AppScreen};
use crossterm::event::KeyCode;

#[allow(clippy::missing_const_for_fn)]
pub fn handle_blip_details_input(app: &mut App, key: KeyCode) {
    if key == KeyCode::Esc {
        app.screen = AppScreen::ViewBlips;
    }
}
