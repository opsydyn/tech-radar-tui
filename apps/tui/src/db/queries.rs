use color_eyre::Result;
use sqlx::{query, query_as, SqlitePool};

use crate::db::models::{AdrRecord, BlipRecord};
use sqlx::query_scalar;

/// Retrieves all ADR records from the database
#[allow(dead_code)]
pub async fn get_adrs(pool: &SqlitePool) -> Result<Vec<AdrRecord>, sqlx::Error> {
    let adrs = query_as::<_, AdrRecord>(
        "SELECT id, title, blip_name, timestamp FROM adr_log ORDER BY id DESC",
    )
    .fetch_all(pool)
    .await?;

    Ok(adrs)
}

/// Retrieves ADR records filtered by blip name
pub async fn get_adrs_by_blip_name(
    pool: &SqlitePool,
    blip_name: &str,
) -> Result<Vec<AdrRecord>, sqlx::Error> {
    let adrs = query_as::<_, AdrRecord>(
        "SELECT id, title, blip_name, timestamp FROM adr_log WHERE blip_name = ? ORDER BY id DESC",
    )
    .bind(blip_name)
    .fetch_all(pool)
    .await?;

    Ok(adrs)
}

pub async fn count_blips(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    query_scalar("SELECT COUNT(*) FROM blip")
        .fetch_one(pool)
        .await
}

pub async fn count_adrs(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    query_scalar("SELECT COUNT(*) FROM adr_log")
        .fetch_one(pool)
        .await
}

pub async fn count_blips_by_quadrant(pool: &SqlitePool) -> Result<Vec<(String, i64)>, sqlx::Error> {
    let rows = query_as::<_, (String, i64)>(
        "SELECT quadrant, COUNT(*) FROM blip WHERE quadrant IS NOT NULL GROUP BY quadrant",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn count_blips_by_ring(pool: &SqlitePool) -> Result<Vec<(String, i64)>, sqlx::Error> {
    let rows = query_as::<_, (String, i64)>(
        "SELECT ring, COUNT(*) FROM blip WHERE ring IS NOT NULL GROUP BY ring",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn recent_blips(pool: &SqlitePool, limit: i64) -> Result<Vec<BlipRecord>, sqlx::Error> {
    let blips = query_as::<_, BlipRecord>(
        "SELECT id, name, ring, quadrant, tag, description, created, \"hasAdr\", adr_id \
         FROM blip ORDER BY created DESC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(blips)
}

/// Retrieves all Blip records from the database
#[allow(dead_code)]
pub async fn get_blips(pool: &SqlitePool) -> Result<Vec<BlipRecord>, sqlx::Error> {
    let blips = query_as::<_, BlipRecord>(
        "SELECT id, name, ring, quadrant, tag, description, created, \"hasAdr\", adr_id 
         FROM blip ORDER BY id DESC",
    )
    .fetch_all(pool)
    .await?;

    Ok(blips)
}

/// Retrieves Blip records filtered by quadrant
#[allow(dead_code)]
pub async fn get_blips_by_quadrant(
    pool: &SqlitePool,
    quadrant: &str,
) -> Result<Vec<BlipRecord>, sqlx::Error> {
    let blips = query_as::<_, BlipRecord>(
        "SELECT id, name, ring, quadrant, tag, description, created, \"hasAdr\", adr_id 
         FROM blip 
         WHERE quadrant = ? 
         ORDER BY ring",
    )
    .bind(quadrant)
    .fetch_all(pool)
    .await?;

    Ok(blips)
}

/// Checks if a blip already exists by name
pub async fn blip_exists_by_name(pool: &SqlitePool, name: &str) -> Result<bool, sqlx::Error> {
    let exists: i64 = query_scalar("SELECT EXISTS(SELECT 1 FROM blip WHERE name = ?)")
        .bind(name)
        .fetch_one(pool)
        .await?;

    Ok(exists != 0)
}

/// Retrieves Blip records filtered by ring
#[allow(dead_code)]
pub async fn get_blips_by_ring(
    pool: &SqlitePool,
    ring: &str,
) -> Result<Vec<BlipRecord>, sqlx::Error> {
    let blips = query_as::<_, BlipRecord>(
        "SELECT id, name, ring, quadrant, tag, description, created, \"hasAdr\", adr_id 
         FROM blip 
         WHERE ring = ? 
         ORDER BY name",
    )
    .bind(ring)
    .fetch_all(pool)
    .await?;

    Ok(blips)
}

/// Retrieves a single Blip record by ID
pub async fn get_blip_by_id(pool: &SqlitePool, id: i32) -> Result<BlipRecord, sqlx::Error> {
    let blip = query_as::<_, BlipRecord>(
        "SELECT id, name, ring, quadrant, tag, description, created, \"hasAdr\", adr_id 
         FROM blip 
         WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(blip)
}


/// Parameters for updating a Blip
#[derive(Debug, Clone)]
pub struct BlipUpdateParams {
    pub id: i32,
    pub name: Option<String>,
    pub ring: Option<String>,
    pub quadrant: Option<String>,
    pub tag: Option<String>,
    pub description: Option<String>,
    pub adr_id: Option<i32>,
}

/// Updates a Blip record in the database with the provided parameters
/// Only fields that are Some will be updated, None fields will keep their current values
pub async fn update_blip(pool: &SqlitePool, params: &BlipUpdateParams) -> Result<(), sqlx::Error> {
    let current = get_blip_by_id(pool, params.id).await?;

    query(
        "UPDATE blip 
         SET name = ?, 
             ring = ?, 
             quadrant = ?, 
             tag = ?, 
             description = ?, 
             adr_id = ?, 
             hasAdr = ? 
         WHERE id = ?",
    )
    .bind(params.name.as_deref().unwrap_or(&current.name))
    .bind(
        params
            .ring
            .as_deref()
            .unwrap_or(&current.ring.unwrap_or_default()),
    )
    .bind(
        params
            .quadrant
            .as_deref()
            .unwrap_or(&current.quadrant.unwrap_or_default()),
    )
    .bind(
        params
            .tag
            .as_deref()
            .unwrap_or(&current.tag.unwrap_or_default()),
    )
    .bind(
        params
            .description
            .as_deref()
            .unwrap_or(&current.description.unwrap_or_default()),
    )
    .bind(params.adr_id.or(current.adr_id))
    .bind(i32::from(params.adr_id.is_some() || current.adr_id.is_some()))
    .bind(params.id)
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;
    use std::env;

    async fn setup_test_db() -> Result<SqlitePool, sqlx::Error> {
        // Use an in-memory database for testing
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect("sqlite::memory:")
            .await?;

        // Set up the schema
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
        .execute(&pool)
        .await?;

        // Insert test data
        query(
            "INSERT INTO blip (id, name, ring, quadrant, tag, description, created, hasAdr, adr_id)
             VALUES (1, 'Test Blip', 'trial', 'tools', 'test', 'A test blip', '2025-04-21', 0, NULL)",
        )
        .execute(&pool)
        .await?;

        Ok(pool)
    }

    #[tokio::test]
    async fn test_get_blip_by_id() -> Result<(), Box<dyn std::error::Error>> {
        let pool = setup_test_db().await?;

        let blip = get_blip_by_id(&pool, 1).await?;
        assert_eq!(blip.id, 1);
        assert_eq!(blip.name, "Test Blip");
        assert_eq!(blip.ring, Some("trial".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_update_blip() -> Result<(), Box<dyn std::error::Error>> {
        let pool = setup_test_db().await?;

        // Update just the name
        let params = BlipUpdateParams {
            id: 1,
            name: Some("Updated Blip".to_string()),
            ring: None,
            quadrant: None,
            tag: None,
            description: None,
        };

        update_blip(&pool, &params).await?;

        // Verify the update
        let updated = get_blip_by_id(&pool, 1).await?;
        assert_eq!(updated.name, "Updated Blip");
        assert_eq!(updated.ring, Some("trial".to_string())); // Should be unchanged

        // Update multiple fields
        let params2 = BlipUpdateParams {
            id: 1,
            name: None, // Keep current name
            ring: Some("adopt".to_string()),
            quadrant: Some("languages".to_string()),
            tag: None,
            description: Some("Updated description".to_string()),
        };

        update_blip(&pool, &params2).await?;

        // Verify the updates
        let updated2 = get_blip_by_id(&pool, 1).await?;
        assert_eq!(updated2.name, "Updated Blip"); // Should be unchanged
        assert_eq!(updated2.ring, Some("adopt".to_string())); // Should be updated
        assert_eq!(updated2.quadrant, Some("languages".to_string())); // Should be updated
        assert_eq!(
            updated2.description,
            Some("Updated description".to_string())
        ); // Should be updated

        Ok(())
    }
}
