use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use ratzilla::ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line as TextLine, Span, Text},
    widgets::canvas::{Canvas, Circle, Line as CanvasLine},
    widgets::{Bar, BarChart, BarGroup, Block, Borders, Paragraph, Wrap},
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

#[derive(serde::Deserialize)]
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

    spawn_local(fetch_radar(data.clone()));

    let backend = DomBackend::new()?;
    let terminal = Terminal::new(backend)?;

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
            .border_style(Style::default().fg(Color::Cyan));
        let inner = block.inner(area);
        f.render_widget(block, area);

        let data = data.borrow();
        if let Some(export) = data.as_ref() {
            render_dashboard(export, f, inner);
        } else {
            let paragraph = Paragraph::new(Text::from(TextLine::from("Loading radar.json...")))
                .alignment(Alignment::Center);
            f.render_widget(paragraph, inner);
        }
    });

    Ok(())
}

fn render_dashboard(export: &RadarExport, f: &mut ratzilla::ratatui::Frame<'_>, area: Rect) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(5),
        ])
        .split(area);

    render_header(export, f, main_layout[0]);

    let content = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(main_layout[1]);

    render_radar(export, f, content[0]);

    let charts = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(content[1]);

    render_blip_type_chart(export, f, charts[0]);
    render_ring_chart(export, f, charts[1]);

    render_footer(export, f, main_layout[2]);
}

fn render_header(export: &RadarExport, f: &mut ratzilla::ratatui::Frame<'_>, area: Rect) {
    let total_blips = export.blips.len();
    let total_adrs = export.adrs.len();
    let coverage = if total_blips > 0 {
        let ratio = total_adrs as f64 / total_blips as f64;
        ratio * 100.0
    } else {
        0.0
    };

    let line = TextLine::from(vec![
        Span::styled("Overview", Style::default().fg(Color::Yellow)),
        Span::raw("  "),
        Span::styled(
            format!("Blips: {total_blips}  ADRs: {total_adrs}  Coverage: {coverage:.1}%"),
            Style::default().fg(Color::White),
        ),
    ]);

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

fn render_footer(export: &RadarExport, f: &mut ratzilla::ratatui::Frame<'_>, area: Rect) {
    let total_blips = export.blips.len();
    let total_adrs = export.adrs.len();

    let line = TextLine::from(vec![
        Span::styled("Data", Style::default().fg(Color::Gray)),
        Span::raw("  "),
        Span::raw(format!("{total_blips} blips • {total_adrs} ADRs")),
    ]);

    let paragraph = Paragraph::new(Text::from(line)).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

fn render_radar(export: &RadarExport, f: &mut ratzilla::ratatui::Frame<'_>, area: Rect) {
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
        Canvas::default()
            .paint(|ctx| {
                let width = f64::from(square.width);
                let height = f64::from(square.height);
                let center_x = width / 2.0;
                let center_y = height / 2.0;
                let max_radius = width.min(height) / 2.0 * 0.9;

                for i in 1..=4 {
                    let ring_radius = max_radius * (f64::from(i) / 4.0);
                    ctx.draw(&Circle {
                        x: center_x,
                        y: center_y,
                        radius: ring_radius,
                        color: Color::DarkGray,
                    });
                }

                ctx.draw(&CanvasLine {
                    x1: center_x,
                    y1: center_y - max_radius,
                    x2: center_x,
                    y2: center_y + max_radius,
                    color: Color::DarkGray,
                });
                ctx.draw(&CanvasLine {
                    x1: center_x - max_radius,
                    y1: center_y,
                    x2: center_x + max_radius,
                    y2: center_y,
                    color: Color::DarkGray,
                });

                for (angle, radius, color) in &points {
                    let x = angle.cos().mul_add(max_radius * radius, center_x);
                    let y = angle.sin().mul_add(max_radius * radius, center_y);

                    ctx.draw(&Circle {
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
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    f.render_widget(block, area);

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

    f.render_widget(chart, inner);
}

fn render_ring_chart(export: &RadarExport, f: &mut ratzilla::ratatui::Frame<'_>, area: Rect) {
    let block = Block::default()
        .title("Ring Distribution")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    f.render_widget(block, area);

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
        let width = inner.width.max(1) - 2;
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

    let paragraph = Paragraph::new(Text::from(lines))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, inner);
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
