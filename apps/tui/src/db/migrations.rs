use crate::config::init_app_config;
use crate::db::models::{AdrMetadataParams, BlipMetadataParams};
use color_eyre::Result;
use sqlx::{
    migrate::MigrateDatabase, query, query_scalar, sqlite::SqlitePoolOptions, Sqlite, SqlitePool,
};

/// Sets up the database by creating the necessary tables if they don't exist
pub async fn setup_database(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Create the adr_log table
    query(
        "CREATE TABLE IF NOT EXISTS adr_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            blip_name TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'proposed',
            timestamp TEXT NOT NULL,
            UNIQUE(title, timestamp)
        )",
    )
    .execute(pool)
    .await?;

    // Create the blip table
    query(
        "CREATE TABLE IF NOT EXISTS blip (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            ring TEXT,
            quadrant TEXT,
            tag TEXT,
            description TEXT,
            created TEXT NOT NULL,
            hasAdr BOOLEAN DEFAULT FALSE,
            adr_id INTEGER
        )",
    )
    .execute(pool)
    .await?;

    ensure_column_exists(
        pool,
        "adr_log",
        "blip_name",
        "ALTER TABLE adr_log ADD COLUMN blip_name TEXT NOT NULL DEFAULT ''",
    )
    .await?;

    ensure_column_exists(
        pool,
        "adr_log",
        "status",
        "ALTER TABLE adr_log ADD COLUMN status TEXT NOT NULL DEFAULT 'proposed'",
    )
    .await?;

    ensure_column_exists(
        pool,
        "blip",
        "adr_id",
        "ALTER TABLE blip ADD COLUMN adr_id INTEGER",
    )
    .await?;

    Ok(())
}

async fn ensure_column_exists(
    pool: &SqlitePool,
    table: &str,
    column: &str,
    alter_statement: &str,
) -> Result<(), sqlx::Error> {
    let count: i64 = query_scalar(&format!(
        "SELECT COUNT(*) FROM pragma_table_info('{table}') WHERE name = ?",
    ))
    .bind(column)
    .fetch_one(pool)
    .await?;

    if count == 0 {
        query(alter_statement).execute(pool).await?;
    }

    Ok(())
}

/// Creates a database connection pool using the database URL from config
pub async fn create_database_pool() -> Result<SqlitePool> {
    // Get database URL from config
    let (database_url, _) = init_app_config()?;

    eprintln!("Initializing database with URL: {database_url}");

    // Extract the database path from the URL for permission checks
    let db_path = match extract_db_path_from_url(&database_url) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error extracting database path: {e}");
            return Err(color_eyre::eyre::eyre!("Invalid database URL format: {e}"));
        }
    };
    eprintln!("Extracted database path: {db_path}");
    // Add extra debug print for clarity
    println!("[DEBUG] Will connect to SQLite DB at: {db_path} (from URL: {database_url})");
    // Check if parent directory exists and is writable
    if let Some(parent) = std::path::Path::new(&db_path).parent() {
        if !parent.exists() {
            eprintln!("Creating parent directory: {}", parent.display());
            std::fs::create_dir_all(parent).map_err(|e| {
                eprintln!("Failed to create directory: {e}");
                color_eyre::eyre::eyre!("Failed to create database directory: {e}")
            })?;
        }

        // Check if directory is writable
        let metadata = parent.metadata().map_err(|e| {
            eprintln!("Failed to get directory metadata: {e}");
            color_eyre::eyre::eyre!("Failed to access directory metadata: {e}")
        })?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = metadata.permissions().mode();
            eprintln!("Directory permissions: {mode:o}");
            if mode & 0o200 == 0 {
                return Err(color_eyre::eyre::eyre!(
                    "Database directory is not writable"
                ));
            }
        }
    }

    // Create the database if it doesn't exist
    // Validate database path and permissions
    eprintln!("Validating database path...");
    let db_file = std::path::Path::new(&db_path);

    // If database exists, check if it's readable/writable
    if db_file.exists() {
        eprintln!("Database file exists, checking permissions");
        match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(db_file)
        {
            Ok(_) => eprintln!("Database file is readable and writable"),
            Err(e) => {
                eprintln!("Database file permission error: {e}");
                return Err(color_eyre::eyre::eyre!(
                    "Database file permission error: {e}"
                ));
            }
        }
    }

    // Create the database if it doesn't exist
    eprintln!("Checking if database exists in SQLx...");
    let db_exists = match Sqlite::database_exists(&database_url).await {
        Ok(exists) => exists,
        Err(e) => {
            eprintln!("Error checking if database exists: {e}");
            return Err(color_eyre::eyre::eyre!("Error checking database: {e}"));
        }
    };

    if db_exists {
        eprintln!("Database already exists in SQLx");
    } else {
        eprintln!("Database does not exist, creating it now");
        Sqlite::create_database(&database_url).await.map_err(|e| {
            eprintln!("Failed to create database: {e}");
            color_eyre::eyre::eyre!("Failed to create SQLite database: {e}")
        })?;
    }
    // Create a connection pool with SQLite-specific options
    eprintln!("Creating connection pool with improved settings");
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        // Add SQLite connection options for better reliability
        .after_connect(|conn, _| {
            Box::pin(async move {
                use sqlx::Executor as _;
                // Enable foreign keys
                conn.execute("PRAGMA foreign_keys = ON;").await?;
                // Set journal mode to WAL for better concurrency
                conn.execute("PRAGMA journal_mode = WAL;").await?;
                // Set synchronous mode for better reliability
                conn.execute("PRAGMA synchronous = NORMAL;").await?;
                Ok(())
            })
        })
        .connect(&database_url)
        .await
        .map_err(|e| {
            eprintln!("Failed to connect to database: {e}");
            color_eyre::eyre::eyre!("Failed to connect to SQLite database: {e}")
        })?;

    eprintln!("Connection pool created successfully");

    // Set up the database schema
    eprintln!("Setting up database schema");
    setup_database(&pool).await.map_err(|e| {
        eprintln!("Failed to set up database schema: {e}");
        color_eyre::eyre::eyre!("Failed to set up database schema: {e}")
    })?;

    eprintln!("Database initialization completed successfully");
    Ok(pool)
}

/// Helper function to extract the database path from a SQLite URL
fn extract_db_path_from_url(url: &str) -> Result<String, color_eyre::eyre::Error> {
    if !url.starts_with("sqlite://") {
        return Err(color_eyre::eyre::eyre!("Not a valid SQLite URL: {url}"));
    }

    // Split the URL into parts
    let path_part = url.trim_start_matches("sqlite://");

    // Handle different path formats
    if cfg!(windows) {
        // Windows: sqlite:///C:/path or sqlite://C:/path
        if let Some(drive_idx) = path_part.find(':') {
            if drive_idx > 0 {
                // Only strip leading slash if it's before the drive letter
                let path = path_part
                    .strip_prefix('/')
                    .map_or_else(|| path_part.to_string(), std::string::ToString::to_string);

                return Ok(path);
            }
        }
    }

    // Unix-like absolute path: sqlite:///path
    if path_part.starts_with('/') {
        // For absolute paths, make sure we keep the leading slash
        return Ok(format!("/{}", path_part.trim_start_matches('/')));
    }

    // Relative path: sqlite://path
    Ok(path_part.to_string())
}

/// Creates a database connection pool with a specified URL
#[allow(dead_code)]
pub async fn create_database_pool_with_url(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    // Create the database if it doesn't exist
    if !Sqlite::database_exists(database_url).await.unwrap_or(false) {
        Sqlite::create_database(database_url).await?;
    }

    // Create a connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await?;

    // Set up the database schema
    setup_database(&pool).await?;

    Ok(pool)
}

/// Gets the next available ID for a new ADR entry
pub async fn get_next_id(pool: &SqlitePool) -> Result<i32, sqlx::Error> {
    let id: i32 = query_scalar("SELECT COALESCE(MAX(id), 0) + 1 as id FROM adr_log")
        .fetch_one(pool)
        .await?;

    Ok(id)
}

/// Gets the next available ID for a new Blip entry
pub async fn get_next_blip_id(pool: &SqlitePool) -> Result<i32, sqlx::Error> {
    let id: i32 = query_scalar("SELECT COALESCE(MAX(id), 0) + 1 as id FROM blip")
        .fetch_one(pool)
        .await?;

    Ok(id)
}

/// Inserts a new ADR record into the database
#[allow(dead_code)]
pub async fn insert_new_adr(
    pool: &SqlitePool,
    id: i32,
    title: &str,
    timestamp: &str,
) -> Result<(), sqlx::Error> {
    query("INSERT INTO adr_log (id, title, timestamp) VALUES (?, ?, ?)")
        .bind(id)
        .bind(title)
        .bind(timestamp)
        .execute(pool)
        .await?;

    Ok(())
}

/// Inserts a new ADR record into the database using AdrMetadataParams
pub async fn insert_new_adr_with_params(
    pool: &SqlitePool,
    params: &AdrMetadataParams,
) -> Result<(), sqlx::Error> {
    query("INSERT INTO adr_log (id, title, blip_name, status, timestamp) VALUES (?, ?, ?, ?, ?)")
        .bind(params.id)
        .bind(&params.title)
        .bind(&params.blip_name)
        .bind(&params.status)
        .bind(&params.created)
        .execute(pool)
        .await?;

    Ok(())
}

/// Inserts a new Blip record into the database
pub async fn insert_new_blip(
    pool: &SqlitePool,
    blip_params: &BlipMetadataParams,
) -> Result<(), sqlx::Error> {
    // Get the next available ID for the Blip
    let id = get_next_blip_id(pool).await?;

    // Check if an ADR exists with the same name
    let has_adr: bool =
        query_scalar("SELECT EXISTS(SELECT 1 FROM adr_log WHERE title = ?) AS has_adr")
            .bind(&blip_params.name)
            .fetch_one(pool)
            .await?;

    // Insert the blip with the appropriate hasAdr flag and explicit ID
    query(
        "INSERT INTO blip (id, name, ring, quadrant, tag, description, created, hasAdr, adr_id) \
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(&blip_params.name)
    .bind(blip_params.ring)
    .bind(blip_params.quadrant)
    .bind(&blip_params.tag)
    .bind(&blip_params.description)
    .bind(&blip_params.created)
    .bind(i32::from(has_adr)) // Convert bool to i32 for SQLite
    .bind(blip_params.adr_id)
    .execute(pool)
    .await?;

    Ok(())
}
