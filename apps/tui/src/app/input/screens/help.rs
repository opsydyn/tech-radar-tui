use crate::app::state::App;
use crossterm::event::KeyCode;

pub fn handle_help_toggle(app: &mut App, key: KeyCode) -> bool {
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

pub fn handle_animation_toggle(app: &mut App, key: KeyCode) -> bool {
    if key == KeyCode::Char(' ') {
        app.toggle_animation_pause();
        return true;
    }

    false
}
