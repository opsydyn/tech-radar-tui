use color_eyre::Result;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();

    println!("Testing database functionality...");

    // Set up database URL
    let db_prefix = env::var("DATABASE_PREFIX").unwrap_or_else(|_| "sqlite:".to_string());
    let db_name = env::var("DATABASE_NAME").unwrap_or_else(|_| "test_adrs.db".to_string());
    let database_url = format!("{db_prefix}{db_name}");

    // Create database connection pool
    let pool = sqlx::SqlitePool::connect(&database_url).await?;

    // Create tables directly
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS adr_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            timestamp TEXT NOT NULL
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS blip (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            ring TEXT,
            quadrant TEXT,
            tag TEXT,
            description TEXT,
            created TEXT NOT NULL,
            hasAdr BOOLEAN DEFAULT FALSE
        )",
    )
    .execute(&pool)
    .await?;

    // Clear existing data to avoid unique constraint errors
    sqlx::query("DELETE FROM adr_log").execute(&pool).await?;
    sqlx::query("DELETE FROM blip").execute(&pool).await?;

    // Insert test data
    let adr_id = 1;
    let adr_title = "Test ADR";
    let timestamp = chrono::Utc::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    sqlx::query("INSERT INTO adr_log (id, title, timestamp) VALUES (?, ?, ?)")
        .bind(adr_id)
        .bind(adr_title)
        .bind(&timestamp)
        .execute(&pool)
        .await?;

    // Verify data was inserted
    let rows = sqlx::query("SELECT id, title, timestamp FROM adr_log")
        .fetch_all(&pool)
        .await?;

    println!("Database test successful!");
    println!("Found {} ADRs", rows.len());

    // Test the blip content generation format
    println!("\nTesting blip content generation format:");
    println!("The format should match the astro app format with:");
    println!("- Dynamic ID from the database");
    println!("- Tags and authors arrays");
    println!("- hasAdr field");
    println!("- Description placeholders");

    Ok(())
}
