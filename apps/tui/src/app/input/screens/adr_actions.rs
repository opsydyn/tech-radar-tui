use crate::app::input::screens::edit_adr::{AdrEditField, AdrEditState};
use crate::app::state::{AdrStatus, App, AppScreen};
use crossterm::event::KeyCode;

#[allow(clippy::missing_const_for_fn)]
pub fn handle_adr_actions_input(app: &mut App, key: KeyCode) {
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
                if let Some(adr) = app.adrs.get(app.selected_adr_index) {
                    let status = AdrStatus::parse(&adr.status)
                        .unwrap_or(AdrStatus::Proposed);
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
