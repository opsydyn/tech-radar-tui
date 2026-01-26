use crate::config::{get_adrs_dir, get_blips_dir, init_app_config};
use crate::db::models::{AdrMetadataParams, BlipMetadataParams, BlipRecord};
use crate::db::queries::{blip_exists_by_name, update_blip, BlipUpdateParams};
use crate::db::{
    create_database_pool, get_next_blip_id, get_next_id, insert_new_adr_with_params,
    insert_new_blip,
};
use crate::{Quadrant, Ring};
use color_eyre::Result;
use sqlx::SqlitePool;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};

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

    pub const fn is_complete(&self) -> bool {
        !self.name.is_empty() && self.quadrant.is_some() && self.ring.is_some()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum InputState {
    WaitingForCommand,
    EnteringTechnology,
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
    BlipActions,
    BlipDetails,
    EditBlip,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum InputMode {
    Adr,
    Blip,
}

/// Represents which field is currently being edited in the EditBlip screen
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditField {
    Name,
    Ring,
    Quadrant,
    Tag,
    Description,
}

/// Holds the temporary state of a blip being edited
#[derive(Debug, Clone)]
pub struct EditBlipState {
    pub field: EditField,
    pub name: String,
    pub ring: String,
    pub quadrant: String,
    pub tag: String,
    pub description: String,
    pub editing: bool,         // Whether we're actively editing the current field
    pub ring_index: usize,     // Index of the currently selected ring
    pub quadrant_index: usize, // Index of the currently selected quadrant
}

impl EditBlipState {
    /// Create a new EditBlipState from a BlipRecord
    pub fn from_blip(blip: &BlipRecord) -> Self {
        // Determine the initial ring index based on the blip's ring value
        let ring_index = match blip.ring.as_deref().unwrap_or("") {
            "assess" => 1,
            "trial" => 2,
            "adopt" => 3,
            _ => 0,
        };

        // Determine the initial quadrant index based on the blip's quadrant value
        let quadrant_index = match blip.quadrant.as_deref().unwrap_or("") {
            "languages" => 1,
            "tools" => 2,
            "techniques" => 3,
            _ => 0,
        };

        Self {
            field: EditField::Name,
            name: blip.name.clone(),
            ring: blip.ring.clone().unwrap_or_default(),
            quadrant: blip.quadrant.clone().unwrap_or_default(),
            tag: blip.tag.clone().unwrap_or_default(),
            description: blip.description.clone().unwrap_or_default(),
            editing: false,
            ring_index,
            quadrant_index,
        }
    }

    /// Get all valid ring options
    pub const fn ring_options() -> &'static [&'static str] {
        &["hold", "assess", "trial", "adopt"]
    }

    /// Get all valid quadrant options
    pub const fn quadrant_options() -> &'static [&'static str] {
        &["platforms", "languages", "tools", "techniques"]
    }

    /// Cycle to the next ring option
    pub fn next_ring(&mut self) {
        self.ring_index = (self.ring_index + 1) % Self::ring_options().len();
        self.ring = Self::ring_options()[self.ring_index].to_string();
    }

    /// Cycle to the previous ring option
    pub fn prev_ring(&mut self) {
        self.ring_index =
            (self.ring_index + Self::ring_options().len() - 1) % Self::ring_options().len();
        self.ring = Self::ring_options()[self.ring_index].to_string();
    }

    /// Cycle to the next quadrant option
    pub fn next_quadrant(&mut self) {
        self.quadrant_index = (self.quadrant_index + 1) % Self::quadrant_options().len();
        self.quadrant = Self::quadrant_options()[self.quadrant_index].to_string();
    }

    /// Cycle to the previous quadrant option
    pub fn prev_quadrant(&mut self) {
        self.quadrant_index = (self.quadrant_index + Self::quadrant_options().len() - 1)
            % Self::quadrant_options().len();
        self.quadrant = Self::quadrant_options()[self.quadrant_index].to_string();
    }
}

#[derive(Debug)]
pub struct App {
    pub running: bool,
    pub input_state: InputState,
    pub current_input: String,
    pub blip_data: BlipData,
    pub input_mode: Option<InputMode>,
    pub status_message: String,
    pub adrs_dir: PathBuf,
    pub blips_dir: PathBuf,
    pub animation_counter: f64,
    pub last_frame: Instant,
    pub show_help: bool,
    pub db_pool: Option<SqlitePool>,
    pub author_name: String,
    pub screen: AppScreen,
    pub blips: Vec<crate::db::models::BlipRecord>,
    pub selected_blip_index: usize,
    pub edit_blip_state: Option<EditBlipState>,
    pub blip_action_index: usize,
    pub selected_adr_index: usize,
    pub adrs: Vec<crate::db::models::AdrRecord>,
    pub adr_filter_name: Option<String>,
    pub quadrant_selection_index: usize,
    pub ring_selection_index: usize,
    pub input_mode_selection_index: usize,
    pub chart_tab_index: usize,
    pub last_checked_blip_name: Option<String>,
    pub last_blip_name_exists: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: true,
            input_state: InputState::WaitingForCommand,
            current_input: String::new(),
            blip_data: BlipData::new(),
            input_mode: None,
            status_message: String::new(),
            adrs_dir: PathBuf::from("./adrs"),
            blips_dir: PathBuf::from("./blips"),
            animation_counter: 0.0,
            last_frame: Instant::now(),
            show_help: false,
            db_pool: None,
            author_name: String::new(),
            screen: AppScreen::Main,
            blips: Vec::new(),
            selected_blip_index: 0,
            edit_blip_state: None,
            blip_action_index: 0,
            selected_adr_index: 0,
            adrs: Vec::new(),
            adr_filter_name: None,
            quadrant_selection_index: 0,
            ring_selection_index: 0,
            input_mode_selection_index: 0,
            chart_tab_index: 0,
            last_checked_blip_name: None,
            last_blip_name_exists: false,
        }
    }

    pub async fn initialize_db(&mut self) -> Result<()> {
        // Initialize app configuration
        let (_, author_name) = init_app_config()?;
        self.author_name = author_name;

        // Get directories from config
        self.adrs_dir = get_adrs_dir();
        self.blips_dir = get_blips_dir();

        // Create database pool
        self.db_pool = Some(create_database_pool().await?);

        self.fetch_blips().await?;

        Ok(())
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_frame);
        self.last_frame = now;

        // Update animation counter (cycles between 0 and 2*PI)
        self.animation_counter += delta.as_secs_f64() * 2.0;
        if self.animation_counter > 2.0 * std::f64::consts::PI {
            self.animation_counter -= 2.0 * std::f64::consts::PI;
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
                self.quadrant_selection_index = 0;
                InputState::ChoosingQuadrant
            }
            InputState::ChoosingQuadrant => {
                self.ring_selection_index = 0;
                InputState::ChoosingRing
            }
            InputState::ChoosingRing => {
                if self.blip_data.is_complete() {
                    // We'll handle file generation in the main event loop
                    // since we can't use async in this method
                    InputState::GeneratingFile
                } else {
                    self.status_message = "Missing data. Please complete all fields.".to_string();
                    InputState::ChoosingRing
                }
            }
            InputState::GeneratingFile => InputState::GeneratingFile, // Stay in this state until file is generated
            InputState::Completed => InputState::WaitingForCommand,
        };
        self.current_input.clear();
    }

    pub fn reset(&mut self) {
        self.input_state = InputState::WaitingForCommand;
        self.current_input.clear();
        self.blip_data = BlipData::new();
        self.input_mode = None;
        self.status_message.clear();
        self.quadrant_selection_index = 0;
        self.ring_selection_index = 0;
        self.input_mode_selection_index = 0;
        self.chart_tab_index = 0;
        self.last_checked_blip_name = None;
        self.last_blip_name_exists = false;
        self.selected_adr_index = 0;
        self.adrs.clear();
        self.adr_filter_name = None;
        self.blip_action_index = 0;
    }

    pub async fn generate_file(&mut self) -> Result<PathBuf> {
        let target_dir = match self.input_mode {
            Some(InputMode::Adr) => &self.adrs_dir,
            Some(InputMode::Blip) => &self.blips_dir,
            None => return Err(color_eyre::eyre::eyre!("No input mode selected")),
        };

        if !target_dir.exists() {
            fs::create_dir_all(target_dir)?;
        }

        let pool = self
            .db_pool
            .as_ref()
            .ok_or_else(|| color_eyre::eyre::eyre!("Database not initialized"))?;

        // Get the appropriate ID based on input mode
        let id = match self.input_mode {
            Some(InputMode::Adr) => get_next_id(pool).await?,
            Some(InputMode::Blip) => get_next_blip_id(pool).await?,
            None => return Err(color_eyre::eyre::eyre!("No input mode selected")),
        };

        let timestamp = chrono::Utc::now().format("%Y-%m-%d").to_string();

        let sanitized_name = self.blip_data.name.replace(' ', "-").to_lowercase();
        let date_prefix = timestamp.split('T').next().unwrap_or("");
        let file_name = format!("{date_prefix}-{sanitized_name}");
        let file_path = get_file_path(target_dir, &file_name);

        let quadrant = self
            .blip_data
            .quadrant
            .ok_or_else(|| color_eyre::eyre::eyre!("Quadrant selection missing"))?;
        let ring = self
            .blip_data
            .ring
            .ok_or_else(|| color_eyre::eyre::eyre!("Ring selection missing"))?;

        match self.input_mode {
            Some(InputMode::Adr) => {
                let adr_params = AdrMetadataParams {
                    id,
                    title: self.blip_data.name.clone(),
                    blip_name: self.blip_data.name.clone(),
                    created: timestamp.clone(),
                };

                insert_new_adr_with_params(pool, &adr_params).await?;

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
                    update_blip(pool, &params).await?;
                }
            }
            Some(InputMode::Blip) => {
                if blip_exists_by_name(pool, &self.blip_data.name).await? {
                    return Err(color_eyre::eyre::eyre!(
                        "Blip already exists: {}",
                        self.blip_data.name
                    ));
                }

                let quadrant_str = quadrant.as_str().to_string();
                let ring_str = ring.as_str().to_string();

                let blip_params = BlipMetadataParams {
                    id,
                    name: self.blip_data.name.clone(),
                    ring: ring_str,
                    quadrant: quadrant_str,
                    tag: String::new(),
                    author: self.author_name.clone(),
                    has_adr: "false".to_string(),
                    description: String::new(),
                    created: timestamp.clone(),
                    adr_id: None,
                };
                insert_new_blip(pool, &blip_params).await?;
                self.fetch_blips().await?;
            }
            None => {
                return Err(color_eyre::eyre::eyre!("No input mode selected"));
            }
        }

        let id_string = id.to_string();
        let content = match self.input_mode {
            Some(InputMode::Adr) => {
                self.generate_adr_content(&id_string, &timestamp, quadrant, ring)
            }
            Some(InputMode::Blip) => {
                self.generate_blip_content(&id_string, &timestamp, quadrant, ring)
            }
            None => return Err(color_eyre::eyre::eyre!("No input mode selected")),
        };

        let mut file = fs::File::create(&file_path)?;
        file.write_all(content.as_bytes())?;

        Ok(file_path)
    }

    // Simple sync content generation functions that don't require async operations
    pub fn generate_adr_content(
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
 title: "{}"
 blip: "{}"
 date: {}
 status: "accepted"
 quadrant: "{}"
 ring: "{}"
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
            self.blip_data.name,
            timestamp,
            quadrant,
            ring,
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
            self.author_name,
            timestamp,
            self.blip_data.name,
            ring,
            quadrant
        )
    }

    pub async fn fetch_blips(&mut self) -> Result<()> {
        use crate::db::queries::get_blips;
        if let Some(pool) = &self.db_pool {
            let blips = get_blips(pool).await?;
            self.blips = blips;
        }
        Ok(())
    }

    pub async fn fetch_adrs_for_blip(&mut self, blip_name: &str) -> Result<()> {
        use crate::db::queries::{get_adrs, get_adrs_by_blip_name};
        if let Some(pool) = &self.db_pool {
            if blip_name.is_empty() {
                let adrs = get_adrs(pool).await?;
                self.adrs = adrs;
                self.adr_filter_name = None;
            } else {
                let adrs = get_adrs_by_blip_name(pool, blip_name).await?;
                self.adrs = adrs;
                self.adr_filter_name = Some(blip_name.to_string());
            }
        }
        Ok(())
    }


    /// Updates a blip in the database and refreshes the blips list
    pub async fn update_blip(&mut self, params: BlipUpdateParams) -> Result<()> {
        if let Some(pool) = &self.db_pool {
            // Update the blip in the database
            update_blip(pool, &params).await?;

            // Refresh the blips list to show the updated data
            self.fetch_blips().await?;

            // Set a status message
            self.status_message = "Blip updated successfully".to_string();
        }
        Ok(())
    }
}

pub fn get_file_path(adrs_dir: impl AsRef<Path>, file_name: &str) -> PathBuf {
    adrs_dir.as_ref().join(format!("{file_name}.mdx"))
}
