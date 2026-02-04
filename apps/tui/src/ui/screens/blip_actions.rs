use crate::app::App;
use crate::ui::widgets::popup::{centered_rect, ClearWidget};
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line as TextLine, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn render_blip_actions(app: &App, f: &mut Frame<'_>) {
    let area = f.area();

    let selected_blip = if app.filtered_blip_indices.is_empty() {
        app.blips.get(app.selected_blip_index)
    } else {
        app.filtered_blip_indices
            .get(app.selected_blip_index)
            .and_then(|index| app.blips.get(*index))
    };

    if let Some(selected_blip) = selected_blip {
        let action_area = centered_rect(60, 55, area);
        f.render_widget(ClearWidget, action_area);

        let block = Block::default()
            .title(format!("Actions for Blip: {}", selected_blip.name))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));

        let actions = [
            "View details",
            if selected_blip.has_adr {
                "View ADR"
            } else {
                "Generate ADR"
            },
            "Edit blip",
            "Back to list",
        ];

        let action_text = actions
            .iter()
            .enumerate()
            .map(|(i, &action)| {
                let is_selected = i == app.blip_action_index;
                let style = if is_selected {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                let prefix = if is_selected { ">" } else { " " };

                TextLine::from(vec![
                    Span::styled(format!("{prefix} "), style),
                    Span::styled(action, style),
                ])
            })
            .collect::<Vec<_>>();

        let paragraph = Paragraph::new(action_text)
            .block(block)
            .alignment(Alignment::Left);

        f.render_widget(paragraph, action_area);

        let help_text = vec![
            Span::styled(
                "↑/↓",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Select action   "),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Confirm   "),
            Span::styled(
                "ESC",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(": Back to list"),
        ];

        let help_area = Rect {
            x: action_area.x,
            y: action_area.y + action_area.height.saturating_sub(3),
            width: action_area.width,
            height: 3,
        };

        let help_paragraph = Paragraph::new(TextLine::from(help_text))
            .block(Block::default().borders(Borders::TOP))
            .alignment(Alignment::Center);

        f.render_widget(help_paragraph, help_area);

        let hint = Paragraph::new(TextLine::from(vec![Span::styled(
            "Press Esc to close",
            Style::default().fg(Color::Gray),
        )]))
        .alignment(Alignment::Center);

        let hint_area = Rect {
            x: action_area.x,
            y: action_area.y + action_area.height.saturating_sub(2),
            width: action_area.width,
            height: 1,
        };

        f.render_widget(hint, hint_area);
    }
}
