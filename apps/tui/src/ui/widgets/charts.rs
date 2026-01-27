use crate::app::App;
use crate::ui::widgets::radar::quadrant_color;
use crate::{Quadrant, Ring};
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::symbols::Marker;
use ratatui::text::{Line as TextLine, Span};
use ratatui::widgets::{
    Axis, Bar, BarChart, BarGroup, Block, Borders, Chart, Dataset, GraphType, Paragraph, Tabs,
};
use ratatui::Frame;
use tui_piechart::{PieChart, PieSlice, Resolution};

pub fn render_chart_tabs(app: &App, f: &mut Frame<'_>, area: Rect) {
    let titles = ["Scatter", "Types"]
        .iter()
        .map(|title| TextLine::from(*title))
        .collect::<Vec<_>>();

    let tabs = Tabs::new(titles)
        .select(app.chart_tab_index)
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Rgb(0, 0, 238))
                .add_modifier(Modifier::BOLD),
        )
        .divider(Span::raw("|"));

    f.render_widget(tabs, area);
}

pub fn render_chart_panel(app: &App, f: &mut Frame<'_>, area: Rect) {
    let chart_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(area.inner(Margin::new(0, 1)));

    if app.chart_tab_index == 0 {
        render_blip_scatter(app, f, chart_split[0]);
        render_ring_barchart(app, f, chart_split[1]);
    } else {
        render_blip_barchart(app, f, chart_split[0]);
        render_ring_piechart(app, f, chart_split[1]);
    }
}

pub fn render_blip_scatter(app: &App, f: &mut Frame<'_>, area: Rect) {
    let blips = &app.blips;
    if blips.is_empty() {
        let block = Block::default()
            .title("Blips Chart")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let paragraph = Paragraph::new("No blips available")
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(paragraph, area);
        return;
    }

    let mut platforms = Vec::new();
    let mut languages = Vec::new();
    let mut tools = Vec::new();
    let mut techniques = Vec::new();

    for blip in blips {
        let quadrant = match blip.quadrant {
            Some(Quadrant::Platforms) => 1.0,
            Some(Quadrant::Languages) => 2.0,
            Some(Quadrant::Tools) => 3.0,
            Some(Quadrant::Techniques) => 4.0,
            _ => continue,
        };
        let ring = match blip.ring {
            Some(Ring::Hold) => 1.0,
            Some(Ring::Assess) => 2.0,
            Some(Ring::Trial) => 3.0,
            Some(Ring::Adopt) => 4.0,
            _ => continue,
        };

        match blip.quadrant {
            Some(Quadrant::Platforms) => platforms.push((quadrant, ring)),
            Some(Quadrant::Languages) => languages.push((quadrant, ring)),
            Some(Quadrant::Tools) => tools.push((quadrant, ring)),
            Some(Quadrant::Techniques) => techniques.push((quadrant, ring)),
            _ => {}
        }
    }

    let datasets = vec![
        Dataset::default()
            .name("Platforms")
            .marker(Marker::Dot)
            .graph_type(GraphType::Scatter)
            .style(Style::default().fg(quadrant_color("platforms")))
            .data(&platforms),
        Dataset::default()
            .name("Languages")
            .marker(Marker::Dot)
            .graph_type(GraphType::Scatter)
            .style(Style::default().fg(quadrant_color("languages")))
            .data(&languages),
        Dataset::default()
            .name("Tools")
            .marker(Marker::Dot)
            .graph_type(GraphType::Scatter)
            .style(Style::default().fg(quadrant_color("tools")))
            .data(&tools),
        Dataset::default()
            .name("Techniques")
            .marker(Marker::Dot)
            .graph_type(GraphType::Scatter)
            .style(Style::default().fg(quadrant_color("techniques")))
            .data(&techniques),
    ];

    let x_labels = vec![
        Span::raw("Platforms"),
        Span::raw("Languages"),
        Span::raw("Tools"),
        Span::raw("Techniques"),
    ];
    let y_labels = vec![
        Span::raw("Hold"),
        Span::raw("Assess"),
        Span::raw("Trial"),
        Span::raw("Adopt"),
    ];

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title("Blips by Quadrant / Ring")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .x_axis(
            Axis::default()
                .title("Quadrant")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.5, 4.5])
                .labels(x_labels),
        )
        .y_axis(
            Axis::default()
                .title("Ring")
                .style(Style::default().fg(Color::Gray))
                .bounds([0.5, 4.5])
                .labels(y_labels),
        );

    f.render_widget(chart, area);
}

pub fn render_blip_barchart(app: &App, f: &mut Frame<'_>, area: Rect) {
    if app.blips.is_empty() {
        let block = Block::default()
            .title("Blip Types")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let paragraph = Paragraph::new("No blips available")
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(paragraph, area);
        return;
    }

    let mut counts = [0_u64; 4];
    for blip in &app.blips {
        let index = match blip.quadrant {
            Some(Quadrant::Platforms) => 0,
            Some(Quadrant::Languages) => 1,
            Some(Quadrant::Tools) => 2,
            Some(Quadrant::Techniques) => 3,
            _ => continue,
        };
        counts[index] += 1;
    }

    let labels = ["Platforms", "Languages", "Tools", "Techniques"];
    let bar_colors = [
        quadrant_color(Quadrant::Platforms.as_str()),
        quadrant_color(Quadrant::Languages.as_str()),
        quadrant_color(Quadrant::Tools.as_str()),
        quadrant_color(Quadrant::Techniques.as_str()),
    ];

    let bars: Vec<Bar<'_>> = counts
        .iter()
        .enumerate()
        .map(|(index, value)| {
            Bar::default()
                .value(*value)
                .label(TextLine::from(labels[index]))
                .style(Style::default().fg(bar_colors[index]))
                .value_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        })
        .collect();

    let max_value = counts.iter().copied().max().unwrap_or(0).max(1);

    let chart = BarChart::default()
        .block(
            Block::default()
                .title("Blip Types")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .data(BarGroup::default().bars(&bars))
        .max(max_value)
        .bar_gap(0)
        .bar_width(6);

    f.render_widget(chart, area);
}

pub fn render_ring_barchart(app: &App, f: &mut Frame<'_>, area: Rect) {
    if app.blips.is_empty() {
        let block = Block::default()
            .title("Ring Counts")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let paragraph = Paragraph::new("No blips available")
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(paragraph, area);
        return;
    }

    let mut counts = [0_u64; 4];
    for blip in &app.blips {
        let index = match blip.ring {
            Some(Ring::Hold) => 0,
            Some(Ring::Assess) => 1,
            Some(Ring::Trial) => 2,
            Some(Ring::Adopt) => 3,
            _ => continue,
        };
        counts[index] += 1;
    }

    let labels = ["Hold", "Assess", "Trial", "Adopt"];
    let colors = [Color::Gray, Color::Cyan, Color::Yellow, Color::Rgb(0, 0, 238)];

    let bars: Vec<Bar<'_>> = counts
        .iter()
        .enumerate()
        .map(|(index, value)| {
            Bar::default()
                .value(*value)
                .label(TextLine::from(labels[index]))
                .style(Style::default().fg(colors[index]))
                .value_style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
        })
        .collect();

    let max_value = counts.iter().copied().max().unwrap_or(0).max(1);

    let chart = BarChart::default()
        .block(
            Block::default()
                .title("Ring Counts")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .data(BarGroup::default().bars(&bars))
        .max(max_value)
        .bar_gap(0)
        .bar_width(6);

    f.render_widget(chart, area);
}

pub fn render_ring_piechart(app: &App, f: &mut Frame<'_>, area: Rect) {
    if app.blips.is_empty() {
        let block = Block::default()
            .title("Ring Distribution")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let paragraph = Paragraph::new("No blips available")
            .block(block)
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(paragraph, area);
        return;
    }

    let mut counts = [0_f64; 4];
    for blip in &app.blips {
        let index = match blip.ring {
            Some(Ring::Hold) => 0,
            Some(Ring::Assess) => 1,
            Some(Ring::Trial) => 2,
            Some(Ring::Adopt) => 3,
            _ => continue,
        };
        counts[index] += 1.0;
    }

    let slices = vec![
        PieSlice::new("Hold", counts[0], Color::Gray),
        PieSlice::new("Assess", counts[1], Color::Cyan),
        PieSlice::new("Trial", counts[2], Color::Yellow),
        PieSlice::new("Adopt", counts[3], Color::Rgb(0, 0, 238)),
    ];

    let chart = PieChart::new(slices)
        .block(
            Block::default()
                .title("Ring Distribution")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .show_legend(true)
        .show_percentages(true)
        .resolution(Resolution::Braille);

    f.render_widget(chart, area);
}
