// UI module for ratatui_adr-gen
// Handles all UI rendering functions

pub mod screens;
pub mod widgets;

use crate::app::state::AppScreen;
use crate::app::App;
use ratatui::Frame;

pub fn ui(app: &App, f: &mut Frame<'_>) {
    match app.screen {
        AppScreen::Main => screens::main::render_main(app, f),
        AppScreen::ViewBlips => screens::blips::render_blips_view(app, f),
        AppScreen::ViewAdrs => screens::adrs::render_adrs_view(app, f),
        AppScreen::BlipActions => screens::blip_actions::render_blip_actions(app, f),
        AppScreen::BlipDetails => screens::blip_details::render_blip_details(app, f),
        AppScreen::EditBlip => screens::edit_blip::render_edit_blip(app, f),
    }
}
