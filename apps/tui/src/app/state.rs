use crate::app::actions::AppActions;
use crate::db::models::{AdrMetadataParams, BlipMetadataParams, BlipRecord};
use crate::db::queries::{AdrUpdateParams, BlipUpdateParams};
use crate::{Quadrant, Ring};
use color_eyre::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use std::{
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant},
};

use crate::app::input::screens::edit_adr::{AdrEditField, AdrEditState};
use crate::ui::screens::main::{CompletionBlip, CompletionStats};
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui_core::style::Color as CoreColor;
use tachyonfx::fx;
use tachyonfx::CellFilter;
use tachyonfx::Effect;
use tachyonfx::Interpolation;

#[derive(Debug)]
pub struct BlipData {
    pub name: String,
    pub quadrant: Option<Quadrant>,
    pub ring: Option<Ring>,
}

impl BlipData {
    pub const fn new() -> Self {
        Self {
            name: String::new(),
            quadrant: None,
            ring: None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum InputState {
    WaitingForCommand,
    EnteringTechnology,
    ChoosingAdrStatus,
    ChoosingQuadrant,
    ChoosingRing,
    GeneratingFile,
    Completed,
}

#[derive(Debug, PartialEq, Eq)]
pub enum AppScreen {
    Main,
    ViewBlips,
    ViewAdrs,
    AdrActions,
    AdrDetails,
    EditAdr,
    BlipActions,
    BlipDetails,
    EditBlip,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InputMode {
    Adr,
    Blip,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AdrStatus {
    Proposed,
    Accepted,
    Rejected,
    Deprecated,
    Superseded,
}

impl AdrStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Deprecated => "deprecated",
            Self::Superseded => "superseded",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Proposed => "Proposed",
            Self::Accepted => "Accepted",
            Self::Rejected => "Rejected",
            Self::Deprecated => "Deprecated",
            Self::Superseded => "Superseded",
        }
    }

    pub const fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Self::Proposed),
            1 => Some(Self::Accepted),
            2 => Some(Self::Rejected),
            3 => Some(Self::Deprecated),
            4 => Some(Self::Superseded),
            _ => None,
        }
    }

    pub const fn all() -> [Self; 5] {
        [
            Self::Proposed,
            Self::Accepted,
            Self::Rejected,
            Self::Deprecated,
            Self::Superseded,
        ]
    }

    pub fn next(self) -> Self {
        let statuses = Self::all();
        let index = statuses.iter().position(|item| *item == self).unwrap_or(0);
        statuses[(index + 1) % statuses.len()]
    }

    pub fn prev(self) -> Self {
        let statuses = Self::all();
        let index = statuses.iter().position(|item| *item == self).unwrap_or(0);
        statuses[(index + statuses.len() - 1) % statuses.len()]
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "proposed" => Some(Self::Proposed),
            "accepted" => Some(Self::Accepted),
            "rejected" => Some(Self::Rejected),
            "deprecated" => Some(Self::Deprecated),
            "superseded" => Some(Self::Superseded),
            _ => None,
        }
    }
}

/// Represents which field is currently being edited in the EditBlip screen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditField {
    Name,
    Ring,
    Quadrant,
    Tag,
    Description,
    Save,
}

/// Holds the temporary state of a blip being edited
#[derive(Debug, Clone)]
pub struct EditBlipState {
    pub id: i32,
    pub adr_id: Option<i32>,
    pub field: EditField,
    pub name: String,
    pub ring: String,
    pub quadrant: String,
    pub tag: String,
    pub description: String,
    pub editing: bool,
    pub ring_index: usize,
    pub quadrant_index: usize,
}

impl EditBlipState {
    pub fn from_blip(blip: &BlipRecord) -> Self {
        let ring_index = match blip.ring {
            Some(crate::Ring::Assess) => 1,
            Some(crate::Ring::Trial) => 2,
            Some(crate::Ring::Adopt) => 3,
            _ => 0,
        };

        let quadrant_index = match blip.quadrant {
            Some(crate::Quadrant::Languages) => 1,
            Some(crate::Quadrant::Tools) => 2,
            Some(crate::Quadrant::Techniques) => 3,
            _ => 0,
        };

        Self {
            id: blip.id,
            adr_id: blip.adr_id,
            field: EditField::Name,
            name: blip.name.clone(),
            ring: Self::ring_options()[ring_index].to_string(),
            quadrant: Self::quadrant_options()[quadrant_index].to_string(),
            tag: blip.tag.clone().unwrap_or_default(),
            description: blip.description.clone().unwrap_or_default(),
            editing: false,
            ring_index,
            quadrant_index,
        }
    }

    pub const fn ring_options() -> &'static [&'static str] {
        &["hold", "assess", "trial", "adopt"]
    }

    pub const fn quadrant_options() -> &'static [&'static str] {
        &["platforms", "languages", "tools", "techniques"]
    }

    pub fn next_ring(&mut self) {
        self.ring_index = (self.ring_index + 1) % Self::ring_options().len();
        self.ring = Self::ring_options()[self.ring_index].to_string();
    }

    pub fn prev_ring(&mut self) {
        self.ring_index =
            (self.ring_index + Self::ring_options().len() - 1) % Self::ring_options().len();
        self.ring = Self::ring_options()[self.ring_index].to_string();
    }

    pub fn next_quadrant(&mut self) {
        self.quadrant_index = (self.quadrant_index + 1) % Self::quadrant_options().len();
        self.quadrant = Self::quadrant_options()[self.quadrant_index].to_string();
    }

    pub fn prev_quadrant(&mut self) {
        self.quadrant_index = (self.quadrant_index + Self::quadrant_options().len() - 1)
            % Self::quadrant_options().len();
        self.quadrant = Self::quadrant_options()[self.quadrant_index].to_string();
    }
}

pub struct App {
    pub running: bool,
    pub input_state: InputState,
    pub current_input: String,
    pub blip_data: BlipData,
    pub input_mode: Option<InputMode>,
    pub adr_status: Option<AdrStatus>,
    pub status_message: String,
    pub save_notice_until: Option<Instant>,
    pub actions: AppActions,
    pub animation_counter: f64,
    pub last_frame: Instant,
    pub last_tick: Duration,
    pub animation_paused: bool,
    pub show_help: bool,
    pub completion_stats: Option<CompletionStats>,
    pub completion_fx: Mutex<Option<Effect>>,
    pub ring_pie_fx: Mutex<Option<Effect>>,
    pub ring_pie_area: Mutex<Option<Rect>>,
    pub settings_selection_index: usize,
    pub settings_editing: bool,
    pub settings_input: String,
    pub settings_blip_dir: String,
    pub settings_adr_dir: String,
    pub settings_db_name: String,
    pub screen: AppScreen,
    pub blips: Vec<crate::db::models::BlipRecord>,
    pub selected_blip_index: usize,
    pub edit_blip_state: Option<EditBlipState>,
    pub edit_adr_state: Option<AdrEditState>,
    pub blip_action_index: usize,
    pub adr_action_index: usize,
    pub selected_adr_index: usize,
    pub adrs: Vec<crate::db::models::AdrRecord>,
    pub adr_filter_name: Option<String>,
    pub quadrant_selection_index: usize,
    pub ring_selection_index: usize,
    pub adr_status_selection_index: usize,
    pub input_mode_selection_index: usize,
    pub chart_tab_index: usize,
    pub last_checked_blip_name: Option<String>,
    pub last_blip_name_exists: bool,
    pub search_query: String,
    pub search_active: bool,
    pub search_result_index: usize,
    pub search_throbber_state: throbber_widgets_tui::ThrobberState,
    pub filtered_blip_indices: Vec<usize>,
    pub filtered_adr_indices: Vec<usize>,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            input_state: InputState::WaitingForCommand,
            current_input: String::new(),
            blip_data: BlipData::new(),
            input_mode: None,
            adr_status: None,
            status_message: String::new(),
            save_notice_until: None,
            actions: AppActions::new(),
            animation_counter: 0.0,
            last_frame: Instant::now(),
            last_tick: Duration::from_millis(0),
            animation_paused: false,
            show_help: false,
            completion_stats: None,
            completion_fx: Mutex::new(None),
            ring_pie_fx: Mutex::new(None),
            ring_pie_area: Mutex::new(None),
            settings_selection_index: 0,
            settings_editing: false,
            settings_input: String::new(),
            settings_blip_dir: String::new(),
            settings_adr_dir: String::new(),
            settings_db_name: String::new(),
            screen: AppScreen::Main,

            blips: Vec::new(),
            selected_blip_index: 0,
            edit_blip_state: None,
            edit_adr_state: None,
            blip_action_index: 0,
            adr_action_index: 0,
            selected_adr_index: 0,
            adrs: Vec::new(),
            adr_filter_name: None,
            quadrant_selection_index: 0,
            ring_selection_index: 0,
            adr_status_selection_index: 0,
            input_mode_selection_index: 0,
            chart_tab_index: 0,
            last_checked_blip_name: None,

            last_blip_name_exists: false,
            search_query: String::new(),
            search_active: false,
            search_result_index: 0,
            search_throbber_state: throbber_widgets_tui::ThrobberState::default(),
            filtered_blip_indices: Vec::new(),
            filtered_adr_indices: Vec::new(),
        }
    }

    pub async fn initialize_db(&mut self) -> Result<()> {
        self.actions.initialize().await?;
        self.load_settings_from_env();
        self.load_settings_from_db().await;
        self.fetch_blips().await?;
        Ok(())
    }

    pub async fn load_settings_from_db(&mut self) {
        let Ok(settings) = self.actions.get_settings().await else {
            return;
        };

        for (key, value) in settings {
            match key.as_str() {
                "ADR_DIR" => self.settings_adr_dir = value,
                "BLIP_DIR" => self.settings_blip_dir = value,
                "DATABASE_NAME" => self.settings_db_name = value,
                _ => {}
            }
        }

        self.apply_settings_runtime();
    }

    pub fn apply_settings_runtime(&mut self) {
        self.actions.adrs_dir = PathBuf::from(&self.settings_adr_dir);
        self.actions.blips_dir = PathBuf::from(&self.settings_blip_dir);
    }

    pub async fn ensure_adrs_loaded(&mut self) -> Result<()> {
        if self.adrs.is_empty() {
            self.fetch_adrs_for_blip("").await?;
        }
        Ok(())
    }

    pub async fn persist_settings(&self) -> Result<()> {
        self.actions
            .set_setting("ADR_DIR", &self.settings_adr_dir)
            .await?;
        self.actions
            .set_setting("BLIP_DIR", &self.settings_blip_dir)
            .await?;
        self.actions
            .set_setting("DATABASE_NAME", &self.settings_db_name)
            .await?;
        Ok(())
    }

    pub fn load_settings_from_env(&mut self) {
        self.settings_adr_dir = crate::config::get_adrs_dir().to_string_lossy().to_string();
        self.settings_blip_dir = crate::config::get_blips_dir().to_string_lossy().to_string();
        self.settings_db_name =
            std::env::var("DATABASE_NAME").unwrap_or_else(|_| "adrs.db".to_string());
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame);
        self.last_frame = now;
        self.last_tick = delta;

        if let Some(until) = self.save_notice_until {
            if Instant::now() >= until {
                self.save_notice_until = None;
                self.status_message.clear();
            }
        }

        if self.animation_paused {
            return;
        }

        // Update animation counter (cycles between 0 and 2*PI)
        self.animation_counter += delta.as_secs_f64() * 2.0;
        if self.animation_counter > 2.0 * std::f64::consts::PI {
            self.animation_counter -= 2.0 * std::f64::consts::PI;
        }

        if self.search_active {
            self.search_throbber_state.calc_next();
        }
    }

    pub fn process_current_input(&mut self) {
        match self.input_state {
            InputState::EnteringTechnology => {
                if !self.current_input.is_empty() {
                    self.blip_data.name = self.current_input.clone();
                }
            }
            InputState::ChoosingQuadrant => {
                if let Some(quadrant) = Quadrant::from_index(self.quadrant_selection_index) {
                    self.blip_data.quadrant = Some(quadrant);
                } else {
                    self.status_message = "Invalid quadrant selection.".to_string();
                    return;
                }
            }
            InputState::ChoosingRing => {
                if let Some(ring) = Ring::from_index(self.ring_selection_index) {
                    self.blip_data.ring = Some(ring);
                } else {
                    self.status_message = "Invalid ring selection.".to_string();
                    return;
                }
            }
            InputState::ChoosingAdrStatus => {
                if let Some(status) = AdrStatus::from_index(self.adr_status_selection_index) {
                    self.adr_status = Some(status);
                } else {
                    self.status_message = "Invalid status selection.".to_string();
                    return;
                }
            }
            _ => {}
        }
        self.status_message.clear();
    }

    pub fn advance_state(&mut self) {
        self.input_state = match self.input_state {
            InputState::WaitingForCommand => {
                self.input_mode = match self.input_mode_selection_index {
                    0 => Some(InputMode::Adr),
                    1 => Some(InputMode::Blip),
                    _ => None,
                };
                InputState::EnteringTechnology
            }
            InputState::EnteringTechnology => {
                if self.input_mode == Some(InputMode::Adr) {
                    self.adr_status = Some(AdrStatus::Proposed);
                    self.adr_status_selection_index = 0;
                    InputState::ChoosingAdrStatus
                } else {
                    self.quadrant_selection_index = 0;
                    InputState::ChoosingQuadrant
                }
            }
            InputState::ChoosingAdrStatus => {
                if let Some(status) = AdrStatus::from_index(self.adr_status_selection_index) {
                    self.adr_status = Some(status);
                    InputState::GeneratingFile
                } else {
                    self.status_message = "Invalid status selection.".to_string();
                    InputState::ChoosingAdrStatus
                }
            }
            InputState::ChoosingQuadrant => {
                self.ring_selection_index = 0;
                InputState::ChoosingRing
            }
            InputState::ChoosingRing => {
                if let Some(ring) = Ring::from_index(self.ring_selection_index) {
                    self.blip_data.ring = Some(ring);
                    InputState::GeneratingFile
                } else {
                    self.status_message = "Invalid ring selection.".to_string();
                    InputState::ChoosingRing
                }
            }
            InputState::GeneratingFile => InputState::GeneratingFile, // Stay in this state until file is generated
            InputState::Completed => {
                self.completion_stats = None;
                if let Ok(mut effect) = self.completion_fx.lock() {
                    *effect = None;
                }
                InputState::WaitingForCommand
            }
        };
        self.current_input.clear();
    }

    pub async fn refresh_completion_stats(&mut self) {
        let total_blips = self.actions.count_blips().await.unwrap_or(0);
        let total_adrs = self.actions.count_adrs().await.unwrap_or(0);
        let recent = self.actions.recent_blips(5).await.unwrap_or_default();

        let coverage = if total_blips > 0 {
            #[allow(clippy::cast_precision_loss)]
            Some((total_adrs as f64 / total_blips as f64) * 100.0)
        } else {
            None
        };

        let recent = recent
            .into_iter()
            .map(|blip| {
                let ring = blip
                    .ring
                    .map_or_else(|| "(none)".to_string(), |ring| ring.as_str().to_string());
                let quadrant = blip.quadrant.map_or_else(
                    || "(none)".to_string(),
                    |quadrant| quadrant.as_str().to_string(),
                );
                CompletionBlip {
                    name: blip.name,
                    ring,
                    quadrant,
                }
            })
            .collect();

        self.completion_stats = Some(CompletionStats {
            total_blips,
            total_adrs,
            coverage,
            recent,
        });

        if let Ok(mut effect) = self.completion_fx.lock() {
            *effect = Some(fx::fade_from_fg(
                CoreColor::Yellow,
                (800, Interpolation::SineInOut),
            ));
        }
    }

    pub fn ensure_ring_pie_fx(&self) {
        let Ok(mut effect) = self.ring_pie_fx.lock() else {
            return;
        };

        if effect.is_some() {
            return;
        }

        let area = self.ring_pie_area.lock().map_or(None, |area| *area);
        let Some(area) = area else {
            return;
        };

        let mut key_filters = Vec::new();
        let row_count = 4_u16.min(area.height);
        for row in 0..row_count {
            let key_area = Rect {
                x: area.x,
                y: area.y + row,
                width: 2,
                height: 1,
            };
            key_filters.push(CellFilter::Area(key_area));
        }

        let filter = CellFilter::AnyOf(key_filters);
        let shimmer = fx::ping_pong(fx::fade_from_fg(
            Color::White,
            (2400, Interpolation::SineInOut),
        ))
        .with_filter(filter);

        *effect = Some(fx::repeating(shimmer));
    }

    pub fn reset(&mut self) {
        self.input_state = InputState::WaitingForCommand;
        self.current_input.clear();
        self.blip_data = BlipData::new();
        self.input_mode = None;
        self.adr_status = None;
        self.status_message.clear();
        self.save_notice_until = None;
        self.completion_stats = None;
        if let Ok(mut effect) = self.completion_fx.lock() {
            *effect = None;
        }
        if let Ok(mut effect) = self.ring_pie_fx.lock() {
            *effect = None;
        }
        if let Ok(mut area) = self.ring_pie_area.lock() {
            *area = None;
        }
        self.quadrant_selection_index = 0;
        self.ring_selection_index = 0;
        self.adr_status_selection_index = 0;
        self.input_mode_selection_index = 0;
        self.chart_tab_index = 0;
        self.last_checked_blip_name = None;
        self.last_blip_name_exists = false;
        self.settings_selection_index = 0;
        self.settings_editing = false;
        self.settings_input.clear();
        self.selected_adr_index = 0;
        self.adrs.clear();
        self.adr_filter_name = None;
        self.blip_action_index = 0;
        self.adr_action_index = 0;
        self.edit_adr_state = None;
        self.search_query.clear();
        self.search_active = false;
        self.filtered_blip_indices.clear();
        self.filtered_adr_indices.clear();
    }

    pub fn toggle_animation_pause(&mut self) {
        self.animation_paused = !self.animation_paused;
        self.status_message = if self.animation_paused {
            "Animation paused".to_string()
        } else {
            "Animation resumed".to_string()
        };
    }

    pub fn apply_search_filter(&mut self) {
        if !self.search_active || self.search_query.trim().is_empty() {
            self.filtered_blip_indices.clear();
            self.filtered_adr_indices.clear();
            self.search_result_index = 0;
            self.search_throbber_state = throbber_widgets_tui::ThrobberState::default();
            return;
        }

        let matcher = SkimMatcherV2::default();
        let query = self.search_query.trim();

        let mut blip_scores = self
            .blips
            .iter()
            .enumerate()
            .filter_map(|(index, blip)| {
                let mut candidate = blip.name.clone();
                if let Some(tag) = blip.tag.as_ref() {
                    candidate.push(' ');
                    candidate.push_str(tag);
                }
                if let Some(description) = blip.description.as_ref() {
                    candidate.push(' ');
                    candidate.push_str(description);
                }
                if let Some(ring) = blip.ring {
                    candidate.push(' ');
                    candidate.push_str(ring.as_str());
                }
                if let Some(quadrant) = blip.quadrant {
                    candidate.push(' ');
                    candidate.push_str(quadrant.as_str());
                }

                matcher
                    .fuzzy_match(&candidate, query)
                    .map(|score| (index, score))
            })
            .collect::<Vec<_>>();

        blip_scores.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        self.filtered_blip_indices = blip_scores.into_iter().map(|(index, _)| index).collect();

        let mut adr_scores = self
            .adrs
            .iter()
            .enumerate()
            .filter_map(|(index, adr)| {
                let candidate = format!(
                    "{} {} {} {}",
                    adr.title, adr.blip_name, adr.status, adr.timestamp
                );
                matcher
                    .fuzzy_match(&candidate, query)
                    .map(|score| (index, score))
            })
            .collect::<Vec<_>>();

        adr_scores.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        self.filtered_adr_indices = adr_scores.into_iter().map(|(index, _)| index).collect();

        let total_results = self.filtered_blip_indices.len() + self.filtered_adr_indices.len();
        if total_results == 0 {
            self.search_result_index = 0;
        } else if self.search_result_index >= total_results {
            self.search_result_index = total_results.saturating_sub(1);
        }

        self.selected_blip_index = 0;
        self.selected_adr_index = 0;
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.search_active = false;
        self.filtered_blip_indices.clear();
        self.filtered_adr_indices.clear();
        self.search_result_index = 0;
        self.search_throbber_state = throbber_widgets_tui::ThrobberState::default();
        self.selected_blip_index = 0;
        self.selected_adr_index = 0;
    }

    pub async fn generate_file(&mut self) -> Result<PathBuf> {
        let input_mode = self
            .input_mode
            .ok_or_else(|| color_eyre::eyre::eyre!("No input mode selected"))?;
        let target_dir = match input_mode {
            InputMode::Adr => self.actions.adrs_dir.clone(),
            InputMode::Blip => self.actions.blips_dir.clone(),
        };

        let id = self.actions.next_id(input_mode).await?;

        let timestamp = chrono::Utc::now().format("%Y-%m-%d").to_string();

        let sanitized_name = self.blip_data.name.replace(' ', "-").to_lowercase();
        let date_prefix = timestamp.split('T').next().unwrap_or("");
        let file_name = format!("{date_prefix}-{sanitized_name}");
        let file_path = get_file_path(&target_dir, &file_name);

        let content = match input_mode {
            InputMode::Adr => {
                let adr_status = self
                    .adr_status
                    .ok_or_else(|| color_eyre::eyre::eyre!("ADR status missing"))?;

                let adr_params = AdrMetadataParams {
                    id,
                    title: self.blip_data.name.clone(),
                    blip_name: self.blip_data.name.clone(),
                    status: adr_status.as_str().to_string(),
                    created: timestamp.clone(),
                };

                self.actions.insert_adr(&adr_params).await?;

                if let Some(existing_blip) = self
                    .blips
                    .iter()
                    .find(|blip| blip.name == self.blip_data.name)
                {
                    let params = BlipUpdateParams {
                        id: existing_blip.id,
                        name: None,
                        ring: None,
                        quadrant: None,
                        tag: None,
                        description: None,
                        adr_id: Some(id),
                    };

                    self.actions.update_blip(&params).await?;
                }

                self.generate_adr_content(
                    &id.to_string(),
                    &timestamp,
                    adr_status,
                    self.blip_data.name.as_str(),
                )
            }
            InputMode::Blip => {
                let quadrant = self
                    .blip_data
                    .quadrant
                    .ok_or_else(|| color_eyre::eyre::eyre!("Quadrant selection missing"))?;
                let ring = self
                    .blip_data
                    .ring
                    .ok_or_else(|| color_eyre::eyre::eyre!("Ring selection missing"))?;

                if self
                    .actions
                    .blip_exists_by_name(&self.blip_data.name)
                    .await?
                {
                    return Err(color_eyre::eyre::eyre!(
                        "Blip already exists: {}",
                        self.blip_data.name
                    ));
                }

                let blip_params = BlipMetadataParams {
                    id,
                    name: self.blip_data.name.clone(),
                    ring,
                    quadrant,
                    tag: String::new(),
                    author: self.actions.author_name.clone(),
                    has_adr: "false".to_string(),
                    description: String::new(),
                    created: timestamp.clone(),
                    adr_id: None,
                };
                self.actions.insert_blip(&blip_params).await?;
                self.fetch_blips().await?;

                self.generate_blip_content(&id.to_string(), &timestamp, quadrant, ring)
            }
        };

        std::fs::create_dir_all(target_dir)?;
        std::fs::write(&file_path, content)?;

        Ok(file_path)
    }

    // Simple sync content generation functions that don't require async operations
    pub fn generate_adr_content(
        &self,
        id: &str,
        timestamp: &str,
        status: AdrStatus,
        blip_name: &str,
    ) -> String {
        let blip = if blip_name.is_empty() {
            "null"
        } else {
            blip_name
        };

        format!(
            r#"---
 id: "{}"
 title: "{}"
 blip: {}
 date: {}
 status: "{}"
 authors: ["{}"]
 ---
 
 # {}
 
 ## Context
 
 [Describe the context and problem statement, e.g., in free form using two to three sentences. You may want to articulate the problem in form of a question.]
 
 ## Decision
 
 [Describe the decision that was made]
 
 ## Consequences
 
 [Describe the resulting context, after applying the decision. All consequences should be listed here, not just the "positive" ones. A particular decision may have positive, negative, and neutral consequences, but all of them affect the team and project in the future.]
 "#,
            id,
            self.blip_data.name,
            blip,
            timestamp,
            status.as_str(),
            self.actions.author_name,
            self.blip_data.name
        )
    }

    // Simple sync content generation functions that don't require async operations
    pub fn generate_blip_content(
        &self,
        id: &str,
        timestamp: &str,
        quadrant: Quadrant,
        ring: Ring,
    ) -> String {
        let quadrant = quadrant.as_str();
        let ring = ring.as_str();

        format!(
            r#"---
 id: "{}"
 name: "{}"
 ring: "{}"
 quadrant: "{}"
 tags: [""]
 authors: ["{}"]
 hasAdr: false
 adrId: null
 description: {{{{description}}}}
 created: "{}"
 ---
 
 # "{}"
 **Ring**: "{}"
 **Quadrant**: "{}"
 **New**: false
 **Description**: {{{{description}}}}
 **has ADR**: false
 "#,
            id,
            self.blip_data.name,
            ring,
            quadrant,
            self.actions.author_name,
            timestamp,
            self.blip_data.name,
            ring,
            quadrant
        )
    }

    pub async fn fetch_blips(&mut self) -> Result<()> {
        self.blips = self.actions.fetch_blips().await?;
        self.apply_search_filter();
        Ok(())
    }

    pub async fn fetch_adrs_for_blip(&mut self, blip_name: &str) -> Result<()> {
        self.adrs = self.actions.fetch_adrs_for_blip(blip_name).await?;
        self.adr_filter_name = if blip_name.is_empty() {
            None
        } else {
            Some(blip_name.to_string())
        };
        self.apply_search_filter();
        Ok(())
    }

    /// Updates a blip in the database and refreshes the blips list
    pub async fn update_blip(&mut self, params: BlipUpdateParams) -> Result<()> {
        let blip_id = params.id;
        self.actions.update_blip(&params).await?;
        self.fetch_blips().await?;
        self.refresh_edit_blip_state(blip_id);
        self.status_message = "Blip updated successfully".to_string();
        if let Err(e) = self.sync_blip_file(blip_id) {
            self.status_message = format!("Blip saved to DB, but markdown sync failed: {e}");
        }
        Ok(())
    }

    fn sync_blip_file(&self, blip_id: i32) -> Result<()> {
        let Some(blip) = self.blips.iter().find(|item| item.id == blip_id) else {
            return Ok(());
        };

        let ring = blip
            .ring
            .map_or_else(String::new, |ring| ring.as_str().to_string());
        let quadrant = blip
            .quadrant
            .map_or_else(String::new, |quadrant| quadrant.as_str().to_string());
        let sanitized_name = blip.name.replace(' ', "-").to_lowercase();
        let date_prefix = blip.created.split('T').next().unwrap_or("None");
        let file_name = format!("{date_prefix}-{sanitized_name}");
        let mut file_path = get_file_path(&self.actions.blips_dir, &file_name);

        if !file_path.exists() {
            if let Ok(entries) = std::fs::read_dir(&self.actions.blips_dir) {
                let suffix = format!("-{sanitized_name}.mdx");
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.ends_with(&suffix))
                    {
                        file_path = path;
                        break;
                    }
                }
            }
        }

        if !file_path.exists() {
            std::fs::create_dir_all(&self.actions.blips_dir)?;
            file_path = get_file_path(&self.actions.blips_dir, &file_name);
        }

        let created = blip.created.clone();
        let content = format!(
            r#"---
 id: "{}"
 name: "{}"
 ring: "{}"
 quadrant: "{}"
 tags: ["{}"]
 authors: ["{}"]
 hasAdr: {}
 adrId: {}
 description: {{{{description}}}}
 created: "{}"
 ---
 
 # "{}"
 **Ring**: "{}"
 **Quadrant**: "{}"
 **New**: false
 **Description**: {{{{description}}}}
 **has ADR**: {}
 "#,
            blip.id,
            blip.name,
            ring,
            quadrant,
            blip.tag.clone().unwrap_or_default(),
            self.actions.author_name,
            blip.has_adr,
            blip.adr_id
                .map_or_else(|| "null".to_string(), |id| id.to_string()),
            created,
            blip.name,
            ring,
            quadrant,
            blip.has_adr,
        );

        std::fs::write(file_path, content)?;
        Ok(())
    }

    fn refresh_edit_blip_state(&mut self, blip_id: i32) {
        let Some(blip) = self.blips.iter().find(|item| item.id == blip_id) else {
            return;
        };

        if let Some(edit_state) = &mut self.edit_blip_state {
            *edit_state = EditBlipState::from_blip(blip);
            edit_state.field = EditField::Save;
        }
    }

    /// Updates an ADR in the database and refreshes the ADR list
    pub async fn update_adr(&mut self, params: AdrUpdateParams) -> Result<()> {
        let adr_id = params.id;
        self.actions.update_adr(&params).await?;
        let filter = self.adr_filter_name.clone().unwrap_or_default();
        self.fetch_adrs_for_blip(&filter).await?;
        self.refresh_edit_adr_state(adr_id);
        self.status_message = "ADR updated successfully".to_string();
        if let Err(e) = self.sync_adr_file(adr_id) {
            self.status_message = format!("ADR saved to DB, but markdown sync failed: {e}");
        }
        Ok(())
    }

    fn sync_adr_file(&self, adr_id: i32) -> Result<()> {
        let Some(adr) = self.adrs.iter().find(|item| item.id == adr_id) else {
            return Ok(());
        };

        let status = AdrStatus::parse(&adr.status).unwrap_or(AdrStatus::Proposed);
        let sanitized_name = adr.blip_name.replace(' ', "-").to_lowercase();
        let date_prefix = adr.timestamp.split('T').next().unwrap_or("None");
        let file_name = format!("{date_prefix}-{sanitized_name}");
        let mut file_path = get_file_path(&self.actions.adrs_dir, &file_name);

        if !file_path.exists() {
            if let Ok(entries) = std::fs::read_dir(&self.actions.adrs_dir) {
                let suffix = format!("-{sanitized_name}.mdx");
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.ends_with(&suffix))
                    {
                        file_path = path;
                        break;
                    }
                }
            }
        }

        if !file_path.exists() {
            std::fs::create_dir_all(&self.actions.adrs_dir)?;
            file_path = get_file_path(&self.actions.adrs_dir, &file_name);
        }

        let blip = if adr.blip_name.is_empty() {
            "null"
        } else {
            &adr.blip_name
        };

        let content = format!(
            r#"---
 id: "{}"
 title: "{}"
 blip: {}
 date: {}
 status: "{}"
 ---

 # {}

 ## Context

 [Describe the context and problem statement, e.g., in free form using two to three sentences. You may want to articulate the problem in form of a question.]

 ## Decision

 [Describe the decision that was made]

 ## Consequences

 [Describe the resulting context, after applying the decision. All consequences should be listed here, not just the "positive" ones. A particular decision may have positive, negative, and neutral consequences, but all of them affect the team and project in the future.]
 "#,
            adr.id,
            adr.title,
            blip,
            adr.timestamp,
            status.as_str(),
            adr.title,
        );

        std::fs::write(file_path, content)?;
        Ok(())
    }

    fn refresh_edit_adr_state(&mut self, adr_id: i32) {
        let Some(adr) = self.adrs.iter().find(|item| item.id == adr_id) else {
            return;
        };

        let status = AdrStatus::parse(&adr.status).unwrap_or(AdrStatus::Proposed);

        if let Some(edit_state) = &mut self.edit_adr_state {
            edit_state.id = adr.id;
            edit_state.title.clone_from(&adr.title);
            edit_state.status = status;
            edit_state.field = AdrEditField::Save;
        }
    }
}

pub fn get_file_path(adrs_dir: impl AsRef<Path>, file_name: &str) -> PathBuf {
    adrs_dir.as_ref().join(format!("{file_name}.mdx"))
}
