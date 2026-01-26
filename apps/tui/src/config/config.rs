use color_eyre::eyre::eyre;
use dotenv::dotenv;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use std::str;

/// Initializes the application configuration
/// Returns a tuple containing the database URL and author name
pub fn init_app_config() -> color_eyre::eyre::Result<(String, String)> {
    // Load environment variables from .env file
    dotenv().ok();
    
    // Get current directory
    let base_dir: PathBuf = env::current_dir()?;
    
    // Get database configuration from environment variables
    let db_name = env::var("DATABASE_NAME").unwrap_or_else(|_| "adrs.db".to_string());
    
    // Create the database path relative to the current directory
    let database_path = base_dir.join(&db_name);
    
    // Create parent directory if it doesn't exist
    if let Some(parent) = database_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    
    // For SQLite, we just need the path as a string
    // We don't use canonicalize() because the file might not exist yet
    let path_str = database_path.to_str()
        .ok_or_else(|| eyre!("Invalid database path"))?
        .to_string();
    
    // Format the database URL - ensure it's in the correct format for SQLx
    // SQLx requires:
    // - For absolute paths: sqlite:///absolute/path/to/file.db (3 slashes)
    // Format the database URL - ensure it's in the correct format for SQLx
    // SQLx requires:
    // - For absolute paths: sqlite:///absolute/path/to/file.db (3 slashes total)
    // - For relative paths: sqlite://relative/path/to/file.db (2 slashes total)
    
    // Strip any leading slashes from the path to avoid double slashes
    let clean_path = path_str.trim_start_matches('/');
    
    let database_url = if database_path.is_absolute() {
        // Absolute path needs exactly 3 slashes total (sqlite:///)
        eprintln!("Using absolute database path: {path_str}");
        format!("sqlite:///{clean_path}")
    } else {
        // Relative path needs exactly 2 slashes total (sqlite://)
        eprintln!("Using relative database path: {path_str}");
        format!("sqlite://{clean_path}")
    };
    // Get author name from git config
    let author_name = get_github_username().unwrap_or_else(|_| "unknown author".to_string());

    Ok((database_url, author_name))
}

/// Gets the GitHub username from git config
fn get_github_username() -> color_eyre::eyre::Result<String> {
    let username = Command::new("git")
        .args(["config", "--get", "user.name"])
        .output()?;

    let username_str = str::from_utf8(&username.stdout)?
        .trim()
        .to_string();
    
    if username_str.is_empty() {
        return Err(eyre!("Git username not found"));
    }
    
    Ok(username_str)
}

/// Gets the directory path for storing ADRs
pub fn get_adrs_dir() -> PathBuf {
    env::var("ADR_DIR").map_or_else(|_| PathBuf::from("./adrs"), PathBuf::from)
}

/// Gets the directory path for storing blips
#[allow(dead_code)]
pub fn get_blips_dir() -> PathBuf {
    env::var("BLIP_DIR").map_or_else(|_| PathBuf::from("./blips"), PathBuf::from)
}
