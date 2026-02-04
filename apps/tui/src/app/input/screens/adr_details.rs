use crate::app::state::{App, AppScreen};
use crossterm::event::KeyCode;

#[allow(clippy::missing_const_for_fn)]
pub fn handle_adr_details_input(app: &mut App, key: KeyCode) {
    if key == KeyCode::Esc {
        if app.search_active {
            app.clear_search();
        }
        app.screen = AppScreen::AdrActions;
        return;
    }

    if app.search_active {
        match key {
            KeyCode::Char(ch) => {
                app.search_query.push(ch);
                app.apply_search_filter();
            }
            KeyCode::Backspace => {
                app.search_query.pop();
                app.apply_search_filter();
            }
            KeyCode::Enter => {
                app.search_active = false;
                app.apply_search_filter();
                app.status_message = "Search applied".to_string();
            }
            _ => {}
        }
    }
}
