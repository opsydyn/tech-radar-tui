use crate::app::state::{App, AppScreen};
use crossterm::event::KeyCode;

mod blip_actions;
mod blip_details;
mod blips;
mod edit_blip;
mod help;
mod main;
mod adrs;

pub async fn dispatch_input(app: &mut App, key: KeyCode) {
    if help::handle_help_toggle(app, key) {
        return;
    }

    if help::handle_animation_toggle(app, key) {
        return;
    }

    match app.screen {
        AppScreen::ViewBlips => blips::handle_view_blips_input(app, key),
        AppScreen::BlipActions => blip_actions::handle_blip_actions_input(app, key).await,
        AppScreen::ViewAdrs => adrs::handle_view_adrs_input(app, key),
        AppScreen::BlipDetails => blip_details::handle_blip_details_input(app, key),
        AppScreen::EditBlip => edit_blip::handle_edit_blip_input(app, key).await,
        AppScreen::Main => main::handle_main_input(app, key).await,
    }
}
