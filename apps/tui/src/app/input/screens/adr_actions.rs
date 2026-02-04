use crate::app::input::screens::edit_adr::{AdrEditField, AdrEditState};
use crate::app::state::{AdrStatus, App, AppScreen};
use crossterm::event::KeyCode;

#[allow(clippy::missing_const_for_fn)]
pub fn handle_adr_actions_input(app: &mut App, key: KeyCode) {
    if app.search_active {
        match key {
            KeyCode::Esc => {
                app.clear_search();
                app.screen = AppScreen::ViewAdrs;
            }
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
        return;
    }

    match key {
        KeyCode::Up => {
            app.adr_action_index = app.adr_action_index.saturating_sub(1);
        }
        KeyCode::Down => {
            let next = app.adr_action_index + 1;
            app.adr_action_index = if next > 2 { 0 } else { next };
        }
        KeyCode::Enter => match app.adr_action_index {
            0 => {
                app.screen = AppScreen::AdrDetails;
            }
            1 => {
                let adr = if app.filtered_adr_indices.is_empty() {
                    app.adrs.get(app.selected_adr_index)
                } else {
                    app.filtered_adr_indices
                        .get(app.selected_adr_index)
                        .and_then(|index| app.adrs.get(*index))
                };
                if let Some(adr) = adr {
                    let status = AdrStatus::parse(&adr.status).unwrap_or(AdrStatus::Proposed);
                    app.edit_adr_state = Some(AdrEditState {
                        id: adr.id,
                        field: AdrEditField::Title,
                        title: adr.title.clone(),
                        status,
                        editing: false,
                    });
                    app.screen = AppScreen::EditAdr;
                }
            }
            _ => {
                app.screen = AppScreen::ViewAdrs;
            }
        },
        KeyCode::Esc => {
            app.screen = AppScreen::ViewAdrs;
        }
        _ => {}
    }
}
