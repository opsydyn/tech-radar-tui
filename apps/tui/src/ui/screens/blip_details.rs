use crate::app::App;
use ratatui::style::{Color, Style};
use ratatui::text::{Line as TextLine, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

pub fn render_blip_details(app: &App, f: &mut Frame<'_>) {
    let area = f.area();

    let Some(blip) = app.blips.get(app.selected_blip_index) else {
        return;
    };

    let block = Block::default()
        .title(format!("Blip Details: {}", blip.name))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let lines = vec![
        TextLine::from(format!("Name: {}", blip.name)),
        TextLine::from(format!(
            "Ring: {}",
            blip.ring
                .map_or_else(|| "(none)".to_string(), |ring| ring.as_str().to_string())
        )),
        TextLine::from(format!(
            "Quadrant: {}",
            blip.quadrant.map_or_else(
                || "(none)".to_string(),
                |quadrant| quadrant.as_str().to_string(),
            ),
        )),
        TextLine::from(format!(
            "Tag: {}",
            blip.tag.clone().unwrap_or_else(|| "(none)".to_string())
        )),
        TextLine::from(format!(
            "ADR Linked: {}",
            blip.adr_id
                .map_or_else(|| "none".to_string(), |id| id.to_string())
        )),
    ];

    let paragraph = Paragraph::new(Text::from(lines))
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}
