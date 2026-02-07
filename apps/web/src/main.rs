mod animation;

use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use crate::animation::{advance_animation_counter, AnimationMode};
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

#[derive(serde::Deserialize, Clone)]
#[allow(dead_code)]
struct RadarAdr {
    id: i32,
    title: String,
    blip_name: String,
    status: String,
    timestamp: String,
}

struct AnimationState {
    counter: f64,
    last_tick: Option<f64>,
    mode: AnimationMode,
}

impl AnimationState {
    fn new() -> Self {
        Self {
            counter: 0.0,
            last_tick: None,
            mode: AnimationMode::Running,
        }
    }

    fn tick(&mut self, now_seconds: f64) {
        let (next_counter, next_tick) =
            advance_animation_counter(self.counter, self.last_tick, now_seconds, self.mode);
        self.counter = next_counter;
        self.last_tick = next_tick;
    }

    fn toggle_mode(&mut self) {
        self.mode = match self.mode {
            AnimationMode::Running => AnimationMode::Paused,
            AnimationMode::Paused => AnimationMode::Running,
        };
    }

    fn is_paused(&self) -> bool {
        self.mode == AnimationMode::Paused
    }
}

#[derive(Clone)]
struct SearchState {
    query: String,
    active: bool,
    column: SearchColumn,
    row: usize,
    detail_open: bool,
}

#[derive(Clone, Copy)]
enum TableDetailKind {
    RecentBlip,
    AllBlip,
    Adr,
}

#[derive(Clone, Copy)]
struct TableDetailState {
    active: bool,
    kind: TableDetailKind,
    row: usize,
}

struct DashboardState {
    tab_index: usize,
    row_offset: usize,
    table_selected_row: usize,
    table_detail: TableDetailState,
    animation_counter: f64,
    animation_paused: bool,
    search_state: SearchState,
}

impl SearchState {
    fn reset(&mut self) {
        self.query.clear();
        self.active = false;
        self.column = SearchColumn::Blips;
        self.row = 0;
        self.detail_open = false;
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SearchColumn {
    Blips,
    Adrs,
}

fn main() -> io::Result<()> {
    let data = Rc::new(RefCell::new(None::<RadarExport>));
    let tab_index = Rc::new(RefCell::new(0_usize));
    let row_offset = Rc::new(RefCell::new(0_usize));
    let table_selected_row = Rc::new(RefCell::new(0_usize));
    let table_view_rows = Rc::new(RefCell::new(1_usize));
    let animation_state = Rc::new(RefCell::new(AnimationState::new()));
    let search_state = Rc::new(RefCell::new(SearchState {
        query: String::new(),
        active: false,
        column: SearchColumn::Blips,
        row: 0,
        detail_open: false,
    }));
    let table_detail_state = Rc::new(RefCell::new(TableDetailState {
        active: false,
        kind: TableDetailKind::AllBlip,
        row: 0,
    }));

    spawn_local(fetch_radar(data.clone()));

    let backend = DomBackend::new()?;
    let terminal = Terminal::new(backend)?;

    terminal.on_key_event({
        let tab_index = tab_index.clone();
        let row_offset = row_offset.clone();
        let table_selected_row = table_selected_row.clone();
        let table_view_rows = table_view_rows.clone();
        let animation_state = animation_state.clone();
        let search_state = search_state.clone();
        let table_detail_state = table_detail_state.clone();
        let data = data.clone();
        move |event| {
            if search_state.borrow().active {
                handle_search_input(&search_state, event.code);
                return;
            }

            if table_detail_state.borrow().active {
                match event.code {
                    ratzilla::event::KeyCode::Esc | ratzilla::event::KeyCode::Enter => {
                        table_detail_state.borrow_mut().active = false;
                    }
                    _ => {}
                }
                return;
            }

            match event.code {
                ratzilla::event::KeyCode::Left => {
                    let mut index = tab_index.borrow_mut();
                    *index = if *index == 0 { 2 } else { *index - 1 };
                    *row_offset.borrow_mut() = 0;
                    *table_selected_row.borrow_mut() = 0;
                    table_detail_state.borrow_mut().active = false;
                }
                ratzilla::event::KeyCode::Right => {
                    let mut index = tab_index.borrow_mut();
                    *index = (*index + 1) % 3;
                    *row_offset.borrow_mut() = 0;
                    *table_selected_row.borrow_mut() = 0;
                    table_detail_state.borrow_mut().active = false;
                }
                ratzilla::event::KeyCode::Up => {
                    if let Some(export) = data.borrow().as_ref() {
                        let tab = *tab_index.borrow();
                        let total_rows = total_rows_for_tab(export, tab);
                        let max_rows = (*table_view_rows.borrow()).max(1);
                        let mut selected = table_selected_row.borrow_mut();
                        if total_rows > 0 && *selected > 0 {
                            *selected -= 1;
                        }
                        let mut offset = row_offset.borrow_mut();
                        let max_offset = total_rows.saturating_sub(max_rows);
                        *offset = (*selected).min(max_offset);
                    }
                }
                ratzilla::event::KeyCode::Down => {
                    if let Some(export) = data.borrow().as_ref() {
                        let tab = *tab_index.borrow();
                        let total_rows = total_rows_for_tab(export, tab);
                        let max_rows = (*table_view_rows.borrow()).max(1);
                        let mut selected = table_selected_row.borrow_mut();
                        if total_rows > 0 && *selected + 1 < total_rows {
                            *selected += 1;
                        }
                        let mut offset = row_offset.borrow_mut();
                        let max_offset = total_rows.saturating_sub(max_rows);
                        *offset = selected
                            .saturating_sub(max_rows.saturating_sub(1))
                            .min(max_offset);
                    }
                }
                ratzilla::event::KeyCode::PageUp => {
                    if let Some(export) = data.borrow().as_ref() {
                        let tab = *tab_index.borrow();
                        let total_rows = total_rows_for_tab(export, tab);
                        let max_rows = (*table_view_rows.borrow()).max(1);
                        let mut selected = table_selected_row.borrow_mut();
                        *selected = selected.saturating_sub(max_rows);
                        let mut offset = row_offset.borrow_mut();
                        let max_offset = total_rows.saturating_sub(max_rows);
                        *offset = (*selected).min(max_offset);
                    }
                }
                ratzilla::event::KeyCode::PageDown => {
                    if let Some(export) = data.borrow().as_ref() {
                        let tab = *tab_index.borrow();
                        let total_rows = total_rows_for_tab(export, tab);
                        let max_rows = (*table_view_rows.borrow()).max(1);
                        let mut selected = table_selected_row.borrow_mut();
                        if total_rows > 0 {
                            *selected = (*selected + max_rows).min(total_rows - 1);
                        }
                        let mut offset = row_offset.borrow_mut();
                        let max_offset = total_rows.saturating_sub(max_rows);
                        *offset = selected
                            .saturating_sub(max_rows.saturating_sub(1))
                            .min(max_offset);
                    }
                }
                ratzilla::event::KeyCode::Char(' ') => {
                    let mut state = animation_state.borrow_mut();
                    state.toggle_mode();
                }
                ratzilla::event::KeyCode::Char('s') => {
                    let mut state = search_state.borrow_mut();
                    state.active = true;
                    state.query.clear();
                    state.column = SearchColumn::Blips;
                    state.row = 0;
                    state.detail_open = false;
                }
                ratzilla::event::KeyCode::Enter => {
                    if let Some(export) = data.borrow().as_ref() {
                        let tab = *tab_index.borrow();
                        let selected = *table_selected_row.borrow();
                        let total_rows = total_rows_for_tab(export, tab);
                        if selected < total_rows {
                            let mut detail = table_detail_state.borrow_mut();
                            detail.active = true;
                            detail.row = selected;
                            detail.kind = match tab {
                                0 => TableDetailKind::RecentBlip,
                                1 => TableDetailKind::AllBlip,
                                _ => TableDetailKind::Adr,
                            };
                        }
                    }
                }
                ratzilla::event::KeyCode::Char('1') => {
                    *tab_index.borrow_mut() = 0;
                    *row_offset.borrow_mut() = 0;
                    *table_selected_row.borrow_mut() = 0;
                    table_detail_state.borrow_mut().active = false;
                }
                ratzilla::event::KeyCode::Char('2') => {
                    *tab_index.borrow_mut() = 1;
                    *row_offset.borrow_mut() = 0;
                    *table_selected_row.borrow_mut() = 0;
                    table_detail_state.borrow_mut().active = false;
                }
                ratzilla::event::KeyCode::Char('3') => {
                    *tab_index.borrow_mut() = 2;
                    *row_offset.borrow_mut() = 0;
                    *table_selected_row.borrow_mut() = 0;
                    table_detail_state.borrow_mut().active = false;
                }
                _ => {}
            }
        }
    });

    terminal.draw_web(move |f| {
        let now = web_sys::window()
            .and_then(|window| window.performance())
            .map(|performance| performance.now() / 1000.0)
            .unwrap_or_default();

        let mut animation = animation_state.borrow_mut();
        animation.tick(now);

        let area = f.area();
        let block = Block::default()
            .title("TECH RADAR")
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
            let state = DashboardState {
                tab_index: *tab_index.borrow(),
                row_offset: *row_offset.borrow(),
                table_selected_row: *table_selected_row.borrow(),
                table_detail: *table_detail_state.borrow(),
                animation_counter: animation.counter,
                animation_paused: animation.is_paused(),
                search_state: search_state.borrow().clone(),
            };

            let view_rows = render_dashboard(export, &state, f, inner);
            *table_view_rows.borrow_mut() = view_rows.max(1);

            if state.search_state.active {
                // Clear entire area behind popup to create solid modal overlay
                let overlay_bg = Style::default().fg(Color::White).bg(Color::Black);
                let overlay: Vec<TextLine> = (0..inner.height)
                    .map(|_| {
                        TextLine::from(Span::styled(" ".repeat(inner.width as usize), overlay_bg))
                    })
                    .collect();
                f.render_widget(Paragraph::new(Text::from(overlay)), inner);

                render_search_popup(export, &state.search_state, f, inner);
                if state.search_state.detail_open {
                    render_search_detail_popup(export, &state.search_state, f, inner);
                }
            }

            if state.table_detail.active {
                render_table_detail_popup(export, &state.table_detail, f, inner);
            }
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
    state: &DashboardState,
    f: &mut ratzilla::ratatui::Frame<'_>,
    area: Rect,
) -> usize {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(1),
            Constraint::Min(12),
            Constraint::Length(8),
        ])
        .split(area);

    render_header(export, &state.search_state, f, main_layout[0]);
    render_gap(f, main_layout[1]);

    let content = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(58), Constraint::Percentage(42)])
        .split(main_layout[2]);

    render_radar_panel(export, state.animation_counter, f, content[0]);

    let charts = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(content[1]);

    render_blip_type_chart(export, f, charts[0]);
    render_ring_chart(export, f, charts[1]);

    render_footer(
        export,
        state.tab_index,
        state.row_offset,
        state.table_selected_row,
        state.animation_paused,
        f,
        main_layout[3],
    )
}

fn render_header(
    export: &RadarExport,
    search_state: &SearchState,
    f: &mut ratzilla::ratatui::Frame<'_>,
    area: Rect,
) {
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
    f.render_widget(block, area);

    let inner = area.inner(Margin::new(1, 1));
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    let hint = Paragraph::new(Text::from(TextLine::from(Span::styled(
        "Press s to search",
        Style::default().fg(Color::Gray),
    ))));
    f.render_widget(hint, rows[1]);

    let paragraph = Paragraph::new(Text::from(line)).alignment(Alignment::Left);
    f.render_widget(paragraph, rows[0]);

    if search_state.active {
        let search_text = format!("Search: {}", search_state.query);
        let search_line = TextLine::from(Span::styled(
            search_text,
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
        f.render_widget(Paragraph::new(Text::from(search_line)), rows[1]);
    }
}

fn handle_search_input(state: &Rc<RefCell<SearchState>>, key: ratzilla::event::KeyCode) {
    let mut search_state = state.borrow_mut();
    match key {
        ratzilla::event::KeyCode::Esc => {
            if search_state.detail_open {
                search_state.detail_open = false;
            } else {
                search_state.reset();
            }
        }
        ratzilla::event::KeyCode::Enter => {
            search_state.detail_open = !search_state.detail_open;
        }
        ratzilla::event::KeyCode::Left => {
            if !search_state.detail_open {
                search_state.column = SearchColumn::Blips;
                search_state.row = 0;
            }
        }
        ratzilla::event::KeyCode::Right => {
            if !search_state.detail_open {
                search_state.column = SearchColumn::Adrs;
                search_state.row = 0;
            }
        }
        ratzilla::event::KeyCode::Up => {
            if !search_state.detail_open {
                search_state.row = search_state.row.saturating_sub(1);
            }
        }
        ratzilla::event::KeyCode::Down => {
            if !search_state.detail_open {
                search_state.row = search_state.row.saturating_add(1);
            }
        }
        ratzilla::event::KeyCode::Backspace => {
            if !search_state.detail_open {
                search_state.query.pop();
                search_state.row = 0;
            }
        }
        ratzilla::event::KeyCode::Char(ch) => {
            if !search_state.detail_open {
                search_state.query.push(ch);
                search_state.row = 0;
            }
        }
        _ => {}
    }
}

fn render_search_popup(
    export: &RadarExport,
    search_state: &SearchState,
    f: &mut ratzilla::ratatui::Frame<'_>,
    area: Rect,
) {
    let popup_area = Rect {
        x: area
            .x
            .saturating_add(area.width / 2)
            .saturating_sub(area.width / 3),
        y: area
            .y
            .saturating_add(area.height / 2)
            .saturating_sub(area.height / 3),
        width: area.width.saturating_mul(2) / 3,
        height: area.height.saturating_mul(2) / 3,
    };

    let bg = Style::default().fg(Color::White).bg(Color::Black);

    // Fill popup area with spaces to clear radar canvas characters underneath
    let clear_lines: Vec<TextLine> = (0..popup_area.height)
        .map(|_| TextLine::from(Span::styled(" ".repeat(popup_area.width as usize), bg)))
        .collect();
    f.render_widget(Paragraph::new(Text::from(clear_lines)), popup_area);

    let block = Block::default()
        .title("Search Results")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .style(bg);
    f.render_widget(block, popup_area);

    let inner = popup_area.inner(Margin::new(1, 1));
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Min(5),
            Constraint::Length(2),
        ])
        .split(inner);

    let query_line = TextLine::from(vec![
        Span::styled("Query: ", Style::default().fg(Color::Gray)),
        Span::styled(
            search_state.query.as_str(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    f.render_widget(Paragraph::new(Text::from(query_line)).style(bg), layout[0]);

    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(layout[1]);

    let blip_matches = filter_blips(export, search_state);
    let adr_matches = filter_adrs(export, search_state);

    let max_rows = layout[1].height.saturating_sub(1) as usize;
    let max_rows = max_rows.max(1);
    let max_blip_row = blip_matches.len().min(max_rows).saturating_sub(1);
    let max_adr_row = adr_matches.len().min(max_rows).saturating_sub(1);

    let blip_row = if blip_matches.is_empty() {
        0
    } else {
        search_state.row.min(max_blip_row)
    };
    let adr_row = if adr_matches.is_empty() {
        0
    } else {
        search_state.row.min(max_adr_row)
    };

    render_search_blip_column(
        "Blips",
        &blip_matches,
        blip_row,
        search_state.column == SearchColumn::Blips,
        f,
        columns[0],
    );

    render_search_adr_column(
        "ADRs",
        &adr_matches,
        adr_row,
        search_state.column == SearchColumn::Adrs,
        f,
        columns[1],
    );

    let hint = TextLine::from(vec![
        Span::styled("↑/↓", Style::default().fg(Color::Yellow)),
        Span::raw(": Navigate   "),
        Span::styled("←/→", Style::default().fg(Color::Yellow)),
        Span::raw(": Column   "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(": Close"),
    ]);
    f.render_widget(
        Paragraph::new(Text::from(hint))
            .alignment(Alignment::Center)
            .style(bg),
        layout[2],
    );
}

fn filter_blips(export: &RadarExport, search_state: &SearchState) -> Vec<RadarBlip> {
    let query = search_state.query.trim().to_lowercase();
    if query.is_empty() {
        return export.blips.clone();
    }

    export
        .blips
        .iter()
        .filter(|blip| {
            let mut haystack = blip.name.to_lowercase();
            if let Some(tag) = &blip.tag {
                haystack.push(' ');
                haystack.push_str(&tag.to_lowercase());
            }
            if let Some(description) = &blip.description {
                haystack.push(' ');
                haystack.push_str(&description.to_lowercase());
            }
            if let Some(ring) = &blip.ring {
                haystack.push(' ');
                haystack.push_str(&ring.to_lowercase());
            }
            if let Some(quadrant) = &blip.quadrant {
                haystack.push(' ');
                haystack.push_str(&quadrant.to_lowercase());
            }
            haystack.contains(&query)
        })
        .cloned()
        .collect()
}

fn filter_adrs(export: &RadarExport, search_state: &SearchState) -> Vec<RadarAdr> {
    let query = search_state.query.trim().to_lowercase();
    if query.is_empty() {
        return export.adrs.clone();
    }

    export
        .adrs
        .iter()
        .filter(|adr| {
            let mut haystack = adr.title.to_lowercase();
            haystack.push(' ');
            haystack.push_str(&adr.blip_name.to_lowercase());
            haystack.push(' ');
            haystack.push_str(&adr.status.to_lowercase());
            haystack.push(' ');
            haystack.push_str(&adr.timestamp.to_lowercase());
            haystack.contains(&query)
        })
        .cloned()
        .collect()
}

fn render_search_blip_column(
    title: &str,
    rows: &[RadarBlip],
    selected_row: usize,
    is_active: bool,
    f: &mut ratzilla::ratatui::Frame<'_>,
    area: Rect,
) {
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray))
        .style(Style::default().bg(Color::Black));
    f.render_widget(block, area);

    let inner = area.inner(Margin::new(1, 1));
    if inner.height == 0 {
        return;
    }

    let max_rows = inner.height as usize;
    let rows = rows.iter().take(max_rows).enumerate().map(|(index, item)| {
        let is_selected = is_active && selected_row == index;
        let style = if is_selected {
            Style::default().fg(Color::Black).bg(Color::Yellow)
        } else {
            Style::default().fg(Color::White).bg(Color::Black)
        };
        TextLine::from(Span::styled(item.name.clone(), style))
    });

    let bg = Style::default().fg(Color::White).bg(Color::Black);
    f.render_widget(Paragraph::new(Text::from_iter(rows)).style(bg), inner);
}

fn render_search_adr_column(
    title: &str,
    rows: &[RadarAdr],
    selected_row: usize,
    is_active: bool,
    f: &mut ratzilla::ratatui::Frame<'_>,
    area: Rect,
) {
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray))
        .style(Style::default().bg(Color::Black));
    f.render_widget(block, area);

    let inner = area.inner(Margin::new(1, 1));
    if inner.height == 0 {
        return;
    }

    let max_rows = inner.height as usize;
    let rows = rows.iter().take(max_rows).enumerate().map(|(index, item)| {
        let is_selected = is_active && selected_row == index;
        let style = if is_selected {
            Style::default().fg(Color::Black).bg(Color::Yellow)
        } else {
            Style::default().fg(Color::White).bg(Color::Black)
        };
        TextLine::from(Span::styled(item.title.clone(), style))
    });

    let bg = Style::default().fg(Color::White).bg(Color::Black);
    f.render_widget(Paragraph::new(Text::from_iter(rows)).style(bg), inner);
}

fn render_search_detail_popup(
    export: &RadarExport,
    search_state: &SearchState,
    f: &mut ratzilla::ratatui::Frame<'_>,
    area: Rect,
) {
    let popup_area = Rect {
        x: area
            .x
            .saturating_add(area.width / 2)
            .saturating_sub(area.width / 4),
        y: area
            .y
            .saturating_add(area.height / 2)
            .saturating_sub(area.height / 4),
        width: area.width / 2,
        height: area.height / 2,
    };

    let bg = Style::default().fg(Color::White).bg(Color::Black);

    // Fill popup area with spaces to clear canvas characters underneath
    let clear_lines: Vec<TextLine> = (0..popup_area.height)
        .map(|_| TextLine::from(Span::styled(" ".repeat(popup_area.width as usize), bg)))
        .collect();
    f.render_widget(Paragraph::new(Text::from(clear_lines)), popup_area);

    let block = Block::default()
        .title("Details")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(bg);
    f.render_widget(block, popup_area);

    let inner = popup_area.inner(Margin::new(1, 1));

    let lines = match search_state.column {
        SearchColumn::Blips => build_blip_detail_lines(export, search_state),
        SearchColumn::Adrs => build_adr_detail_lines(export, search_state),
    };

    let content = Paragraph::new(Text::from(lines))
        .style(Style::default().bg(Color::Black))
        .wrap(Wrap { trim: true });
    f.render_widget(content, inner);
}

fn build_blip_detail_lines(
    export: &RadarExport,
    search_state: &SearchState,
) -> Vec<TextLine<'static>> {
    let blip_matches = filter_blips(export, search_state);
    let Some(blip) = blip_matches.get(search_state.row) else {
        return vec![TextLine::from("No blip selected")];
    };

    blip_detail_lines(blip)
}

fn build_adr_detail_lines(
    export: &RadarExport,
    search_state: &SearchState,
) -> Vec<TextLine<'static>> {
    let adr_matches = filter_adrs(export, search_state);
    let Some(adr) = adr_matches.get(search_state.row) else {
        return vec![TextLine::from("No ADR selected")];
    };

    adr_detail_lines(adr)
}

fn blip_detail_lines(blip: &RadarBlip) -> Vec<TextLine<'static>> {
    vec![
        TextLine::from(Span::styled(
            blip.name.clone(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        TextLine::from(""),
        TextLine::from(format!(
            "Quadrant: {}",
            blip.quadrant
                .clone()
                .unwrap_or_else(|| "(none)".to_string())
        )),
        TextLine::from(format!(
            "Ring: {}",
            blip.ring.clone().unwrap_or_else(|| "(none)".to_string())
        )),
        TextLine::from(format!(
            "Tag: {}",
            blip.tag.clone().unwrap_or_else(|| "(none)".to_string())
        )),
        TextLine::from(format!(
            "Has ADR: {}",
            if blip.has_adr { "Yes" } else { "No" }
        )),
        TextLine::from(format!("Created: {}", blip.created)),
    ]
}

fn adr_detail_lines(adr: &RadarAdr) -> Vec<TextLine<'static>> {
    vec![
        TextLine::from(Span::styled(
            adr.title.clone(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )),
        TextLine::from(""),
        TextLine::from(format!("Blip: {}", adr.blip_name)),
        TextLine::from(format!("Status: {}", adr.status)),
        TextLine::from(format!("Date: {}", adr.timestamp)),
    ]
}

fn render_table_detail_popup(
    export: &RadarExport,
    detail: &TableDetailState,
    f: &mut ratzilla::ratatui::Frame<'_>,
    area: Rect,
) {
    let popup_area = Rect {
        x: area.x + (area.width / 2).saturating_sub(area.width / 4),
        y: area.y + (area.height / 2).saturating_sub(area.height / 4),
        width: area.width / 2,
        height: area.height / 2,
    };

    let bg = Style::default().fg(Color::White).bg(Color::Black);
    let clear_lines: Vec<TextLine> = (0..popup_area.height)
        .map(|_| TextLine::from(Span::styled(" ".repeat(popup_area.width as usize), bg)))
        .collect();
    f.render_widget(Paragraph::new(Text::from(clear_lines)), popup_area);

    let block = Block::default()
        .title("Details")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(bg);
    f.render_widget(block, popup_area);

    let inner = popup_area.inner(Margin::new(1, 1));
    let lines = match detail.kind {
        TableDetailKind::RecentBlip => {
            let mut blips = export.blips.clone();
            blips.sort_by(|a, b| b.created.cmp(&a.created));
            blips
                .get(detail.row)
                .map(blip_detail_lines)
                .unwrap_or_else(|| vec![TextLine::from("No blip selected")])
        }
        TableDetailKind::AllBlip => export
            .blips
            .get(detail.row)
            .map(blip_detail_lines)
            .unwrap_or_else(|| vec![TextLine::from("No blip selected")]),
        TableDetailKind::Adr => export
            .adrs
            .get(detail.row)
            .map(adr_detail_lines)
            .unwrap_or_else(|| vec![TextLine::from("No ADR selected")]),
    };

    let content = Paragraph::new(Text::from(lines))
        .style(Style::default().bg(Color::Black))
        .wrap(Wrap { trim: true });
    f.render_widget(content, inner);
}

fn render_gap(f: &mut ratzilla::ratatui::Frame<'_>, area: Rect) {
    let paragraph = Paragraph::new("")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(paragraph, area);
}

fn total_rows_for_tab(export: &RadarExport, tab_index: usize) -> usize {
    match tab_index {
        0 | 1 => export.blips.len(),
        2 => export.adrs.len(),
        _ => 0,
    }
}

fn render_footer(
    export: &RadarExport,
    tab_index: usize,
    row_offset: usize,
    selected_row: usize,
    animation_paused: bool,
    f: &mut ratzilla::ratatui::Frame<'_>,
    area: Rect,
) -> usize {
    let total_blips = export.blips.len();
    let total_adrs = export.adrs.len();

    let tabs = ["Recent blips", "All blips", "All ADRs"];
    let tab_titles = tabs
        .iter()
        .map(|title| TextLine::from(*title))
        .collect::<Vec<_>>();

    let mut info_items = vec![
        Span::styled("Tables", Style::default().fg(Color::Gray)),
        Span::raw("  "),
        Span::raw(format!("{total_blips} blips • {total_adrs} ADRs")),
        Span::raw("  "),
        Span::styled("Tab/1-3", Style::default().fg(Color::Gray)),
        Span::raw("  "),
        Span::styled("Arrows", Style::default().fg(Color::Gray)),
        Span::raw("  "),
        Span::styled("Space", Style::default().fg(Color::Gray)),
        Span::raw(": pause"),
    ];
    info_items.push(Span::raw("  "));
    if animation_paused {
        info_items.push(Span::styled(
            "Animation paused",
            Style::default().fg(Color::Yellow),
        ));
    } else {
        info_items.push(Span::styled(
            "Animation active",
            Style::default().fg(Color::Gray),
        ));
    }
    let info = TextLine::from(info_items);

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

    let desired_rows = match tab_index {
        0 => 8,
        _ => 18,
    };
    // Table includes a header row plus a spacer row before data rows.
    // Keep scrolling math based on visible data rows only.
    let view_rows = desired_rows
        .min(table_area.height.saturating_sub(2) as usize)
        .max(1);

    match tab_index {
        0 => render_recent_blips(export, row_offset, selected_row, view_rows, f, table_area),
        1 => render_all_blips(export, row_offset, selected_row, view_rows, f, table_area),
        2 => render_all_adrs(export, row_offset, selected_row, view_rows, f, table_area),
        _ => {}
    }

    view_rows
}

fn render_radar_panel(
    export: &RadarExport,
    animation_counter: f64,
    f: &mut ratzilla::ratatui::Frame<'_>,
    area: Rect,
) {
    let block = Block::default()
        .title("TECH RADAR")
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

            let pulse = (animation_counter * 0.6 + jitter).sin().mul_add(0.25, 0.75);
            let blip_radius = 0.03 + (pulse * 0.015);

            Some((
                angle,
                radius,
                blip_radius,
                quadrant_color(blip.quadrant.as_deref()),
            ))
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

                let pulse = (animation_counter * 0.6).sin().mul_add(0.5, 0.5);
                let pulse_radius = max_radius * (0.45 + pulse * 0.5);
                ctx.draw(&ratzilla::ratatui::widgets::canvas::Circle {
                    x: center_x,
                    y: center_y,
                    radius: pulse_radius,
                    color: Color::LightCyan,
                });

                ctx.draw(&ratzilla::ratatui::widgets::canvas::Line {
                    x1: center_x,
                    y1: center_y - max_radius,
                    x2: center_x,
                    y2: center_y + max_radius,
                    color: Color::DarkGray,
                });
                ctx.draw(&ratzilla::ratatui::widgets::canvas::Line {
                    x1: center_x - max_radius,
                    y1: center_y,
                    x2: center_x + max_radius,
                    y2: center_y,
                    color: Color::DarkGray,
                });

                let sweep_angle = animation_counter * 1.4;
                let sweep_x = sweep_angle.cos().mul_add(max_radius, center_x);
                let sweep_y = sweep_angle.sin().mul_add(max_radius, center_y);
                ctx.draw(&ratzilla::ratatui::widgets::canvas::Line {
                    x1: center_x,
                    y1: center_y,
                    x2: sweep_x,
                    y2: sweep_y,
                    color: Color::LightCyan,
                });

                let ghost_angle = sweep_angle + (std::f64::consts::PI / 20.0);
                let ghost_x = ghost_angle.cos().mul_add(max_radius * 0.92, center_x);
                let ghost_y = ghost_angle.sin().mul_add(max_radius * 0.92, center_y);
                ctx.draw(&ratzilla::ratatui::widgets::canvas::Line {
                    x1: center_x,
                    y1: center_y,
                    x2: ghost_x,
                    y2: ghost_y,
                    color: Color::DarkGray,
                });

                for (angle, radius, blip_radius, color) in &points {
                    let x = angle.cos().mul_add(max_radius * radius, center_x);
                    let y = angle.sin().mul_add(max_radius * radius, center_y);

                    ctx.draw(&ratzilla::ratatui::widgets::canvas::Circle {
                        x,
                        y,
                        radius: max_radius * blip_radius,
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
        .border_style(Style::default().fg(Color::Gray))
        .style(Style::default().bg(Color::Black));

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
    selected_row: usize,
    view_rows: usize,
    f: &mut ratzilla::ratatui::Frame<'_>,
    area: Rect,
) {
    let mut blips = export.blips.clone();
    blips.sort_by(|a, b| b.created.cmp(&a.created));

    render_blip_rows(&blips, row_offset, selected_row, f, area, view_rows);
}

fn render_all_blips(
    export: &RadarExport,
    row_offset: usize,
    selected_row: usize,
    view_rows: usize,
    f: &mut ratzilla::ratatui::Frame<'_>,
    area: Rect,
) {
    render_blip_rows(&export.blips, row_offset, selected_row, f, area, view_rows);
}

fn render_blip_rows(
    blips: &[RadarBlip],
    row_offset: usize,
    selected_row: usize,
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
    .chain(
        blips
            .iter()
            .skip(row_offset)
            .take(max_rows)
            .enumerate()
            .map(|(index, blip)| {
                let data_index = row_offset + index;
                let is_selected = data_index == selected_row;
                let style = if is_selected {
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Rgb(0, 0, 238))
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
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
                .style(style)
            }),
    );

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
    selected_row: usize,
    view_rows: usize,
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
    .chain(
        export
            .adrs
            .iter()
            .skip(row_offset)
            .take(view_rows)
            .enumerate()
            .map(|(index, adr)| {
                let data_index = row_offset + index;
                let is_selected = data_index == selected_row;
                let style = if is_selected {
                    Style::default()
                        .fg(Color::White)
                        .bg(Color::Rgb(0, 0, 238))
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                Row::new(vec![
                    Cell::from(adr.title.clone()),
                    Cell::from(adr.blip_name.clone()),
                    Cell::from(adr.status.clone()),
                    Cell::from(adr.timestamp.clone()),
                ])
                .style(style)
            }),
    );

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
        .viewport_content_length(view_rows.min(area.height.saturating_sub(1) as usize));
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
