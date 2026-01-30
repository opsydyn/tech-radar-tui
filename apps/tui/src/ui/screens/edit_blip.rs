use crate::app::state::EditField;
use crate::app::App;
use crate::ui::widgets::popup::{centered_rect, ClearWidget};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line as TextLine, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use std::time::Instant;

pub fn render_edit_blip(app: &App, f: &mut Frame<'_>) {
    let area = f.area();

    if let (Some(selected_blip), Some(edit_state)) =
        (app.blips.get(app.selected_blip_index), &app.edit_blip_state)
    {
        let form_area = Rect {
            x: area.width.saturating_sub(60) / 2,
            y: area.height.saturating_sub(15) / 2,
            width: 60.min(area.width),
            height: 15.min(area.height),
        };

        let block = Block::default()
            .title(format!("Edit Blip: {}", selected_blip.name))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        f.render_widget(block, form_area);

        let form_chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(1), // Name
                Constraint::Length(1), // Ring
                Constraint::Length(1), // Quadrant
                Constraint::Length(1), // Tag
                Constraint::Length(3), // Description
                Constraint::Length(1), // Save
                Constraint::Length(1), // Help text
            ])
            .split(form_area);

        let field_style = |field: EditField| {
            let is_selected = edit_state.field == field;
            let is_editing = is_selected && edit_state.editing;

            if is_editing {
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            }
        };

        let field_label = |name: &str, field: EditField| {
            let style = field_style(field);
            let prefix = if edit_state.field == field && edit_state.editing {
                "► "
            } else if edit_state.field == field {
                "> "
            } else {
                "  "
            };

            let suffix = if field == EditField::Save { "" } else { ": " };
            Span::styled(format!("{prefix}{name}{suffix}"), style)
        };

        let name_text = TextLine::from(vec![
            field_label("Name", EditField::Name),
            Span::styled(&edit_state.name, field_style(EditField::Name)),
        ]);
        f.render_widget(Paragraph::new(name_text), form_chunks[0]);

        let ring_text = TextLine::from(vec![
            field_label("Ring", EditField::Ring),
            Span::styled(&edit_state.ring, field_style(EditField::Ring)),
        ]);
        f.render_widget(Paragraph::new(ring_text), form_chunks[1]);

        let quadrant_text = TextLine::from(vec![
            field_label("Quadrant", EditField::Quadrant),
            Span::styled(&edit_state.quadrant, field_style(EditField::Quadrant)),
        ]);
        f.render_widget(Paragraph::new(quadrant_text), form_chunks[2]);

        let tag_text = TextLine::from(vec![
            field_label("Tag", EditField::Tag),
            Span::styled(&edit_state.tag, field_style(EditField::Tag)),
        ]);
        f.render_widget(Paragraph::new(tag_text), form_chunks[3]);

        let description_label = field_label("Description", EditField::Description);
        let description_value =
            Span::styled(&edit_state.description, field_style(EditField::Description));

        let description_text = Text::from(vec![
            TextLine::from(vec![description_label]),
            TextLine::from(vec![description_value]),
        ]);
        f.render_widget(Paragraph::new(description_text), form_chunks[4]);

        let save_style = field_style(EditField::Save);
        let save_block = Block::default()
            .borders(Borders::ALL)
            .border_style(save_style);
        let save = Paragraph::new(Text::from(Span::styled("Save", save_style)))
            .block(save_block)
            .alignment(Alignment::Center);
        f.render_widget(save, form_chunks[5]);

        let status_text = if edit_state.editing {
            match edit_state.field {
                EditField::Ring | EditField::Quadrant => {
                    "Editing: ←/→ cycle options, Enter confirm, Esc cancel"
                }
                _ => "Editing: type to edit, Enter confirm, Esc cancel",
            }
        } else {
            "Navigate: ↑/↓ select, Enter edit/save, Esc exit"
        };

        let status_line = Paragraph::new(status_text)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));

        f.render_widget(status_line, form_chunks[6]);

        if let Some(until) = app.save_notice_until {
            if until > Instant::now() {
                let popup_area = centered_rect(40, 20, area);
                f.render_widget(ClearWidget, popup_area);
                let popup = Block::default()
                    .title("Saved")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green));
                let message = Paragraph::new(app.status_message.as_str())
                    .block(popup)
                    .alignment(Alignment::Center);
                f.render_widget(message, popup_area);
                f.set_cursor_position((0, 0));
            }
        }
    }
}
