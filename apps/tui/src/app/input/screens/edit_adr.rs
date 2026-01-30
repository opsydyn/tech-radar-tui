use crate::app::state::{AdrStatus, App, AppScreen};
use crate::db::queries::AdrUpdateParams;
use crossterm::event::KeyCode;

#[allow(clippy::missing_const_for_fn)]
pub async fn handle_edit_adr_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => {
            if let Some(edit_state) = &mut app.edit_adr_state {
                if edit_state.editing {
                    edit_state.editing = false;
                    return;
                }
            }
            app.edit_adr_state = None;
            app.screen = AppScreen::ViewAdrs;
        }
        KeyCode::Up => {
            if let Some(edit_state) = &mut app.edit_adr_state {
                if !edit_state.editing {
                    edit_state.field = edit_state.field.prev();
                }
            }
        }
        KeyCode::Down => {
            if let Some(edit_state) = &mut app.edit_adr_state {
                if !edit_state.editing {
                    edit_state.field = edit_state.field.next();
                }
            }
        }
        KeyCode::Enter => {
            if let Some(edit_state) = &mut app.edit_adr_state {
                if edit_state.editing {
                    edit_state.editing = false;
                } else if edit_state.field == AdrEditField::Save {
                    apply_edit_save(app).await;
                } else {
                    edit_state.editing = true;
                }
            }
        }
        _ => {
            if let Some(edit_state) = &mut app.edit_adr_state {
                if edit_state.editing {
                    handle_edit_input(edit_state, key);
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AdrEditField {
    Title,
    Status,
    Save,
}

impl AdrEditField {
    pub const fn next(self) -> Self {
        match self {
            Self::Title => Self::Status,
            Self::Status => Self::Save,
            Self::Save => Self::Title,
        }
    }

    pub const fn prev(self) -> Self {
        match self {
            Self::Title => Self::Save,
            Self::Status => Self::Title,
            Self::Save => Self::Status,
        }
    }
}

fn handle_edit_input(edit_state: &mut AdrEditState, key: KeyCode) {
    match edit_state.field {
        AdrEditField::Title => match key {
            KeyCode::Char(c) => edit_state.title.push(c),
            KeyCode::Backspace => {
                edit_state.title.pop();
            }
            _ => {}
        },
        AdrEditField::Status => match key {
            KeyCode::Left => edit_state.status = edit_state.status.prev(),
            KeyCode::Right => edit_state.status = edit_state.status.next(),
            _ => {}
        },
        AdrEditField::Save => {}
    }
}

async fn apply_edit_save(app: &mut App) {
    let (Some(adr), Some(edit_state)) = (app.adrs.get(app.selected_adr_index), &app.edit_adr_state)
    else {
        return;
    };

    let params = AdrUpdateParams {
        id: adr.id,
        title: Some(edit_state.title.clone()),
        blip_name: None,
        status: Some(edit_state.status.as_str().to_string()),
        created: None,
    };

    match app.actions.update_adr(&params).await {
        Ok(()) => {
            app.status_message = "ADR updated successfully".to_string();
        }
        Err(e) => {
            app.status_message = format!("Failed to update ADR: {e}");
        }
    }

    if let Ok(adrs) = app.actions.fetch_adrs_for_blip("").await {
        app.adrs = adrs;
    }

    if let Some(edit_state) = &mut app.edit_adr_state {
        edit_state.editing = false;
    }
}

#[derive(Clone)]
pub struct AdrEditState {
    pub field: AdrEditField,
    pub title: String,
    pub status: AdrStatus,
    pub editing: bool,
}
