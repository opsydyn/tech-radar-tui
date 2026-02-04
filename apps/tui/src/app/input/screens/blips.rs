use crate::app::state::{App, AppScreen};
use crossterm::event::KeyCode;

#[allow(clippy::missing_const_for_fn)]
pub fn handle_view_blips_input(app: &mut App, key: KeyCode) {
    let total_rows = if app.filtered_blip_indices.is_empty() {
        app.blips.len()
    } else {
        app.filtered_blip_indices.len()
    };

    match key {
        KeyCode::Esc => {
            if app.search_active {
                app.clear_search();
            } else {
                app.screen = AppScreen::Main;
            }
        }
        KeyCode::Char('q') => {
            app.running = false;
        }
        KeyCode::Enter => {
            if total_rows > 0 {
                app.screen = AppScreen::BlipActions;
            }
        }
        KeyCode::Up => {
            if app.selected_blip_index > 0 {
                app.selected_blip_index -= 1;
            }
        }
        KeyCode::Down => {
            if total_rows > 0 && app.selected_blip_index + 1 < total_rows {
                app.selected_blip_index += 1;
            }
        }
        KeyCode::PageUp => {
            if app.selected_blip_index > 0 {
                app.selected_blip_index = app.selected_blip_index.saturating_sub(5);
            }
        }
        KeyCode::PageDown => {
            if total_rows > 0 {
                let new_index = app.selected_blip_index + 5;
                app.selected_blip_index = if new_index >= total_rows {
                    total_rows - 1
                } else {
                    new_index
                };
            }
        }
        KeyCode::Home => {
            app.selected_blip_index = 0;
        }
        KeyCode::End => {
            if total_rows > 0 {
                app.selected_blip_index = total_rows - 1;
            }
        }
        _ => {}
    }
}
