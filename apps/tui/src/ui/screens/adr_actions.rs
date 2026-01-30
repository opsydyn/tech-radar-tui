use crate::app::App;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line as TextLine, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn render_adr_actions(app: &App, f: &mut Frame<'_>) {
    let area = f.area();

    let Some(selected_adr) = app.adrs.get(app.selected_adr_index) else {
        return;
    };

    let action_area = Rect {
        x: area.width.saturating_sub(50) / 2,
        y: area.height.saturating_sub(10) / 2,
        width: 50.min(area.width),
        height: 10.min(area.height),
    };

    let block = Block::default()
        .title(format!("Actions for ADR: {}", selected_adr.title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let actions = ["View details", "Edit ADR", "Back to list"];

    let action_text = actions
        .iter()
        .enumerate()
        .map(|(i, &action)| {
            let is_selected = i == app.adr_action_index;
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
        x: area.x,
        y: area.height - 3,
        width: area.width,
        height: 3,
    };

    let help_paragraph = Paragraph::new(TextLine::from(help_text))
        .block(Block::default().borders(Borders::TOP))
        .alignment(Alignment::Center);

    f.render_widget(help_paragraph, help_area);
}
