use crate::app::{AdrStatus, App, InputMode, InputState};
use crate::ui::widgets::charts::{render_chart_panel, render_chart_tabs};
use crate::ui::widgets::radar::{render_full_radar, render_mini_radar, render_radar};
use crate::{Quadrant, Ring};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line as TextLine, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

pub fn render_main(app: &App, f: &mut Frame<'_>) {
    let main_layout = build_main_layout(app, f);

    if app.show_help {
        render_help(f, main_layout[0]);
        return;
    }

    render_title_section(app, f, main_layout[0]);
    render_content_section(app, f, main_layout[1]);
    render_status_section(app, f, main_layout[2]);
    render_shortcuts(f, main_layout[3]);
}

fn build_main_layout(app: &App, f: &Frame<'_>) -> Vec<Rect> {
    if app.show_help {
        return Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)])
            .split(f.area().inner(Margin::new(2, 1)))
            .to_vec();
    }

    Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6), // Title area
            Constraint::Min(5),    // Content area
            Constraint::Length(3), // Status area
            Constraint::Length(1), // Shortcuts hint
        ])
        .split(f.area().inner(Margin::new(2, 1)))
        .to_vec()
}

fn render_title_section(app: &App, f: &mut Frame<'_>, area: Rect) {
    let title_block = Block::default()
        .title("== Tech Radar ADR Generator ==")
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    f.render_widget(title_block, area);

    let title_inner = area.inner(Margin::new(1, 1));
    let title_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(title_inner);

    let title_paragraph = Paragraph::new(Text::from(vec![TextLine::from(vec![
        Span::styled(
            "Tech Radar ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "ADR Generator",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ])]))
    .alignment(Alignment::Left);
    f.render_widget(title_paragraph, title_chunks[0]);

    render_mini_radar(f, title_chunks[1], app.animation_counter);
}

fn render_content_section(app: &App, f: &mut Frame<'_>, area: Rect) {
    let content_block = Block::default()
        .title(" Input ")
        .title_style(Style::default().fg(Color::Green))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    let input_prompt = prompt_line(&app.input_state);
    let mode_text = mode_text_line(app.input_mode);

    let cursor = cursor_char(&app.input_state, app.animation_counter);
    let input_text = input_line(&app.current_input, cursor);
    let info_lines = entry_info_lines(app);

    let mut content_lines = vec![
        TextLine::from(Span::styled(
            input_prompt,
            Style::default().fg(Color::Green),
        )),
        TextLine::from(mode_text),
    ];

    append_input_state_lines(app, input_text, &mut content_lines);

    if !info_lines.is_empty() {
        content_lines.push(TextLine::from(""));
        content_lines.extend(info_lines);

        if app.input_mode == Some(InputMode::Blip)
            && app.blip_data.quadrant.is_some()
            && app.blip_data.ring.is_some()
        {
            content_lines.push(TextLine::from(""));
            content_lines.push(TextLine::from(Span::styled(
                "Position in Radar:",
                Style::default().fg(Color::Gray),
            )));
        }
    }

    let content_paragraph = Paragraph::new(Text::from(content_lines))
        .block(content_block)
        .wrap(Wrap { trim: true });

    let content_inner = area.inner(Margin::new(1, 1));
    let horizontal_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(content_inner);

    let left_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(9), Constraint::Min(8)])
        .split(horizontal_split[0]);

    f.render_widget(content_paragraph, left_split[0]);
    render_full_radar(app, f, left_split[1]);
    render_side_panel(app, f, horizontal_split[1]);
}

fn append_input_state_lines<'a>(
    app: &App,
    input_text: TextLine<'a>,
    content_lines: &mut Vec<TextLine<'a>>,
) {
    match app.input_state {
        InputState::WaitingForCommand => {
            content_lines.extend(mode_selection_lines(app.input_mode_selection_index));
        }
        InputState::ChoosingAdrStatus => {
            content_lines.extend(adr_status_selection_lines(app.adr_status_selection_index));
        }
        InputState::ChoosingQuadrant => {
            content_lines.extend(quadrant_selection_lines(app.quadrant_selection_index));
        }
        InputState::ChoosingRing => {
            content_lines.extend(ring_selection_lines(app.ring_selection_index));
        }
        _ => {
            content_lines.push(input_text);
        }
    }
}

fn entry_info_lines(app: &App) -> Vec<TextLine<'_>> {
    if app.blip_data.name.is_empty() {
        return Vec::new();
    }

    let name_style = Style::default().fg(Color::White);
    let label_style = Style::default().fg(Color::Gray);
    let value_style = Style::default().fg(Color::Yellow);

    let mut lines = vec![info_line(
        "Technology",
        app.blip_data.name.as_str(),
        label_style,
        name_style,
    )];

    if app.input_mode == Some(InputMode::Adr) {
        let status_label = app.adr_status.map_or("Not selected", AdrStatus::as_str);
        lines.push(info_line(
            "ADR Status",
            status_label,
            label_style,
            value_style,
        ));
        return lines;
    }

    lines.extend([
        info_line(
            "Quadrant",
            &app.blip_data
                .quadrant
                .map_or_else(|| "Not selected".to_string(), |q| q.as_str().to_string()),
            label_style,
            value_style,
        ),
        info_line(
            "Ring",
            &app.blip_data
                .ring
                .map_or_else(|| "Not selected".to_string(), |r| r.as_str().to_string()),
            label_style,
            value_style,
        ),
    ]);

    lines
}

fn render_side_panel(app: &App, f: &mut Frame<'_>, area: Rect) {
    match app.input_state {
        InputState::WaitingForCommand => render_charts_panel(app, f, area),
        _ if app.blip_data.quadrant.is_some() && app.blip_data.ring.is_some() => {
            render_mini_selection_radar(app, f, area);
        }
        _ => {}
    }
}

fn render_charts_panel(app: &App, f: &mut Frame<'_>, area: Rect) {
    let right_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Percentage(97)])
        .split(area);

    render_chart_tabs(app, f, right_split[0]);
    render_chart_panel(app, f, right_split[1]);
}

fn render_mini_selection_radar(app: &App, f: &mut Frame<'_>, area: Rect) {
    let radar_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width.min(20),
        height: area.height.min(20),
    };

    if radar_area.height >= 5 {
        render_radar(
            f,
            radar_area,
            app.blip_data.quadrant,
            app.blip_data.ring,
            app.animation_counter,
        );
    }
}

fn render_status_section(app: &App, f: &mut Frame<'_>, area: Rect) {
    let status_block = Block::default()
        .title(" Status ")
        .title_style(Style::default().fg(Color::Yellow))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let status_text = if app.status_message.is_empty() {
        Text::from(Span::styled(
            if app.animation_paused {
                "Animation paused"
            } else {
                ""
            },
            Style::default().fg(Color::Gray),
        ))
    } else {
        let style = if app.status_message.starts_with("Error") {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        };

        Text::from(Span::styled(&app.status_message, style))
    };

    let status_paragraph = Paragraph::new(status_text)
        .block(status_block)
        .wrap(Wrap { trim: true });
    f.render_widget(status_paragraph, area);
}

fn render_shortcuts(f: &mut Frame<'_>, area: Rect) {
    let shortcuts = shortcuts_line();
    let shortcuts_paragraph = Paragraph::new(shortcuts).alignment(Alignment::Center);
    f.render_widget(shortcuts_paragraph, area);
}

const fn prompt_line(state: &InputState) -> &'static str {
    match state {
        InputState::WaitingForCommand => "Select entry type (Use Left/Right and Enter)",
        InputState::EnteringTechnology => "Enter technology name:",
        InputState::ChoosingAdrStatus => "Choose ADR status (Use Up/Down and Enter):",
        InputState::ChoosingQuadrant => "Choose quadrant (Use Up/Down and Enter):",
        InputState::ChoosingRing => "Choose ring (Use Up/Down and Enter):",
        InputState::GeneratingFile => "Generating file... Please wait",
        InputState::Completed => "File generated! Press 'n' for new entry or 'q' to quit",
    }
}

fn mode_text_line(mode: Option<InputMode>) -> Span<'static> {
    match mode {
        Some(InputMode::Adr) => Span::styled(
            "Mode: ADR",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        ),
        Some(InputMode::Blip) => Span::styled(
            "Mode: Blip",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        None => Span::raw(""),
    }
}

fn cursor_char(state: &InputState, animation_counter: f64) -> &'static str {
    match state {
        InputState::WaitingForCommand
        | InputState::Completed
        | InputState::GeneratingFile
        | InputState::ChoosingAdrStatus => "",
        _ => {
            let blink = (animation_counter * 2.0).sin() > 0.0;
            if blink {
                "█"
            } else {
                " "
            }
        }
    }
}

fn input_line(current_input: &str, cursor: &str) -> TextLine<'static> {
    TextLine::from(Span::styled(
        format!("> {current_input}{cursor}"),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    ))
}

fn mode_selection_lines(selection_index: usize) -> Vec<TextLine<'static>> {
    let mode_items = ["ADR", "Blip"];

    mode_items
        .iter()
        .enumerate()
        .map(|(index, label)| {
            let is_selected = index == selection_index;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let prefix = if is_selected { ">" } else { " " };
            TextLine::from(Span::styled(format!("{prefix} {label}"), style))
        })
        .collect()
}

fn adr_status_selection_lines(selection_index: usize) -> Vec<TextLine<'static>> {
    let status_items = [
        AdrStatus::Proposed,
        AdrStatus::Accepted,
        AdrStatus::Rejected,
        AdrStatus::Deprecated,
        AdrStatus::Superseded,
    ];

    let mut lines = Vec::new();
    for (index, status) in status_items.iter().enumerate() {
        let is_selected = index == selection_index;
        let style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let prefix = if is_selected { ">" } else { " " };
        lines.push(TextLine::from(Span::styled(
            format!("{prefix} {}", status.label()),
            style,
        )));

        if is_selected {
            lines.push(TextLine::from(Span::styled(
                format!("   {}", status.as_str()),
                Style::default().fg(Color::Gray),
            )));
        }
    }

    lines
}

fn quadrant_selection_lines(selection_index: usize) -> Vec<TextLine<'static>> {
    let quadrant_items = [
        Quadrant::Platforms,
        Quadrant::Languages,
        Quadrant::Tools,
        Quadrant::Techniques,
    ];

    let mut lines = Vec::new();
    for (index, quadrant) in quadrant_items.iter().enumerate() {
        let is_selected = index == selection_index;
        let style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let prefix = if is_selected { ">" } else { " " };
        lines.push(TextLine::from(Span::styled(
            format!("{prefix} {}", quadrant.label()),
            style,
        )));

        if is_selected {
            lines.push(TextLine::from(Span::styled(
                format!("   {}", quadrant.as_str()),
                Style::default().fg(Color::Gray),
            )));
        }
    }

    lines
}

fn ring_selection_lines(selection_index: usize) -> Vec<TextLine<'static>> {
    let ring_items = [Ring::Hold, Ring::Assess, Ring::Trial, Ring::Adopt];

    let mut lines = Vec::new();
    for (index, ring) in ring_items.iter().enumerate() {
        let is_selected = index == selection_index;
        let style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };
        let prefix = if is_selected { ">" } else { " " };
        lines.push(TextLine::from(Span::styled(
            format!("{prefix} {}", ring.label()),
            style,
        )));

        if is_selected {
            lines.push(TextLine::from(Span::styled(
                format!("   {}", ring.as_str()),
                Style::default().fg(Color::Gray),
            )));
        }
    }

    lines
}

fn info_line(
    label: &str,
    value: &str,
    label_style: Style,
    value_style: Style,
) -> TextLine<'static> {
    TextLine::from(vec![
        Span::styled(format!("{label}: "), label_style),
        Span::styled(value.to_string(), value_style),
    ])
}

fn shortcuts_line() -> TextLine<'static> {
    TextLine::from(vec![
        Span::styled(
            "F1",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(": Help | ", Style::default().fg(Color::Gray)),
        Span::styled(
            "Space",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(": Pause | ", Style::default().fg(Color::Gray)),
        Span::styled(
            "Esc",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(": Cancel | ", Style::default().fg(Color::Gray)),
        Span::styled(
            "Enter",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(": Confirm | ", Style::default().fg(Color::Gray)),
        Span::styled(
            "a",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(": Create ADR | ", Style::default().fg(Color::Gray)),
        Span::styled(
            "b",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(": Create Blip | ", Style::default().fg(Color::Gray)),
        Span::styled(
            "n",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(": New entry | ", Style::default().fg(Color::Gray)),
        Span::styled(
            "v",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(": View ADRs | ", Style::default().fg(Color::Gray)),
        Span::styled(
            "l",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(": View Blips | ", Style::default().fg(Color::Gray)),
        Span::styled(
            "q",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(": Quit", Style::default().fg(Color::Gray)),
    ])
}

fn render_help(f: &mut Frame<'_>, area: Rect) {
    let help_block = Block::default()
        .title("== Help & Keyboard Shortcuts ==")
        .title_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let help_text = vec![
        TextLine::from(vec![Span::styled(
            "Tech Radar ADR Generator",
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )]),
        TextLine::from(""),
        TextLine::from("This tool helps you create Architectural Decision Records (ADRs) and Blips for your Tech Radar."),
        TextLine::from(""),
        TextLine::from(vec![Span::styled(
            "Keyboard Shortcuts:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        TextLine::from(vec![
            Span::styled("  F1", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" - Toggle this help screen", Style::default()),
        ]),
        TextLine::from(vec![
            Span::styled("  Space", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" - Pause/resume animations", Style::default()),
        ]),
        TextLine::from(vec![
            Span::styled("  Esc", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" - Cancel current input / Go back", Style::default()),
        ]),
        TextLine::from(vec![
            Span::styled("  Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" - Confirm input", Style::default()),
        ]),
        TextLine::from(vec![
            Span::styled("  a", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" - Create ADR", Style::default()),
        ]),
        TextLine::from(vec![
            Span::styled("  b", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" - Create Blip", Style::default()),
        ]),
        TextLine::from(vec![
            Span::styled("  n", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" - New entry (after completion)", Style::default()),
        ]),
        TextLine::from(vec![
            Span::styled("  q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" - Quit application", Style::default()),
        ]),
        TextLine::from(""),
        TextLine::from(vec![Span::styled(
            "Quadrants:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        TextLine::from("  1 - Platforms: Infrastructure, platforms, APIs and services"),
        TextLine::from("  2 - Languages: Programming languages and frameworks"),
        TextLine::from("  3 - Tools: Development, testing and operational tools"),
        TextLine::from("  4 - Techniques: Methods, practices and approaches"),
        TextLine::from(""),
        TextLine::from(vec![Span::styled(
            "Rings:",
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        TextLine::from("  1 - Hold: Technologies we've used but are actively moving away from"),
        TextLine::from("  2 - Assess: Worth exploring with the goal of understanding how it affects us"),
        TextLine::from("  3 - Trial: Worth pursuing, important to understand how to build up this capability"),
        TextLine::from("  4 - Adopt: We feel strongly that the industry should be adopting these items"),
        TextLine::from(""),
        TextLine::from(vec![Span::styled(
            "Press Esc to close this help screen",
            Style::default().fg(Color::Yellow),
        )]),
    ];

    let help_paragraph = Paragraph::new(Text::from(help_text))
        .block(help_block)
        .wrap(Wrap { trim: true });

    f.render_widget(help_paragraph, area);
}
