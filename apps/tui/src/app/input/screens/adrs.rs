use crate::app::state::{App, AppScreen};
use crossterm::event::KeyCode;

#[allow(clippy::missing_const_for_fn)]
pub fn handle_view_adrs_input(app: &mut App, key: KeyCode) {
    let total_rows = if app.filtered_adr_indices.is_empty() {
        app.adrs.len()
    } else {
        app.filtered_adr_indices.len()
    };

    match key {
        KeyCode::Esc => {
            if app.search_active {
                app.clear_search();
            } else {
                app.screen = AppScreen::Main;
            }
        }
        KeyCode::Enter => {
            if total_rows > 0 {
                app.adr_action_index = 0;
                app.screen = AppScreen::AdrActions;
            }
        }
        KeyCode::Up => {
            if app.selected_adr_index > 0 {
                app.selected_adr_index -= 1;
            }
        }
        KeyCode::Down => {
            if total_rows > 0 && app.selected_adr_index + 1 < total_rows {
                app.selected_adr_index += 1;
            }
        }
        _ => {}
    }
}
