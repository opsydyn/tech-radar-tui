use crate::app::input::screens::edit_adr::AdrEditField;
use crate::app::state::App;

use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line as TextLine, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

pub fn render_edit_adr(app: &App, f: &mut Frame<'_>) {
    let Some(edit_state) = &app.edit_adr_state else {
        return;
    };

    let area = f.area();
    let popup = Rect {
        x: area.width.saturating_sub(60) / 2,
        y: area.height.saturating_sub(12) / 2,
        width: 60.min(area.width),
        height: 12.min(area.height),
    };

    let block = Block::default()
        .title("Edit ADR")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    f.render_widget(block, popup);

    let inner = popup.inner(ratatui::layout::Margin::new(1, 1));
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(2),
            Constraint::Length(1),
            Constraint::Length(2),
        ])
        .split(inner);

    let field_style = |field: AdrEditField| {
        let is_selected = edit_state.field == field;
        let is_editing = edit_state.editing && is_selected;
        if is_editing {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        }
    };

    let title_line = TextLine::from(vec![
        Span::styled("Title: ", Style::default().fg(Color::Gray)),
        Span::styled(&edit_state.title, field_style(AdrEditField::Title)),
    ]);
    f.render_widget(Paragraph::new(title_line), layout[0]);

    let status_text = edit_state.status.label();
    let status_line = TextLine::from(vec![
        Span::styled("Status: ", Style::default().fg(Color::Gray)),
        Span::styled(status_text, field_style(AdrEditField::Status)),
    ]);
    f.render_widget(Paragraph::new(status_line), layout[1]);

    let save_style = field_style(AdrEditField::Save);
    let save_block = Block::default()
        .borders(Borders::ALL)
        .border_style(save_style);
    let save = Paragraph::new(Text::from(Span::styled("Save", save_style)))
        .block(save_block)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(save, layout[3]);

    let status_text = if !app.status_message.is_empty() {
        app.status_message.as_str()
    } else if edit_state.editing {
        match edit_state.field {
            AdrEditField::Status => "Editing: ←/→ cycle status, Enter confirm, Esc cancel",
            _ => "Editing: type to edit, Enter confirm, Esc cancel",
        }
    } else {
        "Navigate: ↑/↓ select, Enter edit/save, Esc back"
    };
    let status_paragraph = Paragraph::new(Text::from(status_text))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(Color::Gray));
    f.render_widget(status_paragraph, layout[4]);

    let hint = TextLine::from(vec![
        Span::styled(
            "↑/↓",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": Select   "),
        Span::styled(
            "Enter",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": Edit/Save   "),
        Span::styled(
            "Esc",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": Back"),
    ]);
    let hint_paragraph = Paragraph::new(Text::from(hint))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(hint_paragraph, layout[5]);
}
