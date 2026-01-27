use crate::{Quadrant, Ring};
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::widgets::canvas::{Canvas, Circle, Line as CanvasLine};
use ratatui::Frame;

pub fn quadrant_color(quadrant: &str) -> Color {
    match quadrant {
        "platforms" => Color::Rgb(0, 0, 238),
        "languages" => Color::Cyan,
        "tools" => Color::Yellow,
        "techniques" => Color::Magenta,
        _ => Color::Gray,
    }
}


pub fn render_mini_radar(f: &mut Frame<'_>, area: Rect, animation: f64) {
    if area.width < 4 || area.height < 4 {
        return;
    }

    let size = area.width.min(area.height);
    let square = Rect {
        x: area.x + (area.width - size) / 2,
        y: area.y + (area.height - size) / 2,
        width: size,
        height: size,
    };

    f.render_widget(
        Canvas::default()
            .paint(|ctx| {
                let width = f64::from(square.width);
                let height = f64::from(square.height);
                let center_x = width / 2.0;
                let center_y = height / 2.0;
                let radius = width.min(height) / 2.0 * 0.8;

                for i in 1..=3 {
                    let ring_radius = radius * (f64::from(i) / 3.0);
                    ctx.draw(&Circle {
                        x: center_x,
                        y: center_y,
                        radius: ring_radius,
                        color: Color::DarkGray,
                    });
                }

                ctx.draw(&CanvasLine {
                    x1: center_x,
                    y1: center_y - radius,
                    x2: center_x,
                    y2: center_y + radius,
                    color: Color::DarkGray,
                });
                ctx.draw(&CanvasLine {
                    x1: center_x - radius,
                    y1: center_y,
                    x2: center_x + radius,
                    y2: center_y,
                    color: Color::DarkGray,
                });

                let angle = animation * 2.0 * std::f64::consts::PI;
                let sweep_x = angle.cos().mul_add(radius, center_x);
                let sweep_y = angle.sin().mul_add(radius, center_y);

                let ghost_angle = angle + (std::f64::consts::PI / 18.0);
                let ghost_x = ghost_angle.cos().mul_add(radius * 0.92, center_x);
                let ghost_y = ghost_angle.sin().mul_add(radius * 0.92, center_y);

                ctx.draw(&CanvasLine {
                    x1: center_x,
                    y1: center_y,
                    x2: ghost_x,
                    y2: ghost_y,
                    color: Color::LightCyan,
                });

                ctx.draw(&CanvasLine {
                    x1: center_x,
                    y1: center_y,
                    x2: sweep_x,
                    y2: sweep_y,
                    color: Color::Cyan,
                });

                ctx.draw(&Circle {
                    x: center_x,
                    y: center_y,
                    radius: radius * 0.08,
                    color: Color::Cyan,
                });
            })
            .x_bounds([0.0, f64::from(square.width)])
            .y_bounds([0.0, f64::from(square.height)]),
        square,
    );
}

pub fn render_full_radar(app: &crate::app::App, f: &mut Frame<'_>, area: Rect) {
    let blips = &app.blips;
    if area.width < 8 || area.height < 6 {
        return;
    }

    let block = ratatui::widgets::Block::default()
        .title("Tech Radar")
        .borders(ratatui::widgets::Borders::ALL)
        .border_style(ratatui::style::Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if blips.is_empty() {
        let paragraph = ratatui::widgets::Paragraph::new("No blips available")
            .alignment(ratatui::layout::Alignment::Center)
            .style(ratatui::style::Style::default().fg(Color::Gray));
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

    let points = blips
        .iter()
        .filter_map(|blip| {
            let quadrant = match blip.quadrant {
                Some(crate::Quadrant::Platforms) => 0,
                Some(crate::Quadrant::Languages) => 1,
                Some(crate::Quadrant::Tools) => 2,
                Some(crate::Quadrant::Techniques) => 3,
                _ => return None,
            };
            let ring = match blip.ring {
                Some(crate::Ring::Adopt) => 0,
                Some(crate::Ring::Trial) => 1,
                Some(crate::Ring::Assess) => 2,
                Some(crate::Ring::Hold) => 3,
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

            Some((blip, angle, radius))
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

                let pulse = (app.animation_counter * 0.6).sin().mul_add(0.5, 0.5);
                let pulse_radius = max_radius * (0.45 + pulse * 0.5);
                ctx.draw(&Circle {
                    x: center_x,
                    y: center_y,
                    radius: pulse_radius,
                    color: Color::LightCyan,
                });

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

                let sweep_angle = app.animation_counter * 1.4;
                let sweep_x = sweep_angle.cos().mul_add(max_radius, center_x);
                let sweep_y = sweep_angle.sin().mul_add(max_radius, center_y);
                ctx.draw(&CanvasLine {
                    x1: center_x,
                    y1: center_y,
                    x2: sweep_x,
                    y2: sweep_y,
                    color: Color::LightCyan,
                });

                let ghost_angle = sweep_angle + (std::f64::consts::PI / 20.0);
                let ghost_x = ghost_angle.cos().mul_add(max_radius * 0.92, center_x);
                let ghost_y = ghost_angle.sin().mul_add(max_radius * 0.92, center_y);
                ctx.draw(&CanvasLine {
                    x1: center_x,
                    y1: center_y,
                    x2: ghost_x,
                    y2: ghost_y,
                    color: Color::DarkGray,
                });

                for (blip, angle, radius) in &points {
                    let color = blip
                        .quadrant
                        .map_or(Color::Gray, |quadrant| quadrant_color(quadrant.as_str()));
                    let x = angle.cos().mul_add(max_radius * radius, center_x);
                    let y = angle.sin().mul_add(max_radius * radius, center_y);

                    ctx.draw(&Circle {
                        x,
                        y,
                        radius: max_radius * 0.035,
                        color,
                    });
                }
            })
            .x_bounds([0.0, f64::from(square.width)])
            .y_bounds([0.0, f64::from(square.height)]),
        square,
    );
}

#[allow(dead_code)]
pub fn render_radar(
    f: &mut Frame<'_>,
    area: Rect,
    quadrant: Option<Quadrant>,
    ring: Option<Ring>,
    animation: f64,
) {
    f.render_widget(
        Canvas::default()
            .paint(|ctx| {
                let width = f64::from(area.width);
                let height = f64::from(area.height);
                let min_dimension = width.min(height);
                let center_x = width / 2.0;
                let center_y = height / 2.0;
                let max_radius = min_dimension / 2.0 * 0.9;

                for i in 1..=4 {
                    let ring_radius = max_radius * (f64::from(i) / 4.0);
                    ctx.draw(&Circle {
                        x: center_x,
                        y: center_y,
                        radius: ring_radius,
                        color: Color::Gray,
                    });
                }

                ctx.draw(&CanvasLine {
                    x1: center_x,
                    y1: center_y - max_radius,
                    x2: center_x,
                    y2: center_y + max_radius,
                    color: Color::Gray,
                });
                ctx.draw(&CanvasLine {
                    x1: center_x - max_radius,
                    y1: center_y,
                    x2: center_x + max_radius,
                    y2: center_y,
                    color: Color::Gray,
                });

                if let (Some(q), Some(r)) = (quadrant, ring) {
                    let quadrant_idx = match q {
                        Quadrant::Tools => 0,
                        Quadrant::Techniques => 1,
                        Quadrant::Platforms => 2,
                        Quadrant::Languages => 3,
                    };

                    let ring_idx = match r {
                        Ring::Adopt => 0,
                        Ring::Trial => 1,
                        Ring::Assess => 2,
                        Ring::Hold => 3,
                    };

                    let angle = std::f64::consts::PI / 4.0
                        + (f64::from(quadrant_idx) * std::f64::consts::PI / 2.0);
                    let radius = max_radius * ((f64::from(ring_idx) + 0.5) / 4.0);

                    let pulse = (animation * 3.0).sin().mul_add(0.2, 0.8);
                    let blip_radius = max_radius * 0.05 * pulse;

                    let x = angle.cos().mul_add(radius, center_x);
                    let y = angle.sin().mul_add(radius, center_y);

                    ctx.draw(&Circle {
                        x,
                        y,
                        radius: blip_radius,
                        color: Color::Yellow,
                    });
                }
            })
            .x_bounds([0.0, f64::from(area.width)])
            .y_bounds([0.0, f64::from(area.height)]),
        area,
    );
}
