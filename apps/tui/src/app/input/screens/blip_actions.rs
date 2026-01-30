use crate::app::input::helpers::{wrap_decrement, wrap_increment};
use crate::app::state::{App, AppScreen, InputMode, InputState};
use crossterm::event::KeyCode;

pub async fn handle_blip_actions_input(app: &mut App, key: KeyCode) {
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
                if let Some(blip) = app.blips.get(app.selected_blip_index) {
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
    let Some(blip) = app.blips.get(app.selected_blip_index) else {
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
