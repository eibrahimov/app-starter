//! Application settings: a single row holding the base currency the whole app
//! formats amounts in. Local-first and single-user, so one global row suffices.

use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use utoipa::ToSchema;

/// The fixed primary key of the single settings row.
const SETTINGS_ID: &str = "app";

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct Settings {
    /// ISO 4217-style 3-letter currency code, e.g. "USD", "EUR".
    pub base_currency: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateSettings {
    pub base_currency: String,
}

/// True when `code` is a plausible 3-letter uppercase currency code.
pub fn valid_currency(code: &str) -> bool {
    code.len() == 3 && code.chars().all(|c| c.is_ascii_uppercase())
}

pub async fn get(pool: &SqlitePool) -> Result<Settings, sqlx::Error> {
    sqlx::query_as::<_, Settings>("SELECT base_currency FROM settings WHERE id = ?1")
        .bind(SETTINGS_ID)
        .fetch_one(pool)
        .await
}

pub async fn update(pool: &SqlitePool, base_currency: &str) -> Result<Settings, sqlx::Error> {
    sqlx::query("UPDATE settings SET base_currency = ?2, updated_at = ?3 WHERE id = ?1")
        .bind(SETTINGS_ID)
        .bind(base_currency)
        .bind(chrono::Utc::now())
        .execute(pool)
        .await?;
    get(pool).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_currency_requires_three_uppercase_letters() {
        assert!(valid_currency("USD"));
        assert!(valid_currency("EUR"));
        assert!(!valid_currency("usd"));
        assert!(!valid_currency("US"));
        assert!(!valid_currency("USDD"));
        assert!(!valid_currency("U5D"));
        assert!(!valid_currency(""));
    }
}
