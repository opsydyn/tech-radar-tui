use crate::app::state::{App, AppScreen};
use crossterm::event::KeyCode;

#[allow(clippy::missing_const_for_fn)]
pub fn handle_view_blips_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => {
            app.screen = AppScreen::Main;
        }
        KeyCode::Char('q') => {
            app.running = false;
        }
        KeyCode::Up => {
            if app.selected_blip_index > 0 {
                app.selected_blip_index -= 1;
            }
        }
        KeyCode::Down => {
            if !app.blips.is_empty() && app.selected_blip_index < app.blips.len() - 1 {
                app.selected_blip_index += 1;
            }
        }
        KeyCode::PageUp => {
            if app.selected_blip_index > 0 {
                app.selected_blip_index = app.selected_blip_index.saturating_sub(5);
            }
        }
        KeyCode::PageDown => {
            if !app.blips.is_empty() {
                let new_index = app.selected_blip_index + 5;
                app.selected_blip_index = if new_index >= app.blips.len() {
                    app.blips.len() - 1
                } else {
                    new_index
                };
            }
        }
        KeyCode::Home => {
            app.selected_blip_index = 0;
        }
        KeyCode::End => {
            if !app.blips.is_empty() {
                app.selected_blip_index = app.blips.len() - 1;
            }
        }
        KeyCode::Enter => {
            if !app.blips.is_empty() {
                app.screen = AppScreen::BlipActions;
            }
        }
        _ => {}
    }
}
