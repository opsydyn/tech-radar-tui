use sqlx::FromRow;

/// Represents an ADR record in the database
#[derive(Debug, FromRow)]
#[allow(dead_code)]
pub struct AdrRecord {
    pub id: i32,
    pub title: String,
    pub blip_name: String,
    pub status: String,
    pub timestamp: String,
}

/// Represents a Blip record in the database
#[derive(Debug, FromRow, Clone)]
#[allow(dead_code)]
pub struct BlipRecord {
    pub id: i32,
    pub name: String,
    pub ring: Option<crate::Ring>,
    pub quadrant: Option<crate::Quadrant>,
    pub tag: Option<String>,
    pub description: Option<String>,
    pub created: String,
    #[sqlx(rename = "hasAdr")]
    pub has_adr: bool, // SQLite stores booleans as integers, but we can use bool here
    pub adr_id: Option<i32>,
}

/// Parameters for creating a new ADR
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AdrMetadataParams {
    pub id: i32,
    pub title: String,
    pub blip_name: String,
    pub status: String,
    pub created: String,
}

/// Parameters for creating a new Blip
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BlipMetadataParams {
    pub id: i32,
    pub name: String,
    pub ring: crate::Ring,
    pub quadrant: crate::Quadrant,
    pub tag: String,
    pub description: String,
    pub created: String,
    pub author: String,
    pub has_adr: String,
    pub adr_id: Option<i32>,
}
