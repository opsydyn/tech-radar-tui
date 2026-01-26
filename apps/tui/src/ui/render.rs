use crate::app::state::EditField;
use crate::app::{state::AppScreen, App, InputMode, InputState};
use crate::{Quadrant, Ring};

fn quadrant_color(quadrant: &str) -> Color {
    match quadrant {
        "platforms" => Color::Rgb(0, 0, 238),
        "languages" => Color::Cyan,
        "tools" => Color::Yellow,
        "techniques" => Color::Magenta,
        _ => Color::Gray,
    }
}

fn quadrant_color_from_option(value: Option<&str>) -> Color {
    value.map_or(Color::Gray, quadrant_color)
}
use ratatui::widgets::canvas::{Canvas, Circle, Line as CanvasLine};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::{Line as TextLine, Span, Text},
    widgets::{
        Axis, Bar, BarChart, BarGroup, Block, Borders, Cell, Chart, Dataset, GraphType, Paragraph,
        Row, Table, Tabs, Wrap,
    },
    Frame,
};
use tui_piechart::{PieChart, PieSlice, Resolution};

#[allow(clippy::cognitive_complexity)]
pub fn ui(app: &App, f: &mut Frame<'_>) {
    if app.screen == AppScreen::ViewBlips {
        render_blips_view(app, f);
        return;
    }

    if app.screen == AppScreen::ViewAdrs {
        render_adrs_view(app, f);
        return;
    }

    if app.screen == AppScreen::BlipDetails {
        render_blip_details(app, f);
        return;
    }

    if app.screen == AppScreen::BlipActions {
        let area = f.area();

        // Get the selected blip
        if let Some(selected_blip) = app.blips.get(app.selected_blip_index) {
            // Create a centered box for the actions menu
            let action_area = Rect {
                x: area.width.saturating_sub(50) / 2,
                y: area.height.saturating_sub(10) / 2,
                width: 50.min(area.width),
                height: 10.min(area.height),
            };

            // Create the actions menu block
            let block = Block::default()
                .title(format!("Actions for Blip: {}", selected_blip.name))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow));

            // Define the actions
            let actions = [
                "View details",
                if selected_blip.has_adr {
                    "View ADR"
                } else {
                    "Generate ADR"
                },
                "Edit blip",
                "Back to list",
            ];

            // Create the text for the actions
            let action_text = actions
                .iter()
                .enumerate()
                .map(|(i, &action)| {
                    let is_selected = i == app.blip_action_index;
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

            // Create the paragraph with the actions
            let paragraph = Paragraph::new(action_text)
                .block(block)
                .alignment(Alignment::Left);

            // Render the actions menu
            f.render_widget(paragraph, action_area);

            // Render help text at the bottom
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

        return;
    }

    if app.screen == AppScreen::EditBlip {
        let area = f.area();

        // Get the selected blip and edit state
        if let (Some(selected_blip), Some(edit_state)) =
            (app.blips.get(app.selected_blip_index), &app.edit_blip_state)
        {
            // Create a centered box for the edit form
            let form_area = Rect {
                x: area.width.saturating_sub(60) / 2,
                y: area.height.saturating_sub(15) / 2,
                width: 60.min(area.width),
                height: 15.min(area.height),
            };

            // Create the form block
            let block = Block::default()
                .title(format!("Edit Blip: {}", selected_blip.name))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow));

            f.render_widget(block, form_area);

            // Create layout for form fields
            let form_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(1), // Name
                    Constraint::Length(1), // Ring
                    Constraint::Length(1), // Quadrant
                    Constraint::Length(1), // Tag
                    Constraint::Length(3), // Description
                    Constraint::Length(1), // Spacer
                    Constraint::Length(1), // Help text
                ])
                .split(form_area);

            // Helper function to determine if a field is selected and its style
            let field_style = |field: EditField| {
                let is_selected = edit_state.field == field;
                let is_editing = is_selected && edit_state.editing;

                if is_editing {
                    // Editing mode - bright white on blue
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Blue)
                        .add_modifier(Modifier::BOLD)
                } else if is_selected {
                    // Selected but not editing - yellow
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    // Not selected - normal text
                    Style::default()
                }
            };

            // Helper function to create field label
            let field_label = |name: &str, field: EditField| {
                let style = field_style(field);
                let prefix = if edit_state.field == field && edit_state.editing {
                    "► "
                } else if edit_state.field == field {
                    "> "
                } else {
                    "  "
                };

                Span::styled(format!("{prefix}{name}: "), style)
            };

            // Name field
            let name_text = TextLine::from(vec![
                field_label("Name", EditField::Name),
                Span::styled(&edit_state.name, field_style(EditField::Name)),
            ]);
            f.render_widget(Paragraph::new(name_text), form_chunks[0]);

            // Ring field
            let ring_text = TextLine::from(vec![
                field_label("Ring", EditField::Ring),
                Span::styled(&edit_state.ring, field_style(EditField::Ring)),
            ]);
            f.render_widget(Paragraph::new(ring_text), form_chunks[1]);

            // Quadrant field
            let quadrant_text = TextLine::from(vec![
                field_label("Quadrant", EditField::Quadrant),
                Span::styled(&edit_state.quadrant, field_style(EditField::Quadrant)),
            ]);
            f.render_widget(Paragraph::new(quadrant_text), form_chunks[2]);

            // Tag field
            let tag_text = TextLine::from(vec![
                field_label("Tag", EditField::Tag),
                Span::styled(&edit_state.tag, field_style(EditField::Tag)),
            ]);
            f.render_widget(Paragraph::new(tag_text), form_chunks[3]);

            // Description field
            let description_label = field_label("Description", EditField::Description);
            let description_value =
                Span::styled(&edit_state.description, field_style(EditField::Description));

            let description_text = Text::from(vec![
                TextLine::from(vec![description_label]),
                TextLine::from(vec![description_value]),
            ]);
            f.render_widget(Paragraph::new(description_text), form_chunks[4]);

            // Status line - show editing instructions
            let status_text = if edit_state.editing {
                match edit_state.field {
                    EditField::Ring | EditField::Quadrant => {
                        "Editing mode: ←/→ to cycle through options, Enter to confirm, Esc to cancel"
                    }
                    _ => "Editing mode: Type to edit, Enter to confirm, Esc to cancel",
                }
            } else {
                "Navigation mode: ↑/↓ to select field, Enter to edit, Esc to exit"
            };

            let status_line = Paragraph::new(status_text)
                .alignment(Alignment::Center)
                .style(Style::default().fg(Color::Gray));

            f.render_widget(status_line, form_chunks[5]);

            // Help text
            let help_text = TextLine::from(vec![
                Span::styled(
                    "ESC",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Cancel   "),
                Span::styled(
                    "S",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(": Save Changes"),
            ]);
            f.render_widget(
                Paragraph::new(help_text).alignment(Alignment::Center),
                form_chunks[6],
            );
        }

        return;
    }

    // Update layout to include help section
    let main_layout = if app.show_help {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)])
            .split(f.area().inner(Margin::new(2, 1)))
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(6), // Title area
                Constraint::Min(5),    // Content area
                Constraint::Length(3), // Status area
                Constraint::Length(1), // Shortcuts hint
            ])
            .split(f.area().inner(Margin::new(2, 1)))
    };

    if app.show_help {
        render_help(f, main_layout[0]);
        return;
    }

    // Title block with styled border
    let title_block = Block::default()
        .title("== Tech Radar ADR Generator ==")
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    // Content block with styled border
    let content_block = Block::default()
        .title(" Input ")
        .title_style(Style::default().fg(Color::Green))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    // Status block with styled border
    let status_block = Block::default()
        .title(" Status ")
        .title_style(Style::default().fg(Color::Yellow))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    // Create styled text for the prompt
    let input_prompt = match app.input_state {
        InputState::WaitingForCommand => "Select entry type (Use Left/Right and Enter)",
        InputState::EnteringTechnology => "Enter technology name:",
        InputState::ChoosingQuadrant => "Choose quadrant (Use Up/Down and Enter):",
        InputState::ChoosingRing => "Choose ring (Use Up/Down and Enter):",
        InputState::GeneratingFile => "Generating file... Please wait",
        InputState::Completed => "File generated! Press 'n' for new entry or 'q' to quit",
    };

    // Create styled text for the mode
    let mode_text = match app.input_mode {
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
    };

    // Create styled text for the input with cursor animation
    let cursor = if app.input_state != InputState::WaitingForCommand
        && app.input_state != InputState::Completed
        && app.input_state != InputState::GeneratingFile
    {
        let blink = (app.animation_counter * 2.0).sin() > 0.0;
        if blink {
            "█"
        } else {
            " "
        }
    } else {
        ""
    };

    let input_text = Span::styled(
        format!("> {}{}", app.current_input, cursor),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );

    // Create styled text for blip info
    let blip_info = if app.blip_data.name.is_empty() {
        vec![]
    } else {
        let name_style = Style::default().fg(Color::White);
        let label_style = Style::default().fg(Color::Gray);
        let value_style = Style::default().fg(Color::Yellow);

        vec![
            TextLine::from(vec![
                Span::styled("Technology: ", label_style),
                Span::styled(&app.blip_data.name, name_style),
            ]),
            TextLine::from(vec![
                Span::styled("Quadrant: ", label_style),
                Span::styled(
                    app.blip_data
                        .quadrant
                        .map_or_else(|| "Not selected".to_string(), |q| q.as_str().to_string()),
                    value_style,
                ),
            ]),
            TextLine::from(vec![
                Span::styled("Ring: ", label_style),
                Span::styled(
                    app.blip_data
                        .ring
                        .map_or_else(|| "Not selected".to_string(), |r| r.as_str().to_string()),
                    value_style,
                ),
            ]),
        ]
    };

    // Create styled text for status message
    let status_text = if app.status_message.is_empty() {
        Text::from("")
    } else {
        let style = if app.status_message.starts_with("Error") {
            Style::default().fg(Color::Red)
        } else {
            Style::default().fg(Color::Green)
        };

        Text::from(Span::styled(&app.status_message, style))
    };

    // Render title area with animated radar
    let title_area = main_layout[0];
    f.render_widget(title_block, title_area);

    // Split title area for text and radar
    let title_inner = title_area.inner(Margin::new(1, 1));
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

    // Render mini radar in title area
    render_mini_radar(f, title_chunks[1], app.animation_counter);

    // Render content area
    let mut content_lines = vec![
        TextLine::from(Span::styled(
            input_prompt,
            Style::default().fg(Color::Green),
        )),
        TextLine::from(mode_text),
    ];

    if app.input_state == InputState::WaitingForCommand {
        let mode_items = ["ADR", "Blip"];

        for (index, label) in mode_items.iter().enumerate() {
            let is_selected = index == app.input_mode_selection_index;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let prefix = if is_selected { ">" } else { " " };
            content_lines.push(TextLine::from(Span::styled(
                format!("{prefix} {label}"),
                style,
            )));
        }
    } else if app.input_state == InputState::ChoosingQuadrant {
        let quadrant_items = [
            Quadrant::Platforms,
            Quadrant::Languages,
            Quadrant::Tools,
            Quadrant::Techniques,
        ];

        for (index, quadrant) in quadrant_items.iter().enumerate() {
            let is_selected = index == app.quadrant_selection_index;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let prefix = if is_selected { ">" } else { " " };
            content_lines.push(TextLine::from(Span::styled(
                format!("{} {}", prefix, quadrant.label()),
                style,
            )));

            if is_selected {
                content_lines.push(TextLine::from(Span::styled(
                    format!("   {}", quadrant.as_str()),
                    Style::default().fg(Color::Gray),
                )));
            }
        }
    } else if app.input_state == InputState::ChoosingRing {
        let ring_items = [Ring::Hold, Ring::Assess, Ring::Trial, Ring::Adopt];

        for (index, ring) in ring_items.iter().enumerate() {
            let is_selected = index == app.ring_selection_index;
            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            let prefix = if is_selected { ">" } else { " " };
            content_lines.push(TextLine::from(Span::styled(
                format!("{} {}", prefix, ring.label()),
                style,
            )));

            if is_selected {
                content_lines.push(TextLine::from(Span::styled(
                    format!("   {}", ring.as_str()),
                    Style::default().fg(Color::Gray),
                )));
            }
        }
    } else {
        content_lines.push(TextLine::from(input_text));
    }

    // Add blip info if available
    if !blip_info.is_empty() {
        content_lines.push(TextLine::from(""));
        content_lines.extend(blip_info);

        // Add radar visualization if we have quadrant and ring
        if app.blip_data.quadrant.is_some() && app.blip_data.ring.is_some() {
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

    let content_inner = main_layout[1].inner(Margin::new(1, 1));
    let horizontal_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(content_inner);

    let left_split = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9),
            Constraint::Min(8),
        ])
        .split(horizontal_split[0]);

    f.render_widget(content_paragraph, left_split[0]);
    render_full_radar(app, f, left_split[1]);

    if app.input_state == InputState::WaitingForCommand {
        let right_split = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Percentage(97),
            ])
            .split(horizontal_split[1]);

        render_chart_tabs(app, f, right_split[0]);
        render_chart_panel(app, f, right_split[1]);
    } else if app.blip_data.quadrant.is_some() && app.blip_data.ring.is_some() {
        let radar_area = Rect {
            x: horizontal_split[1].x,
            y: horizontal_split[1].y,
            width: horizontal_split[1].width.min(20),
            height: horizontal_split[1].height.min(20),
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

    // Render status area
    let status_paragraph = Paragraph::new(status_text)
        .block(status_block)
        .wrap(Wrap { trim: true });
    f.render_widget(status_paragraph, main_layout[2]);

    // Render keyboard shortcut hints
    let shortcuts = TextLine::from(vec![
        Span::styled(
            "F1",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(": Help | ", Style::default().fg(Color::Gray)),
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
    ]);

    let shortcuts_paragraph = Paragraph::new(shortcuts).alignment(Alignment::Center);
    f.render_widget(shortcuts_paragraph, main_layout[3]);
}

fn render_blips_view(app: &App, f: &mut Frame<'_>) {
    let area = f.area();

    if app.blips.is_empty() {
        let block = Block::default()
            .title("Blips Table")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        let paragraph = Paragraph::new("No blips found.")
            .block(block)
            .alignment(Alignment::Center);
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

    let mut scroll_offset = 0;
    if total_rows > max_visible_rows {
        if app.selected_blip_index >= max_visible_rows + scroll_offset {
            scroll_offset = app.selected_blip_index.saturating_sub(max_visible_rows) + 1;
        } else if app.selected_blip_index < scroll_offset {
            scroll_offset = app.selected_blip_index;
        }
    }

    let visible_blips = app.blips.iter().skip(scroll_offset).take(max_visible_rows);

    let rows = visible_blips.enumerate().map(|(i, blip)| {
        let is_selected = i + scroll_offset == app.selected_blip_index;
        let style = if is_selected {
            Style::default()
                .bg(Color::Rgb(0, 0, 238))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(quadrant_color_from_option(blip.quadrant.as_deref()))
        };

        Row::new(vec![
            Cell::from(blip.id.to_string()),
            Cell::from(blip.name.clone()),
            Cell::from(blip.ring.clone().unwrap_or_default()),
            Cell::from(blip.quadrant.clone().unwrap_or_default()),
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
        .alignment(Alignment::Center);

    f.render_widget(help_paragraph, chunks[1]);
}

fn render_adrs_view(app: &App, f: &mut Frame<'_>) {
    let area = f.area();

    if app.adrs.is_empty() {
        let block = Block::default()
            .title("ADR Log")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        let paragraph = Paragraph::new("No ADRs found.")
            .block(block)
            .alignment(Alignment::Center);
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

    let mut scroll_offset = 0;
    if total_rows > max_visible_rows {
        if app.selected_adr_index >= max_visible_rows + scroll_offset {
            scroll_offset = app.selected_adr_index.saturating_sub(max_visible_rows) + 1;
        } else if app.selected_adr_index < scroll_offset {
            scroll_offset = app.selected_adr_index;
        }
    }

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
        || {
            format!(
                "ADR Log ({} of {})",
                app.selected_adr_index + 1,
                total_rows
            )
        },
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
        .alignment(Alignment::Center);

    f.render_widget(help_paragraph, chunks[1]);
}

fn render_blip_details(app: &App, f: &mut Frame<'_>) {
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
            blip.ring.clone().unwrap_or_else(|| "(none)".to_string())
        )),
        TextLine::from(format!(
            "Quadrant: {}",
            blip.quadrant
                .clone()
                .unwrap_or_else(|| "(none)".to_string())
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

fn render_full_radar(app: &App, f: &mut Frame<'_>, area: Rect) {
    let blips = &app.blips;
    if area.width < 8 || area.height < 6 {
        return;
    }

    let block = Block::default()
        .title("Tech Radar")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if blips.is_empty() {
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

    let points = blips
        .iter()
        .filter_map(|blip| {
            let quadrant = match blip.quadrant.as_deref() {
                Some("platforms") => 0,
                Some("languages") => 1,
                Some("tools") => 2,
                Some("techniques") => 3,
                _ => return None,
            };
            let ring = match blip.ring.as_deref() {
                Some("adopt") => 0,
                Some("trial") => 1,
                Some("assess") => 2,
                Some("hold") => 3,
                _ => return None,
            };

            let hash = blip.name.bytes().fold(0_u64, |acc, b| acc.wrapping_mul(31) + u64::from(b));
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

                for (blip, angle, radius) in &points {
                    let color = quadrant_color_from_option(blip.quadrant.as_deref());
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

fn render_blip_scatter(app: &App, f: &mut Frame<'_>, area: Rect) {
    let blips = &app.blips;
    if blips.is_empty() {
        let block = Block::default()
            .title("Blips Chart")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let paragraph = Paragraph::new("No blips available")
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
        return;
    }

    let mut platforms = Vec::new();
    let mut languages = Vec::new();
    let mut tools = Vec::new();
    let mut techniques = Vec::new();

    for blip in blips {
        let quadrant_label = blip.quadrant.as_deref();
        let quadrant = match quadrant_label {
            Some("platforms") => 1.0,
            Some("languages") => 2.0,
            Some("tools") => 3.0,
            Some("techniques") => 4.0,
            _ => continue,
        };
        let ring = match blip.ring.as_deref() {
            Some("hold") => 1.0,
            Some("assess") => 2.0,
            Some("trial") => 3.0,
            Some("adopt") => 4.0,
            _ => continue,
        };

        match quadrant_label {
            Some("platforms") => platforms.push((quadrant, ring)),
            Some("languages") => languages.push((quadrant, ring)),
            Some("tools") => tools.push((quadrant, ring)),
            Some("techniques") => techniques.push((quadrant, ring)),
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
    let y_labels = vec![Span::raw("Hold"), Span::raw("Assess"), Span::raw("Trial"), Span::raw("Adopt")];

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

fn render_chart_tabs(app: &App, f: &mut Frame<'_>, area: Rect) {
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

fn render_chart_panel(app: &App, f: &mut Frame<'_>, area: Rect) {
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

fn render_blip_barchart(app: &App, f: &mut Frame<'_>, area: Rect) {
    if app.blips.is_empty() {
        let block = Block::default()
            .title("Blip Types")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let paragraph = Paragraph::new("No blips available")
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
        return;
    }

    let mut counts = [0_u64; 4];
    for blip in &app.blips {
        let index = match blip.quadrant.as_deref() {
            Some("platforms") => 0,
            Some("languages") => 1,
            Some("tools") => 2,
            Some("techniques") => 3,
            _ => continue,
        };
        counts[index] += 1;
    }

    let labels = ["Platforms", "Languages", "Tools", "Techniques"];
    let bar_colors = [
        quadrant_color("platforms"),
        quadrant_color("languages"),
        quadrant_color("tools"),
        quadrant_color("techniques"),
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

fn render_ring_barchart(app: &App, f: &mut Frame<'_>, area: Rect) {
    if app.blips.is_empty() {
        let block = Block::default()
            .title("Ring Counts")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let paragraph = Paragraph::new("No blips available")
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
        return;
    }

    let mut counts = [0_u64; 4];
    for blip in &app.blips {
        let index = match blip.ring.as_deref() {
            Some("hold") => 0,
            Some("assess") => 1,
            Some("trial") => 2,
            Some("adopt") => 3,
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

fn render_ring_piechart(app: &App, f: &mut Frame<'_>, area: Rect) {
    if app.blips.is_empty() {
        let block = Block::default()
            .title("Ring Distribution")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let paragraph = Paragraph::new("No blips available")
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(paragraph, area);
        return;
    }

    let mut counts = [0_f64; 4];
    for blip in &app.blips {
        let index = match blip.ring.as_deref() {
            Some("hold") => 0,
            Some("assess") => 1,
            Some("trial") => 2,
            Some("adopt") => 3,
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

                // Draw rings
                for i in 1..=4 {
                    let ring_radius = max_radius * (f64::from(i) / 4.0);
                    ctx.draw(&Circle {
                        x: center_x,
                        y: center_y,
                        radius: ring_radius,
                        color: Color::Gray,
                    });
                }

                // Draw quadrant lines
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

                // Draw blip if we have data
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

                    // Calculate position based on quadrant and ring
                    let angle = std::f64::consts::PI / 4.0
                        + (f64::from(quadrant_idx) * std::f64::consts::PI / 2.0);
                    let radius = max_radius * ((f64::from(ring_idx) + 0.5) / 4.0);

                    // Add a small animation to the blip
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

pub fn render_help(f: &mut Frame<'_>, area: Rect) {
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
        TextLine::from(vec![
            Span::styled("Tech Radar ADR Generator", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        TextLine::from(""),
        TextLine::from("This tool helps you create Architectural Decision Records (ADRs) and Blips for your Tech Radar."),
        TextLine::from(""),
        TextLine::from(vec![
            Span::styled("Keyboard Shortcuts:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        TextLine::from(vec![
            Span::styled("  F1", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" - Toggle this help screen", Style::default()),
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
        TextLine::from(vec![
            Span::styled("Quadrants:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        TextLine::from("  1 - Platforms: Infrastructure, platforms, APIs and services"),
        TextLine::from("  2 - Languages: Programming languages and frameworks"),
        TextLine::from("  3 - Tools: Development, testing and operational tools"),
        TextLine::from("  4 - Techniques: Methods, practices and approaches"),
        TextLine::from(""),
        TextLine::from(vec![
            Span::styled("Rings:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        TextLine::from("  1 - Hold: Technologies we've used but are actively moving away from"),
        TextLine::from("  2 - Assess: Worth exploring with the goal of understanding how it affects us"),
        TextLine::from("  3 - Trial: Worth pursuing, important to understand how to build up this capability"),
        TextLine::from("  4 - Adopt: We feel strongly that the industry should be adopting these items"),
        TextLine::from(""),
        TextLine::from(vec![
            Span::styled("Press Esc to close this help screen", Style::default().fg(Color::Yellow)),
        ]),
    ];

    let help_paragraph = Paragraph::new(Text::from(help_text))
        .block(help_block)
        .wrap(Wrap { trim: true });

    f.render_widget(help_paragraph, area);
}
