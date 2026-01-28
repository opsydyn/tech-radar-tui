use crate::app::state::InputMode;
use crate::config::{get_adrs_dir, get_blips_dir, init_app_config};
use crate::db::models::{AdrMetadataParams, AdrRecord, BlipMetadataParams, BlipRecord};
use crate::db::{
    create_database_pool, get_next_blip_id, get_next_id, insert_new_adr_with_params,
    insert_new_blip,
};
use color_eyre::Result;
use sqlx::SqlitePool;
use std::path::PathBuf;

#[derive(Debug)]
pub struct AppActions {
    pub adrs_dir: PathBuf,
    pub blips_dir: PathBuf,
    pub db_pool: Option<SqlitePool>,
    pub author_name: String,
}

impl AppActions {
    pub fn new() -> Self {
        Self {
            adrs_dir: PathBuf::from("./adrs"),
            blips_dir: PathBuf::from("./blips"),
            db_pool: None,
            author_name: String::new(),
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        let (_, author_name) = init_app_config()?;
        self.author_name = author_name;

        self.adrs_dir = get_adrs_dir();
        self.blips_dir = get_blips_dir();
        self.db_pool = Some(create_database_pool().await?);

        Ok(())
    }

    pub async fn next_id(&self, mode: InputMode) -> Result<i32> {
        let pool = self.pool()?;
        let id = match mode {
            InputMode::Adr => get_next_id(pool).await?,
            InputMode::Blip => get_next_blip_id(pool).await?,
        };

        Ok(id)
    }

    pub async fn insert_adr(&self, params: &AdrMetadataParams) -> Result<()> {
        let pool = self.pool()?;
        insert_new_adr_with_params(pool, params)
            .await
            .map_err(Into::into)
    }

    pub async fn insert_blip(&self, params: &BlipMetadataParams) -> Result<()> {
        let pool = self.pool()?;
        insert_new_blip(pool, params).await.map_err(Into::into)
    }

    pub async fn fetch_blips(&self) -> Result<Vec<BlipRecord>> {
        let pool = self.pool()?;
        crate::db::queries::get_blips(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn fetch_adrs_for_blip(&self, blip_name: &str) -> Result<Vec<AdrRecord>> {
        let pool = self.pool()?;
        let adrs = if blip_name.is_empty() {
            crate::db::queries::get_adrs(pool).await?
        } else {
            crate::db::queries::get_adrs_by_blip_name(pool, blip_name).await?
        };

        Ok(adrs)
    }

    pub async fn update_blip(&self, params: &crate::db::queries::BlipUpdateParams) -> Result<()> {
        let pool = self.pool()?;
        crate::db::queries::update_blip(pool, params)
            .await
            .map_err(Into::into)
    }

    pub async fn blip_exists_by_name(&self, name: &str) -> Result<bool> {
        let pool = self.pool()?;
        crate::db::queries::blip_exists_by_name(pool, name)
            .await
            .map_err(Into::into)
    }

    pub async fn count_blips(&self) -> Result<i64> {
        let pool = self.pool()?;
        crate::db::queries::count_blips(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn count_adrs(&self) -> Result<i64> {
        let pool = self.pool()?;
        crate::db::queries::count_adrs(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn count_blips_by_quadrant(&self) -> Result<Vec<(crate::Quadrant, i64)>> {
        let pool = self.pool()?;
        crate::db::queries::count_blips_by_quadrant(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn count_blips_by_ring(&self) -> Result<Vec<(crate::Ring, i64)>> {
        let pool = self.pool()?;
        crate::db::queries::count_blips_by_ring(pool)
            .await
            .map_err(Into::into)
    }

    pub async fn recent_blips(&self, limit: i64) -> Result<Vec<BlipRecord>> {
        let pool = self.pool()?;
        crate::db::queries::recent_blips(pool, limit)
            .await
            .map_err(Into::into)
    }

    fn pool(&self) -> Result<&SqlitePool> {
        self.db_pool
            .as_ref()
            .ok_or_else(|| color_eyre::eyre::eyre!("Database not initialized"))
    }
}
