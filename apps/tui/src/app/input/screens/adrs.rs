use crate::app::state::{App, AppScreen};
use crossterm::event::KeyCode;

#[allow(clippy::missing_const_for_fn)]
pub fn handle_view_adrs_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => {
            app.screen = AppScreen::Main;
        }
        KeyCode::Up => {
            if app.selected_adr_index > 0 {
                app.selected_adr_index -= 1;
            }
        }
        KeyCode::Down => {
            if !app.adrs.is_empty() && app.selected_adr_index < app.adrs.len() - 1 {
                app.selected_adr_index += 1;
            }
        }
        KeyCode::Enter => {
            if let Some(adr) = app.adrs.get(app.selected_adr_index) {
                app.status_message = format!("ADR: {} | Blip: {}", adr.title, adr.blip_name);
            }
            app.screen = AppScreen::Main;
        }
        _ => {}
    }
}
