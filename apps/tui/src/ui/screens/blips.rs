use crate::app::App;
use crate::ui::widgets::radar::quadrant_color;
use crate::ui::widgets::tables::scroll_offset;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line as TextLine, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;

pub fn render_blips_view(app: &App, f: &mut Frame<'_>) {
    let area = f.area();

    if app.blips.is_empty() {
        let block = Block::default()
            .title("Blips Table")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        let paragraph = Paragraph::new("No blips found.")
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(paragraph, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("ID"),
        Cell::from("Name"),
        Cell::from("Ring"),
        Cell::from("Quadrant"),
        Cell::from("Tag"),
        Cell::from("Has ADR"),
    ])
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );

    let total_rows = app.blips.len();
    let max_visible_rows = area.height.saturating_sub(7) as usize;

    let scroll_offset = scroll_offset(total_rows, max_visible_rows, app.selected_blip_index);

    let visible_blips = app.blips.iter().skip(scroll_offset).take(max_visible_rows);

    let rows = visible_blips.enumerate().map(|(i, blip)| {
        let is_selected = i + scroll_offset == app.selected_blip_index;
        let style = if is_selected {
            Style::default()
                .bg(Color::Rgb(0, 0, 238))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            blip.quadrant.map_or_else(Style::default, |quadrant| {
                Style::default().fg(quadrant_color(quadrant.as_str()))
            })
        };

        Row::new(vec![
            Cell::from(blip.id.to_string()),
            Cell::from(blip.name.clone()),
            Cell::from(
                blip.ring
                    .map_or_else(String::new, |ring| ring.as_str().to_string()),
            ),
            Cell::from(
                blip.quadrant
                    .map_or_else(String::new, |quadrant| quadrant.as_str().to_string()),
            ),
            Cell::from(blip.tag.clone().unwrap_or_default()),
            Cell::from(if blip.has_adr { "Yes" } else { "No" }),
        ])
        .style(style)
    });

    let widths = [
        Constraint::Length(4),
        Constraint::Length(20),
        Constraint::Length(10),
        Constraint::Length(12),
        Constraint::Length(12),
        Constraint::Length(8),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .title(format!(
                    "Blips Table ({} of {})",
                    app.selected_blip_index + 1,
                    total_rows
                ))
                .borders(Borders::ALL),
        )
        .column_spacing(1);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(area);

    f.render_widget(table, chunks[0]);

    let help_text = vec![
        Span::styled(
            "ESC",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": Return to Main Menu   "),
        Span::styled(
            "↑/↓",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": Navigate   "),
        Span::styled(
            "PgUp/PgDn",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": Jump 5 rows   "),
        Span::styled(
            "Home/End",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": First/Last   "),
        Span::styled(
            "Enter",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": Actions   "),
        Span::styled(
            "q",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": Quit"),
    ];

    let help_paragraph = Paragraph::new(TextLine::from(help_text))
        .block(Block::default().borders(Borders::TOP))
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(help_paragraph, chunks[1]);
}
