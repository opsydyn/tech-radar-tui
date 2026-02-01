use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use ratzilla::ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line as TextLine, Span, Text},
    widgets::{
        Bar, BarChart, BarGroup, Block, Borders, Cell, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table, Wrap,
    },
    Terminal,
};
use ratzilla::{DomBackend, WebRenderer};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[derive(serde::Deserialize)]
struct RadarExport {
    blips: Vec<RadarBlip>,
    adrs: Vec<RadarAdr>,
}

#[derive(serde::Deserialize, Clone)]
#[allow(dead_code)]
struct RadarBlip {
    id: i32,
    name: String,
    ring: Option<String>,
    quadrant: Option<String>,
    tag: Option<String>,
    description: Option<String>,
    created: String,
    has_adr: bool,
    adr_id: Option<i32>,
}

#[derive(serde::Deserialize)]
#[allow(dead_code)]
struct RadarAdr {
    id: i32,
    title: String,
    blip_name: String,
    status: String,
    timestamp: String,
}

fn main() -> io::Result<()> {
    let data = Rc::new(RefCell::new(None::<RadarExport>));
    let tab_index = Rc::new(RefCell::new(0_usize));
    let row_offset = Rc::new(RefCell::new(0_usize));

    spawn_local(fetch_radar(data.clone()));

    let backend = DomBackend::new()?;
    let terminal = Terminal::new(backend)?;

    terminal.on_key_event({
        let tab_index = tab_index.clone();
        let row_offset = row_offset.clone();
        move |event| match event.code {
            ratzilla::event::KeyCode::Left => {
                let mut index = tab_index.borrow_mut();
                *index = if *index == 0 { 2 } else { *index - 1 };
                *row_offset.borrow_mut() = 0;
            }
            ratzilla::event::KeyCode::Right => {
                let mut index = tab_index.borrow_mut();
                *index = (*index + 1) % 3;
                *row_offset.borrow_mut() = 0;
            }
            ratzilla::event::KeyCode::Up => {
                let mut offset = row_offset.borrow_mut();
                *offset = offset.saturating_sub(1);
            }
            ratzilla::event::KeyCode::Down => {
                let mut offset = row_offset.borrow_mut();
                *offset = (*offset + 1).min(2000);
            }
            ratzilla::event::KeyCode::Char('1') => {
                *tab_index.borrow_mut() = 0;
                *row_offset.borrow_mut() = 0;
            }
            ratzilla::event::KeyCode::Char('2') => {
                *tab_index.borrow_mut() = 1;
                *row_offset.borrow_mut() = 0;
            }
            ratzilla::event::KeyCode::Char('3') => {
                *tab_index.borrow_mut() = 2;
                *row_offset.borrow_mut() = 0;
            }
            _ => {}
        }
    });

    terminal.draw_web(move |f| {
        let area = f.area();
        let block = Block::default()
            .title("Tech Radar")
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Gray));
        let inner = block.inner(area).inner(Margin::new(1, 1));
        f.render_widget(block, area);

        let data = data.borrow();
        if let Some(export) = data.as_ref() {
            let index = *tab_index.borrow();
            let row_offset = *row_offset.borrow();
            render_dashboard(export, index, row_offset, f, inner);
        } else {
            let paragraph = Paragraph::new(Text::from(TextLine::from("Loading radar.json...")))
                .alignment(Alignment::Center);
            f.render_widget(paragraph, inner);
        }
    });

    Ok(())
}

fn render_dashboard(
    export: &RadarExport,
    tab_index: usize,
    row_offset: usize,
    f: &mut ratzilla::ratatui::Frame<'_>,
    area: Rect,
) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(12),
            Constraint::Length(8),
        ])
        .split(area);

    render_header(export, f, main_layout[0]);
    render_gap(f, main_layout[1]);

    let content = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(main_layout[2]);

    render_radar_panel(export, f, content[0]);

    let charts = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(content[1]);

    render_blip_type_chart(export, f, charts[0]);
    render_ring_chart(export, f, charts[1]);

    render_footer(export, tab_index, row_offset, f, main_layout[3]);
}

fn render_header(export: &RadarExport, f: &mut ratzilla::ratatui::Frame<'_>, area: Rect) {
    let total_blips = export.blips.len();
    let total_adrs = export.adrs.len();

    let line = TextLine::from(vec![Span::styled(
        format!("Blips: {total_blips}  ADRs: {total_adrs}"),
        Style::default().fg(Color::White),
    )]);

    let block = Block::default()
        .title("Overview")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let paragraph = Paragraph::new(Text::from(line))
        .block(block)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn render_gap(f: &mut ratzilla::ratatui::Frame<'_>, area: Rect) {
    let paragraph = Paragraph::new("")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(paragraph, area);
}

fn render_footer(
    export: &RadarExport,
    tab_index: usize,
    row_offset: usize,
    f: &mut ratzilla::ratatui::Frame<'_>,
    area: Rect,
) {
    let total_blips = export.blips.len();
    let total_adrs = export.adrs.len();

    let tabs = ["Recent blips", "All blips", "All ADRs"];
    let tab_titles = tabs
        .iter()
        .map(|title| TextLine::from(*title))
        .collect::<Vec<_>>();

    let info = TextLine::from(vec![
        Span::styled("Tables", Style::default().fg(Color::Gray)),
        Span::raw("  "),
        Span::raw(format!("{total_blips} blips • {total_adrs} ADRs")),
        Span::raw("  "),
        Span::styled("Tab/1-3", Style::default().fg(Color::Gray)),
        Span::raw("  "),
        Span::styled("Arrows", Style::default().fg(Color::Gray)),
    ]);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);

    let tabs = ratzilla::ratatui::widgets::Tabs::new(tab_titles)
        .select(tab_index)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::White)
                .bg(Color::Rgb(0, 0, 238))
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::raw("|"));

    let info_paragraph = Paragraph::new(Text::from(info)).alignment(Alignment::Center);
    f.render_widget(info_paragraph, layout[0]);
    f.render_widget(tabs, layout[1]);
    render_gap(f, layout[2]);

    let table_area = layout[3];

    match tab_index {
        0 => render_recent_blips(export, row_offset, f, table_area),
        1 => render_all_blips(export, row_offset, f, table_area),
        2 => render_all_adrs(export, row_offset, f, table_area),
        _ => {}
    }
}

fn render_radar_panel(export: &RadarExport, f: &mut ratzilla::ratatui::Frame<'_>, area: Rect) {
    let block = Block::default()
        .title("Tech Radar")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if export.blips.is_empty() {
        let paragraph = Paragraph::new("No blips available")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(paragraph, inner);
        return;
    }

    let size = inner.width.min(inner.height);
    let square = Rect {
        x: inner.x + (inner.width - size) / 2,
        y: inner.y + (inner.height - size) / 2,
        width: size,
        height: size,
    };

    let points = export
        .blips
        .iter()
        .filter_map(|blip| {
            let quadrant = match blip.quadrant.as_deref()? {
                "platforms" => 0,
                "languages" => 1,
                "tools" => 2,
                "techniques" => 3,
                _ => return None,
            };
            let ring = match blip.ring.as_deref()? {
                "adopt" => 0,
                "trial" => 1,
                "assess" => 2,
                "hold" => 3,
                _ => return None,
            };

            let hash = blip
                .name
                .bytes()
                .fold(0_u64, |acc, b| acc.wrapping_mul(31) + u64::from(b));
            let jitter = f64::from((hash % 100) as u8) / 100.0;

            let quadrant_angle = std::f64::consts::FRAC_PI_2 * f64::from(quadrant);
            let angle_offset = (jitter - 0.5) * (std::f64::consts::FRAC_PI_2 * 0.6);
            let angle = quadrant_angle + angle_offset;

            let ring_step = 0.2 + (f64::from(ring) * 0.18);
            let radius = ring_step + (jitter * 0.1);

            Some((angle, radius, quadrant_color(blip.quadrant.as_deref())))
        })
        .collect::<Vec<_>>();

    f.render_widget(
        ratzilla::ratatui::widgets::canvas::Canvas::default()
            .paint(|ctx| {
                let width = f64::from(square.width);
                let height = f64::from(square.height);
                let center_x = width / 2.0;
                let center_y = height / 2.0;
                let max_radius = width.min(height) / 2.0 * 0.9;

                for i in 1..=4 {
                    let ring_radius = max_radius * (f64::from(i) / 4.0);
                    ctx.draw(&ratzilla::ratatui::widgets::canvas::Circle {
                        x: center_x,
                        y: center_y,
                        radius: ring_radius,
                        color: Color::Gray,
                    });
                }

                ctx.draw(&ratzilla::ratatui::widgets::canvas::Line {
                    x1: center_x,
                    y1: center_y - max_radius,
                    x2: center_x,
                    y2: center_y + max_radius,
                    color: Color::Gray,
                });
                ctx.draw(&ratzilla::ratatui::widgets::canvas::Line {
                    x1: center_x - max_radius,
                    y1: center_y,
                    x2: center_x + max_radius,
                    y2: center_y,
                    color: Color::Gray,
                });

                for (angle, radius, color) in &points {
                    let x = angle.cos().mul_add(max_radius * radius, center_x);
                    let y = angle.sin().mul_add(max_radius * radius, center_y);

                    ctx.draw(&ratzilla::ratatui::widgets::canvas::Circle {
                        x,
                        y,
                        radius: max_radius * 0.035,
                        color: *color,
                    });
                }
            })
            .x_bounds([0.0, f64::from(square.width)])
            .y_bounds([0.0, f64::from(square.height)]),
        square,
    );
}

fn render_blip_type_chart(export: &RadarExport, f: &mut ratzilla::ratatui::Frame<'_>, area: Rect) {
    let block = Block::default()
        .title("Blip Types")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chart_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(68), Constraint::Percentage(32)])
        .split(inner);

    let chart_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(chart_split[0])[1];

    let legend_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(chart_split[1])[1];

    let mut counts = [0_u64; 4];

    for blip in &export.blips {
        let Some(quadrant) = blip.quadrant.as_deref() else {
            continue;
        };
        let index = match quadrant {
            "platforms" => 0,
            "languages" => 1,
            "tools" => 2,
            "techniques" => 3,
            _ => continue,
        };
        counts[index] += 1;
    }

    let labels = ["Platforms", "Languages", "Tools", "Techniques"];
    let colors = [
        quadrant_color(Some("platforms")),
        quadrant_color(Some("languages")),
        quadrant_color(Some("tools")),
        quadrant_color(Some("techniques")),
    ];

    let bars: Vec<Bar<'_>> = counts
        .iter()
        .enumerate()
        .map(|(index, value)| {
            Bar::default()
                .value(*value)
                .label(TextLine::from(labels[index]))
                .style(Style::default().fg(colors[index]))
                .value_style(Style::default().fg(Color::White))
        })
        .collect();

    let max_value = counts.iter().copied().max().unwrap_or(0).max(1);

    let chart = BarChart::default()
        .block(Block::default())
        .data(BarGroup::default().bars(&bars))
        .max(max_value)
        .bar_gap(1)
        .bar_width(6);

    f.render_widget(chart, chart_area);

    let total = counts.iter().sum::<u64>().max(1);
    let mut legend_lines = Vec::new();
    legend_lines.push(TextLine::from(Span::styled(
        "Legend",
        Style::default().fg(Color::Gray),
    )));
    legend_lines.push(TextLine::from(""));

    for (index, label) in labels.iter().enumerate() {
        let count = counts[index];
        let percent = (count as f64 / total as f64) * 100.0;
        legend_lines.push(TextLine::from(vec![
            Span::styled(
                "■ ",
                Style::default()
                    .fg(colors[index])
                    .add_modifier(Modifier::DIM),
            ),
            Span::styled(
                *label,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::DIM),
            ),
            Span::styled(
                format!("  {count} ({percent:.1}%)"),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::DIM),
            ),
        ]));
        if index + 1 < labels.len() {
            legend_lines.push(TextLine::from(""));
        }
    }

    let legend = Paragraph::new(Text::from(legend_lines))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(legend, legend_area);
}

fn render_ring_chart(export: &RadarExport, f: &mut ratzilla::ratatui::Frame<'_>, area: Rect) {
    let block = Block::default()
        .title("Ring Distribution")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chart_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(inner)[1];

    let mut counts = [0_u64; 4];
    for blip in &export.blips {
        let Some(ring) = blip.ring.as_deref() else {
            continue;
        };
        let index = match ring {
            "hold" => 0,
            "assess" => 1,
            "trial" => 2,
            "adopt" => 3,
            _ => continue,
        };
        counts[index] += 1;
    }

    let labels = ["Hold", "Assess", "Trial", "Adopt"];
    let colors = [
        Color::Gray,
        Color::Cyan,
        Color::Yellow,
        Color::Rgb(0, 0, 238),
    ];

    let mut lines = Vec::new();
    for (index, label) in labels.iter().enumerate() {
        let count = counts[index];
        let width = chart_area.width.max(1) - 2;
        let max_value = counts.iter().copied().max().unwrap_or(1) as f64;
        let ratio = count as f64 / max_value;
        let fill = ((ratio * f64::from(width)).round()).clamp(1.0, f64::from(width)) as usize;
        let empty = width as usize - fill;

        let bar = format!("{}{}", "█".repeat(fill), "░".repeat(empty));
        lines.push(TextLine::from(vec![
            Span::styled(*label, Style::default().fg(colors[index])),
            Span::raw(" "),
            Span::styled(bar, Style::default().fg(colors[index])),
            Span::raw(format!("  {count}")),
        ]));
    }

    let total = counts.iter().sum::<u64>().max(1);
    lines.push(TextLine::from(Span::styled(
        "Legend",
        Style::default().fg(Color::Gray),
    )));
    lines.push(TextLine::from(""));

    for (index, label) in labels.iter().enumerate() {
        let count = counts[index];
        let percent = (count as f64 / total as f64) * 100.0;
        lines.push(TextLine::from(vec![
            Span::styled(
                "■ ",
                Style::default()
                    .fg(colors[index])
                    .add_modifier(Modifier::DIM),
            ),
            Span::styled(
                *label,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::DIM),
            ),
            Span::styled(
                format!("  {count} ({percent:.1}%)"),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::DIM),
            ),
        ]));
        if index + 1 < labels.len() {
            lines.push(TextLine::from(""));
        }
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, chart_area);
}

fn render_recent_blips(
    export: &RadarExport,
    row_offset: usize,
    f: &mut ratzilla::ratatui::Frame<'_>,
    area: Rect,
) {
    let mut blips = export.blips.clone();
    blips.sort_by(|a, b| b.created.cmp(&a.created));

    render_blip_rows(&blips, row_offset, f, area, 8);
}

fn render_all_blips(
    export: &RadarExport,
    row_offset: usize,
    f: &mut ratzilla::ratatui::Frame<'_>,
    area: Rect,
) {
    render_blip_rows(&export.blips, row_offset, f, area, 18);
}

fn render_blip_rows(
    blips: &[RadarBlip],
    row_offset: usize,
    f: &mut ratzilla::ratatui::Frame<'_>,
    area: Rect,
    max_rows: usize,
) {
    if blips.is_empty() {
        let paragraph = Paragraph::new("No blips available")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(paragraph, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("Name"),
        Cell::from("Quadrant"),
        Cell::from("Ring"),
        Cell::from("Tag"),
        Cell::from("Has ADR"),
    ])
    .style(
        Style::default()
            .fg(Color::Rgb(0, 0, 238))
            .bg(Color::Rgb(200, 200, 200))
            .add_modifier(Modifier::BOLD),
    );

    let rows = std::iter::once(Row::new(vec![
        Cell::from(" "),
        Cell::from(" "),
        Cell::from(" "),
        Cell::from(" "),
        Cell::from(" "),
    ]))
    .chain(blips.iter().skip(row_offset).take(max_rows).map(|blip| {
        Row::new(vec![
            Cell::from(blip.name.clone()),
            Cell::from(
                blip.quadrant
                    .clone()
                    .unwrap_or_else(|| "(none)".to_string()),
            ),
            Cell::from(blip.ring.clone().unwrap_or_else(|| "(none)".to_string())),
            Cell::from(blip.tag.clone().unwrap_or_default()),
            Cell::from(if blip.has_adr { "Yes" } else { "No" }),
        ])
        .style(Style::default().fg(Color::White))
    }));

    let table = Table::new(
        rows,
        [
            Constraint::Length(20),
            Constraint::Length(12),
            Constraint::Length(8),
            Constraint::Length(14),
            Constraint::Length(8),
        ],
    )
    .header(header)
    .column_spacing(1);

    f.render_widget(table, area);

    let mut scrollbar_state = ScrollbarState::new(blips.len())
        .position(row_offset)
        .viewport_content_length(max_rows.min(area.height.saturating_sub(1) as usize));
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .thumb_style(Style::default().fg(Color::Rgb(0, 0, 238)));
    let scroll_area = Rect {
        x: area.x,
        y: area.y.saturating_add(1),
        width: area.width,
        height: area.height.saturating_sub(1),
    };
    f.render_stateful_widget(scrollbar, scroll_area, &mut scrollbar_state);
}

fn render_all_adrs(
    export: &RadarExport,
    row_offset: usize,
    f: &mut ratzilla::ratatui::Frame<'_>,
    area: Rect,
) {
    if export.adrs.is_empty() {
        let paragraph = Paragraph::new("No ADRs available")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray));
        f.render_widget(paragraph, area);
        return;
    }

    let header = Row::new(vec![
        Cell::from("Title"),
        Cell::from("Blip"),
        Cell::from("Status"),
        Cell::from("Date"),
    ])
    .style(
        Style::default()
            .fg(Color::Rgb(0, 0, 238))
            .bg(Color::Rgb(200, 200, 200))
            .add_modifier(Modifier::BOLD),
    );

    let rows = std::iter::once(Row::new(vec![
        Cell::from(" "),
        Cell::from(" "),
        Cell::from(" "),
        Cell::from(" "),
    ]))
    .chain(export.adrs.iter().skip(row_offset).take(18).map(|adr| {
        Row::new(vec![
            Cell::from(adr.title.clone()),
            Cell::from(adr.blip_name.clone()),
            Cell::from(adr.status.clone()),
            Cell::from(adr.timestamp.clone()),
        ])
        .style(Style::default().fg(Color::White))
    }));

    let table = Table::new(
        rows,
        [
            Constraint::Length(24),
            Constraint::Length(14),
            Constraint::Length(10),
            Constraint::Length(16),
        ],
    )
    .header(header)
    .column_spacing(1);

    f.render_widget(table, area);

    let mut scrollbar_state = ScrollbarState::new(export.adrs.len())
        .position(row_offset)
        .viewport_content_length(18.min(area.height.saturating_sub(1) as usize));
    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .thumb_style(Style::default().fg(Color::Rgb(0, 0, 238)));
    let scroll_area = Rect {
        x: area.x,
        y: area.y.saturating_add(1),
        width: area.width,
        height: area.height.saturating_sub(1),
    };
    f.render_stateful_widget(scrollbar, scroll_area, &mut scrollbar_state);
}

fn quadrant_color(quadrant: Option<&str>) -> Color {
    match quadrant {
        Some("platforms") => Color::Rgb(0, 0, 238),
        Some("languages") => Color::Cyan,
        Some("tools") => Color::Yellow,
        Some("techniques") => Color::Magenta,
        _ => Color::Gray,
    }
}

async fn fetch_radar(store: Rc<RefCell<Option<RadarExport>>>) {
    let Some(window) = web_sys::window() else {
        return;
    };

    let opts = RequestInit::new();
    opts.set_method("GET");
    opts.set_mode(RequestMode::SameOrigin);

    let Ok(request) = Request::new_with_str_and_init("radar.json", &opts) else {
        return;
    };

    let Ok(response_value) =
        wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await
    else {
        return;
    };

    let Ok(response) = response_value.dyn_into::<Response>() else {
        web_sys::console::error_1(&"Failed to read response".into());
        return;
    };

    let Ok(json) = wasm_bindgen_futures::JsFuture::from(response.json().unwrap()).await else {
        web_sys::console::error_1(&"Failed to read radar.json body".into());
        return;
    };

    let data = match serde_wasm_bindgen::from_value::<RadarExport>(json) {
        Ok(data) => data,
        Err(error) => {
            web_sys::console::error_1(&format!("Failed to parse radar.json: {error}").into());
            return;
        }
    };

    *store.borrow_mut() = Some(data);
}
