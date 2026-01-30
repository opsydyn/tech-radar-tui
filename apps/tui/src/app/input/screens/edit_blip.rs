use crate::app::state::{App, EditField};
use crossterm::event::KeyCode;
use std::time::{Duration, Instant};

#[allow(clippy::cognitive_complexity)]
pub async fn handle_edit_blip_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => {
            if let Some(edit_state) = &mut app.edit_blip_state {
                if edit_state.editing {
                    edit_state.editing = false;
                    return;
                }
            }
            app.save_notice_until = None;
            app.status_message.clear();
            app.screen = crate::app::state::AppScreen::ViewBlips;
            app.edit_blip_state = None;
        }
        KeyCode::Up => {
            if let Some(edit_state) = &mut app.edit_blip_state {
                if !edit_state.editing {
                    edit_state.field = match edit_state.field {
                        EditField::Name => EditField::Save,
                        EditField::Ring => EditField::Name,
                        EditField::Quadrant => EditField::Ring,
                        EditField::Tag => EditField::Quadrant,
                        EditField::Description => EditField::Tag,
                        EditField::Save => EditField::Description,
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
                        EditField::Description => EditField::Save,
                        EditField::Save => EditField::Name,
                    };
                }
            }
        }
        KeyCode::Enter => {
            if let Some(edit_state) = &mut app.edit_blip_state {
                if edit_state.editing {
                    edit_state.editing = false;
                } else if edit_state.field == EditField::Save {
                    apply_edit_save(app).await;
                } else {
                    edit_state.editing = true;
                }
            }
        }
        _ => {
            if let Some(edit_state) = &mut app.edit_blip_state {
                if edit_state.editing {
                    handle_edit_input(edit_state, key);
                }
            }
        }
    }
}

fn handle_edit_input(edit_state: &mut crate::app::state::EditBlipState, key: KeyCode) {
    let field_value = match edit_state.field {
        EditField::Name => &mut edit_state.name,
        EditField::Ring => &mut edit_state.ring,
        EditField::Quadrant => &mut edit_state.quadrant,
        EditField::Tag => &mut edit_state.tag,
        EditField::Description | EditField::Save => &mut edit_state.description,
    };

    match key {
        KeyCode::Char(c) => match edit_state.field {
            EditField::Ring | EditField::Quadrant | EditField::Save => {}
            _ => {
                field_value.push(c);
            }
        },
        KeyCode::Backspace => match edit_state.field {
            EditField::Ring | EditField::Quadrant | EditField::Save => {}
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

async fn apply_edit_save(app: &mut App) {
    let Some(edit_state) = app.edit_blip_state.as_ref() else {
        return;
    };

    let params = crate::db::queries::BlipUpdateParams {
        id: edit_state.id,
        name: Some(edit_state.name.clone()),
        ring: crate::Ring::from_index(edit_state.ring_index),
        quadrant: crate::Quadrant::from_index(edit_state.quadrant_index),
        tag: Some(edit_state.tag.clone()),
        description: Some(edit_state.description.clone()),
        adr_id: edit_state.adr_id,
    };

    match app.update_blip(params).await {
        Ok(()) => {
            app.status_message = "Blip updated successfully".to_string();
            app.save_notice_until = Some(Instant::now() + Duration::from_secs(2));
        }
        Err(e) => {
            app.status_message = format!("Failed to update blip: {e}");
            app.save_notice_until = Some(Instant::now() + Duration::from_secs(3));
        }
    }

    if let Some(edit_state) = &mut app.edit_blip_state {
        edit_state.editing = false;
    }
}
