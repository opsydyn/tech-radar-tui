use crate::app::input::helpers::{wrap_decrement, wrap_increment};
use crate::app::state::{App, AppScreen, InputMode, InputState};
use crossterm::event::KeyCode;

pub async fn handle_blip_actions_input(app: &mut App, key: KeyCode) {
    if app.search_active {
        match key {
            KeyCode::Esc => {
                app.clear_search();
                app.screen = AppScreen::ViewBlips;
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
            app.blip_action_index = wrap_decrement(app.blip_action_index, 4);
        }
        KeyCode::Down => {
            app.blip_action_index = wrap_increment(app.blip_action_index, 4);
        }
        KeyCode::Enter => match app.blip_action_index {
            0 => {
                app.screen = AppScreen::BlipDetails;
            }
            1 => {
                handle_adr_action(app).await;
            }
            2 => {
                let blip = if app.filtered_blip_indices.is_empty() {
                    app.blips.get(app.selected_blip_index)
                } else {
                    app.filtered_blip_indices
                        .get(app.selected_blip_index)
                        .and_then(|index| app.blips.get(*index))
                };
                if let Some(blip) = blip {
                    app.edit_blip_state = Some(crate::app::state::EditBlipState::from_blip(blip));
                    app.screen = AppScreen::EditBlip;
                }
            }
            _ => {
                app.screen = AppScreen::ViewBlips;
            }
        },
        KeyCode::Esc => {
            app.screen = AppScreen::ViewBlips;
        }
        _ => {}
    }
}

async fn handle_adr_action(app: &mut App) {
    let blip = if app.filtered_blip_indices.is_empty() {
        app.blips.get(app.selected_blip_index)
    } else {
        app.filtered_blip_indices
            .get(app.selected_blip_index)
            .and_then(|index| app.blips.get(*index))
    };
    let Some(blip) = blip else {
        app.screen = AppScreen::ViewBlips;
        return;
    };

    let blip_name = blip.name.clone();

    if blip.has_adr {
        match app.fetch_adrs_for_blip(&blip_name).await {
            Ok(()) => {
                app.selected_adr_index = 0;
                app.screen = AppScreen::ViewAdrs;
            }
            Err(e) => {
                app.status_message = format!("Failed to fetch ADRs for blip: {e}");
                app.screen = AppScreen::ViewBlips;
            }
        }
        return;
    }

    app.input_mode = Some(InputMode::Adr);
    app.blip_data.name = blip_name;
    app.blip_data.quadrant = blip.quadrant;
    app.blip_data.ring = blip.ring;

    app.adr_status_selection_index = 0;
    app.adr_status = Some(crate::app::state::AdrStatus::Proposed);
    app.input_state = InputState::ChoosingAdrStatus;
    app.screen = AppScreen::Main;
    app.status_message = "Select ADR status".to_string();
}
