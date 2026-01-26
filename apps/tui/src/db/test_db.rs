use crate::db::migrations::{create_database_pool, insert_new_adr_with_params, insert_new_blip};
use crate::db::models::{AdrMetadataParams, BlipMetadataParams};
use chrono::Local;
use color_eyre::Result;

/// Tests the database setup and insertion functions
#[allow(dead_code)]
pub async fn test_database_setup() -> Result<()> {
    // Create database pool
    let pool = create_database_pool().await?;

    // Test inserting an ADR
    let adr_params = AdrMetadataParams {
        id: 1,
        title: "Test ADR".to_string(),
        created: Local::now().to_string(),
    };

    insert_new_adr_with_params(&pool, &adr_params).await?;
    println!("Successfully inserted ADR with ID: {}", adr_params.id);

    // Test inserting a Blip
    let blip_params = BlipMetadataParams {
        id: 1,
        name: "Test Blip".to_string(),
        ring: "Adopt".to_string(),
        quadrant: "Techniques".to_string(),
        tag: "test".to_string(),
        description: "This is a test blip".to_string(),
        created: Local::now().to_string(),
        author: "Test Author".to_string(),
        has_adr: "true".to_string(),
    };

    insert_new_blip(&pool, &blip_params).await?;
    println!("Successfully inserted Blip with ID: {}", blip_params.id);

    Ok(())
}

/// Main function for testing the database functionality
#[tokio::main]
#[allow(dead_code)]
pub async fn main() -> Result<()> {
    // Run the database test
    test_database_setup().await?;

    Ok(())
}
