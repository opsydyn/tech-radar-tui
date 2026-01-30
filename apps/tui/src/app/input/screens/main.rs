use crate::app::input::helpers::{wrap_decrement, wrap_increment};
use crate::app::state::{AdrStatus, App, AppScreen, InputMode, InputState};
use crate::db::queries::blip_exists_by_name;
use crossterm::event::KeyCode;

pub async fn handle_main_input(app: &mut App, key: KeyCode) {
    match app.input_state {
        InputState::WaitingForCommand => handle_mode_selection(app, key).await,
        InputState::EnteringTechnology => handle_text_input(app, key).await,
        InputState::ChoosingAdrStatus => handle_adr_status_selection(app, key),
        InputState::ChoosingQuadrant => handle_quadrant_selection(app, key),
        InputState::ChoosingRing => handle_ring_selection(app, key),
        InputState::GeneratingFile => {}
        InputState::Completed => match key {
            KeyCode::Char('n') | KeyCode::Esc => {
                app.reset();
            }
            KeyCode::Char('q') => {
                app.running = false;
            }
            KeyCode::Char('l') => {
                if let Err(e) = app.fetch_blips().await {
                    app.status_message = format!("Failed to fetch blips from database: {e}");
                } else {
                    app.selected_blip_index = 0;
                    app.screen = AppScreen::ViewBlips;
                }
            }
            KeyCode::Char('v') => {
                if let Err(e) = app.fetch_adrs_for_blip("").await {
                    app.status_message = format!("Failed to fetch ADRs from database: {e}");
                } else {
                    app.selected_adr_index = 0;
                    app.screen = AppScreen::ViewAdrs;
                }
            }
            _ => {}
        },
    }
}

async fn handle_mode_selection(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Up => {
            app.input_mode_selection_index = wrap_decrement(app.input_mode_selection_index, 2);
        }
        KeyCode::Down => {
            app.input_mode_selection_index = wrap_increment(app.input_mode_selection_index, 2);
        }
        KeyCode::Left => {
            app.chart_tab_index = wrap_decrement(app.chart_tab_index, 2);
        }
        KeyCode::Right => {
            app.chart_tab_index = wrap_increment(app.chart_tab_index, 2);
        }
        KeyCode::Enter => {
            app.advance_state();
        }
        KeyCode::Char('a') => {
            app.input_mode_selection_index = 0;
            app.advance_state();
        }
        KeyCode::Char('b') => {
            app.input_mode_selection_index = 1;
            app.advance_state();
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            app.reset();
        }
        KeyCode::Char('l') => {
            handle_fetch_blips(app).await;
        }
        KeyCode::Char('v') => {
            handle_fetch_adrs(app).await;
        }
        KeyCode::Char('q') => {
            app.running = false;
        }
        _ => {}
    }
}

async fn handle_text_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char(c) => app.current_input.push(c),
        KeyCode::Backspace => {
            app.current_input.pop();
        }
        KeyCode::Enter => {
            app.process_current_input();

            if app.input_mode == Some(InputMode::Blip) {
                if let Some(pool) = app.actions.db_pool.as_ref() {
                    let blip_name = app.blip_data.name.trim();
                    if !blip_name.is_empty() {
                        let already_checked =
                            app.last_checked_blip_name.as_deref() == Some(blip_name);

                        if !already_checked {
                            match blip_exists_by_name(pool, blip_name).await {
                                Ok(true) => {
                                    app.status_message =
                                        format!("Error: Blip already exists: {blip_name}");
                                    app.last_checked_blip_name = Some(blip_name.to_string());
                                    app.last_blip_name_exists = true;
                                    return;
                                }
                                Ok(false) => {
                                    app.last_checked_blip_name = Some(blip_name.to_string());
                                    app.last_blip_name_exists = false;
                                }
                                Err(e) => {
                                    app.status_message =
                                        format!("Error: Failed to check blip name: {e}");
                                    app.last_checked_blip_name = None;
                                    app.last_blip_name_exists = false;
                                    return;
                                }
                            }
                        }

                        if app.last_blip_name_exists {
                            return;
                        }
                    }
                }
            } else {
                app.adr_status_selection_index = 0;
                app.adr_status = Some(AdrStatus::Proposed);
            }

            app.advance_state();
        }
        KeyCode::Esc => {
            app.reset();
        }
        _ => {}
    }
}

async fn handle_fetch_blips(app: &mut App) {
    match app.fetch_blips().await {
        Ok(()) => {
            app.selected_blip_index = 0;
            app.screen = AppScreen::ViewBlips;
        }
        Err(e) => {
            app.status_message = format!("Failed to fetch blips from database: {e}");
        }
    }
}

async fn handle_fetch_adrs(app: &mut App) {
    match app.fetch_adrs_for_blip("").await {
        Ok(()) => {
            app.selected_adr_index = 0;
            app.screen = AppScreen::ViewAdrs;
        }
        Err(e) => {
            app.status_message = format!("Failed to fetch ADRs from database: {e}");
        }
    }
}

fn handle_adr_status_selection(app: &mut App, key: KeyCode) {
    let max_statuses = 5;
    match key {
        KeyCode::Up => {
            app.adr_status_selection_index =
                wrap_decrement(app.adr_status_selection_index, max_statuses);
        }
        KeyCode::Down => {
            app.adr_status_selection_index =
                wrap_increment(app.adr_status_selection_index, max_statuses);
        }
        KeyCode::Enter => {
            app.process_current_input();
            app.advance_state();
        }

        KeyCode::Esc => {
            app.reset();
        }
        _ => {}
    }
}

fn handle_quadrant_selection(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Up => {
            app.quadrant_selection_index = wrap_decrement(app.quadrant_selection_index, 4);
        }
        KeyCode::Down => {
            app.quadrant_selection_index = wrap_increment(app.quadrant_selection_index, 4);
        }
        KeyCode::Enter => {
            app.process_current_input();
            app.advance_state();
        }
        KeyCode::Esc => {
            app.reset();
        }
        _ => {}
    }
}

fn handle_ring_selection(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Up => {
            app.ring_selection_index = wrap_decrement(app.ring_selection_index, 4);
        }
        KeyCode::Down => {
            app.ring_selection_index = wrap_increment(app.ring_selection_index, 4);
        }
        KeyCode::Enter => {
            app.process_current_input();
            app.advance_state();
        }
        KeyCode::Esc => {
            app.reset();
        }
        _ => {}
    }
}
