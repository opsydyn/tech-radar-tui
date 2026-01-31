use crate::app::state::{App, AppScreen};
use crossterm::event::KeyCode;

use crate::app::input::helpers::{wrap_decrement, wrap_increment};

mod adr_actions;
mod adr_details;
mod adrs;
mod blip_actions;
mod blip_details;
mod blips;
pub mod edit_adr;
mod edit_blip;
mod help;
mod main;

pub async fn dispatch_input(app: &mut App, key: KeyCode) -> color_eyre::Result<()> {
    if app.show_help {
        if help::handle_help_toggle(app, key) {
            return Ok(());
        }
        handle_settings_input(app, key).await?;
        return Ok(());
    }

    if help::handle_help_toggle(app, key) {
        return Ok(());
    }

    if help::handle_animation_toggle(app, key) {
        return Ok(());
    }

    match app.screen {
        AppScreen::ViewBlips => blips::handle_view_blips_input(app, key),
        AppScreen::BlipActions => blip_actions::handle_blip_actions_input(app, key).await,
        AppScreen::ViewAdrs => adrs::handle_view_adrs_input(app, key),
        AppScreen::AdrActions => adr_actions::handle_adr_actions_input(app, key),
        AppScreen::AdrDetails => adr_details::handle_adr_details_input(app, key),
        AppScreen::EditAdr => edit_adr::handle_edit_adr_input(app, key).await,
        AppScreen::BlipDetails => blip_details::handle_blip_details_input(app, key),
        AppScreen::EditBlip => edit_blip::handle_edit_blip_input(app, key).await,
        AppScreen::Main => main::handle_main_input(app, key).await,
    }

    Ok(())
}

async fn handle_settings_input(app: &mut App, key: KeyCode) -> color_eyre::Result<()> {
    if app.settings_editing {
        match key {
            KeyCode::Esc => {
                app.settings_editing = false;
                app.settings_input.clear();
            }
            KeyCode::Enter => {
                apply_settings_value(app);
                app.persist_settings().await?;
                app.settings_editing = false;
                app.settings_input.clear();
                app.status_message = "Settings saved".to_string();
            }
            KeyCode::Backspace => {
                app.settings_input.pop();
            }
            KeyCode::Char(ch) => {
                app.settings_input.push(ch);
            }
            _ => {}
        }
        return Ok(());
    }

    match key {
        KeyCode::Up => {
            app.settings_selection_index = wrap_decrement(app.settings_selection_index, 3);
        }
        KeyCode::Down => {
            app.settings_selection_index = wrap_increment(app.settings_selection_index, 3);
        }
        KeyCode::Enter => {
            app.settings_editing = true;
            app.settings_input = match app.settings_selection_index {
                0 => app.settings_adr_dir.clone(),
                1 => app.settings_blip_dir.clone(),
                2 => app.settings_db_name.clone(),
                _ => String::new(),
            };
        }
        _ => {}
    }

    Ok(())
}

fn apply_settings_value(app: &mut App) {
    match app.settings_selection_index {
        0 => app.settings_adr_dir = app.settings_input.trim().to_string(),
        1 => app.settings_blip_dir = app.settings_input.trim().to_string(),
        2 => app.settings_db_name = app.settings_input.trim().to_string(),
        _ => {}
    }
    app.apply_settings_runtime();
}
