use crate::app::state::{App, EditBlipState, EditField, InputMode, InputState};
use crossterm::event::KeyCode;

pub async fn handle_input(app: &mut App, key: KeyCode) {
    if handle_help_toggle(app, key) {
        return;
    }

    if app.screen == crate::app::state::AppScreen::ViewBlips {
        handle_view_blips_input(app, key);
        return;
    }

    if app.screen == crate::app::state::AppScreen::BlipActions {
        handle_blip_actions_input(app, key);
        return;
    }

    if app.screen == crate::app::state::AppScreen::EditBlip {
        handle_edit_blip_input(app, key).await;
        return;
    }

    handle_main_input(app, key).await;
}

fn handle_help_toggle(app: &mut App, key: KeyCode) -> bool {
    if key == KeyCode::F(1) {
        app.show_help = !app.show_help;
        return true;
    }

    if app.show_help {
        if key == KeyCode::Esc {
            app.show_help = false;
        }
        return true;
    }

    false
}

#[allow(clippy::missing_const_for_fn)]
fn handle_view_blips_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => {
            app.screen = crate::app::state::AppScreen::Main;
        }
        KeyCode::Char('q') => {
            app.running = false;
        }
        KeyCode::Up => {
            if app.selected_blip_index > 0 {
                app.selected_blip_index -= 1;
            }
        }
        KeyCode::Down => {
            if !app.blips.is_empty() && app.selected_blip_index < app.blips.len() - 1 {
                app.selected_blip_index += 1;
            }
        }
        KeyCode::PageUp => {
            if app.selected_blip_index > 0 {
                app.selected_blip_index = app.selected_blip_index.saturating_sub(5);
            }
        }
        KeyCode::PageDown => {
            if !app.blips.is_empty() {
                let new_index = app.selected_blip_index + 5;
                app.selected_blip_index = if new_index >= app.blips.len() {
                    app.blips.len() - 1
                } else {
                    new_index
                };
            }
        }
        KeyCode::Home => {
            app.selected_blip_index = 0;
        }
        KeyCode::End => {
            if !app.blips.is_empty() {
                app.selected_blip_index = app.blips.len() - 1;
            }
        }
        KeyCode::Enter => {
            if !app.blips.is_empty() {
                app.screen = crate::app::state::AppScreen::BlipActions;
            }
        }
        _ => {}
    }
}

fn handle_blip_actions_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('1') => {
            app.status_message = "Viewing blip details (not implemented yet)".to_string();
            app.screen = crate::app::state::AppScreen::ViewBlips;
        }
        KeyCode::Char('2') => {
            if let Some(blip) = app.blips.get(app.selected_blip_index) {
                if blip.has_adr {
                    app.status_message = "Viewing ADR (not implemented yet)".to_string();
                } else {
                    app.status_message = "Generating ADR (not implemented yet)".to_string();
                }
            }
            app.screen = crate::app::state::AppScreen::ViewBlips;
        }
        KeyCode::Char('3') => {
            if let Some(blip) = app.blips.get(app.selected_blip_index) {
                app.edit_blip_state = Some(EditBlipState::from_blip(blip));
                app.screen = crate::app::state::AppScreen::EditBlip;
            }
        }
        KeyCode::Esc | KeyCode::Char('4') => {
            app.screen = crate::app::state::AppScreen::ViewBlips;
        }
        _ => {}
    }
}

#[allow(clippy::cognitive_complexity)]
async fn handle_edit_blip_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => {
            if let Some(edit_state) = &mut app.edit_blip_state {
                if edit_state.editing {
                    edit_state.editing = false;
                    return;
                }
            }
            app.screen = crate::app::state::AppScreen::ViewBlips;
            app.edit_blip_state = None;
        }
        KeyCode::Char('s' | 'S') => {
            if let Some(edit_state) = &app.edit_blip_state {
                if edit_state.editing {
                    if let Some(edit_state) = &mut app.edit_blip_state {
                        let field_value = match edit_state.field {
                            EditField::Name => &mut edit_state.name,
                            EditField::Ring => &mut edit_state.ring,
                            EditField::Quadrant => &mut edit_state.quadrant,
                            EditField::Tag => &mut edit_state.tag,
                            EditField::Description => &mut edit_state.description,
                        };
                        field_value.push(if key == KeyCode::Char('S') { 'S' } else { 's' });
                    }
                    return;
                }
            }

            if let (Some(blip), Some(edit_state)) =
                (app.blips.get(app.selected_blip_index), &app.edit_blip_state)
            {
                let params = crate::db::queries::BlipUpdateParams {
                    id: blip.id,
                    name: Some(edit_state.name.clone()),
                    ring: Some(edit_state.ring.clone()),
                    quadrant: Some(edit_state.quadrant.clone()),
                    tag: Some(edit_state.tag.clone()),
                    description: Some(edit_state.description.clone()),
                };

                match app.update_blip(params).await {
                    Ok(()) => {
                        app.status_message = "Blip updated successfully".to_string();
                    }
                    Err(e) => {
                        app.status_message = format!("Failed to update blip: {e}");
                    }
                }
            }

            app.screen = crate::app::state::AppScreen::ViewBlips;
            app.edit_blip_state = None;
        }
        KeyCode::Up => {
            if let Some(edit_state) = &mut app.edit_blip_state {
                if !edit_state.editing {
                    edit_state.field = match edit_state.field {
                        EditField::Name => EditField::Description,
                        EditField::Ring => EditField::Name,
                        EditField::Quadrant => EditField::Ring,
                        EditField::Tag => EditField::Quadrant,
                        EditField::Description => EditField::Tag,
                    };
                }
            }
        }
        KeyCode::Down => {
            if let Some(edit_state) = &mut app.edit_blip_state {
                if !edit_state.editing {
                    edit_state.field = match edit_state.field {
                        EditField::Name => EditField::Ring,
                        EditField::Ring => EditField::Quadrant,
                        EditField::Quadrant => EditField::Tag,
                        EditField::Tag => EditField::Description,
                        EditField::Description => EditField::Name,
                    };
                }
            }
        }
        KeyCode::Enter => {
            if let Some(edit_state) = &mut app.edit_blip_state {
                edit_state.editing = !edit_state.editing;
            }
        }
        _ => {
            if let Some(edit_state) = &mut app.edit_blip_state {
                if edit_state.editing {
                    let field_value = match edit_state.field {
                        EditField::Name => &mut edit_state.name,
                        EditField::Ring => &mut edit_state.ring,
                        EditField::Quadrant => &mut edit_state.quadrant,
                        EditField::Tag => &mut edit_state.tag,
                        EditField::Description => &mut edit_state.description,
                    };

                    match key {
                        KeyCode::Char(c) => match edit_state.field {
                            EditField::Ring | EditField::Quadrant => {}
                            _ => {
                                field_value.push(c);
                            }
                        },
                        KeyCode::Backspace => match edit_state.field {
                            EditField::Ring | EditField::Quadrant => {}
                            _ => {
                                field_value.pop();
                            }
                        },
                        KeyCode::Enter => {
                            edit_state.editing = false;
                        }
                        KeyCode::Left => match edit_state.field {
                            EditField::Ring => edit_state.prev_ring(),
                            EditField::Quadrant => edit_state.prev_quadrant(),
                            _ => {}
                        },
                        KeyCode::Right => match edit_state.field {
                            EditField::Ring => edit_state.next_ring(),
                            EditField::Quadrant => edit_state.next_quadrant(),
                            _ => {}
                        },
                        _ => {}
                    }
                }
            }
        }
    }
}

async fn handle_main_input(app: &mut App, key: KeyCode) {
    match app.input_state {
        InputState::WaitingForCommand => handle_command_input(app, key).await,
        InputState::EnteringTechnology => handle_text_input(app, key),
        InputState::ChoosingQuadrant => handle_quadrant_selection(app, key),
        InputState::ChoosingRing => handle_ring_selection(app, key),
        InputState::GeneratingFile => {}
        InputState::Completed => {
            if key == KeyCode::Char('n') {
                app.reset();
            } else if key == KeyCode::Char('q') {
                app.running = false;
            }
        }
    }
}

async fn handle_command_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char('q') => app.running = false,
        KeyCode::Char('a') => {
            app.input_mode = Some(InputMode::Adr);
            app.advance_state();
        }
        KeyCode::Char('b') => {
            app.input_mode = Some(InputMode::Blip);
            app.advance_state();
        }
        KeyCode::Char('l') => {
            match app.fetch_blips().await {
                Ok(()) => {
                    // Reset selection to the first item when switching to Blips view
                    app.selected_blip_index = 0;
                    app.screen = crate::app::state::AppScreen::ViewBlips;
                }
                Err(e) => {
                    eprintln!("[DEBUG] fetch_blips error: {e:?}");
                    app.status_message = format!("Failed to fetch blips from database: {e}");
                }
            }
        }
        _ => {}
    }
}

fn handle_text_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Char(c) => app.current_input.push(c),
        KeyCode::Backspace => {
            app.current_input.pop();
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
            if app.quadrant_selection_index == 0 {
                app.quadrant_selection_index = 3;
            } else {
                app.quadrant_selection_index -= 1;
            }
        }
        KeyCode::Down => {
            app.quadrant_selection_index = (app.quadrant_selection_index + 1) % 4;
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
            if app.ring_selection_index == 0 {
                app.ring_selection_index = 3;
            } else {
                app.ring_selection_index -= 1;
            }
        }
        KeyCode::Down => {
            app.ring_selection_index = (app.ring_selection_index + 1) % 4;
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
