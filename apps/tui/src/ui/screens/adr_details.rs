use crate::app::App;
use crate::ui::widgets::popup::{centered_rect, ClearWidget};
use ratatui::style::{Color, Style};
use ratatui::text::{Line as TextLine, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

pub fn render_adr_details(app: &App, f: &mut Frame<'_>) {
    let area = f.area();

    let Some(adr) = app.adrs.get(app.selected_adr_index) else {
        return;
    };

    let popup_area = centered_rect(70, 60, area);
    f.render_widget(ClearWidget, popup_area);

    let block = Block::default()
        .title(format!("ADR Details: {}", adr.title))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let lines = vec![
        TextLine::from(format!("ID: {}", adr.id)),
        TextLine::from(format!("Title: {}", adr.title)),
        TextLine::from(format!("Blip: {}", adr.blip_name)),
        TextLine::from(format!("Status: {}", adr.status)),
        TextLine::from(format!("Timestamp: {}", adr.timestamp)),
        TextLine::from(""),
        TextLine::from("Press Esc to close"),
    ];

    let paragraph = Paragraph::new(Text::from(lines))
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, popup_area);
}
