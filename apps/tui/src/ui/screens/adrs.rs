use crate::app::App;
use crate::ui::widgets::radar::quadrant_color;
use crate::ui::widgets::tables::scroll_offset;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line as TextLine, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};
use ratatui::Frame;

pub fn render_adrs_view(app: &App, f: &mut Frame<'_>) {
    let area = f.area();

    if app.adrs.is_empty() {
        let block = Block::default()
            .title("ADR Log")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        let paragraph = Paragraph::new("No ADRs found.")
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(paragraph, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("ID"),
        Cell::from("Title"),
        Cell::from("Blip"),
        Cell::from("Timestamp"),
    ])
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    );

    let total_rows = app.adrs.len();
    let max_visible_rows = area.height.saturating_sub(7) as usize;

    let scroll_offset = scroll_offset(total_rows, max_visible_rows, app.selected_adr_index);

    let visible_adrs = app.adrs.iter().skip(scroll_offset).take(max_visible_rows);

    let rows = visible_adrs.enumerate().map(|(i, adr)| {
        let is_selected = i + scroll_offset == app.selected_adr_index;
        let style = if is_selected {
            Style::default()
                .bg(Color::Rgb(0, 0, 238))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(quadrant_color("platforms"))
        };
        Row::new(vec![
            Cell::from(adr.id.to_string()),
            Cell::from(adr.title.clone()),
            Cell::from(adr.blip_name.clone()),
            Cell::from(adr.timestamp.clone()),
        ])
        .style(style)
    });

    let title = app.adr_filter_name.as_ref().map_or_else(
        || format!("ADR Log ({} of {})", app.selected_adr_index + 1, total_rows),
        |filter| {
            format!(
                "ADR Log for {} ({} of {})",
                filter,
                app.selected_adr_index + 1,
                total_rows
            )
        },
    );

    let widths = [
        Constraint::Length(4),
        Constraint::Length(20),
        Constraint::Length(20),
        Constraint::Length(12),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().title(title).borders(Borders::ALL))
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
            "Enter",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(": Details"),
    ];

    let help_paragraph = Paragraph::new(TextLine::from(help_text))
        .block(Block::default().borders(Borders::TOP))
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(help_paragraph, chunks[1]);
}
