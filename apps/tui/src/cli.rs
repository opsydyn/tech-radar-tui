use clap::{CommandFactory, Parser};

#[derive(Debug, Parser)]
#[command(name = "ratatui_adr-gen", version, about = "Tech Radar TUI")]
pub struct CliArgs {
    /// Print stats and exit
    #[arg(long)]
    pub headless: bool,

    /// Print headless stats as JSON
    #[arg(long)]
    pub json: bool,

    /// Enable debug logging
    #[arg(long)]
    pub debug: bool,

    /// Override database path
    #[arg(long, value_name = "PATH")]
    pub db: Option<String>,

    /// Override ADR output directory
    #[arg(long = "adr-dir", value_name = "PATH")]
    pub adr_dir: Option<String>,

    /// Override Blip output directory
    #[arg(long = "blip-dir", value_name = "PATH")]
    pub blip_dir: Option<String>,
}

impl CliArgs {
    pub fn apply_env_overrides(&self) {
        if let Some(db) = &self.db {
            std::env::set_var("DATABASE_NAME", db);
        }
        if let Some(dir) = &self.adr_dir {
            std::env::set_var("ADR_DIR", dir);
        }
        if let Some(dir) = &self.blip_dir {
            std::env::set_var("BLIP_DIR", dir);
        }
        if self.debug {
            std::env::set_var("DEBUG", "1");
        }
    }

    pub fn help_text() -> String {
        let mut command = Self::command();
        let mut buffer = Vec::new();
        command.write_help(&mut buffer).ok();
        String::from_utf8_lossy(&buffer).to_string()
    }
}
