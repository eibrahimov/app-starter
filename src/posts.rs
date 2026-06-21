//! Second example domain: a "posts" resource demonstrating patterns beyond
//! `items` — a status lifecycle (draft -> published -> archived), filtered
//! list queries with pagination, and an aggregate stats query.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use utoipa::ToSchema;
use uuid::Uuid;

/// Lifecycle state of a post.
///
/// One closed vocabulary, expressed once: the same lowercase strings are the
/// stored TEXT value (`sqlx::Type`), the on-the-wire JSON (`serde`), and the
/// OpenAPI/TypeScript enum (`utoipa::ToSchema`). Typing `Post.status` as this
/// enum narrows the generated `status` from an open `string` to the closed
/// union `"draft" | "published" | "archived"`. Because the wire values are
/// unchanged, this is an additive contract refinement that stays compatible
/// within `/api/v1`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum PostStatus {
    Draft,
    Published,
    Archived,
}

impl PostStatus {
    /// The stored/wire string for this status. Kept alongside the derived
    /// `serde`/`sqlx` mappings for binding into queries and for `parse`; the
    /// round-trip tests pin all three representations to the same strings.
    pub fn as_str(self) -> &'static str {
        match self {
            PostStatus::Draft => "draft",
            PostStatus::Published => "published",
            PostStatus::Archived => "archived",
        }
    }

    /// Returns `None` for unknown values so handlers can reject them as 400.
    /// The list handler routes `?status=` through this to keep an unknown value
    /// a clear `AppError::BadRequest` rather than a generic deserialization
    /// rejection.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "draft" => Some(PostStatus::Draft),
            "published" => Some(PostStatus::Published),
            "archived" => Some(PostStatus::Archived),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
pub struct Post {
    pub id: String,
    pub title: String,
    pub body: String,
    pub status: PostStatus,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreatePost {
    pub title: String,
    #[serde(default)]
    pub body: String,
}

/// Per-status counts for the stats endpoint.
#[derive(Debug, Default, Serialize, ToSchema)]
pub struct PostStats {
    pub draft: i64,
    pub published: i64,
    pub archived: i64,
}

const SELECT_COLUMNS: &str = "SELECT id, title, body, status, created_at, published_at FROM posts";

pub async fn list(
    pool: &SqlitePool,
    status: Option<PostStatus>,
    limit: i64,
    offset: i64,
) -> Result<Vec<Post>, sqlx::Error> {
    match status {
        Some(status) => {
            sqlx::query_as::<_, Post>(&format!(
                "{SELECT_COLUMNS} WHERE status = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3"
            ))
            .bind(status.as_str())
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
        }
        None => {
            sqlx::query_as::<_, Post>(&format!(
                "{SELECT_COLUMNS} ORDER BY created_at DESC LIMIT ?1 OFFSET ?2"
            ))
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
        }
    }
}

pub async fn get(pool: &SqlitePool, id: &str) -> Result<Option<Post>, sqlx::Error> {
    sqlx::query_as::<_, Post>(&format!("{SELECT_COLUMNS} WHERE id = ?1"))
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn create(pool: &SqlitePool, title: String, body: String) -> Result<Post, sqlx::Error> {
    let post = Post {
        id: Uuid::new_v4().to_string(),
        title,
        body,
        status: PostStatus::Draft,
        created_at: Utc::now(),
        published_at: None,
    };
    sqlx::query(
        "INSERT INTO posts (id, title, body, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .bind(&post.id)
    .bind(&post.title)
    .bind(&post.body)
    // Bind the status through PostStatus's own sqlx encoding, so this insert's
    // (draft) status comes from the enum rather than a duplicated literal. The DB
    // CHECK constraint (migration 20260621000001) enforces the closed set for
    // every write; publish/archive still store in-vocabulary SQL literals.
    .bind(post.status)
    .bind(post.created_at)
    .execute(pool)
    .await?;
    Ok(post)
}

/// Returns true when the post moved draft -> published, false otherwise
/// (missing id or not currently a draft).
pub async fn publish(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE posts SET status = 'published', published_at = ?2 \
         WHERE id = ?1 AND status = 'draft'",
    )
    .bind(id)
    .bind(Utc::now())
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Returns true when the post moved published -> archived, false otherwise
/// (missing id or not currently published).
pub async fn archive(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let result =
        sqlx::query("UPDATE posts SET status = 'archived' WHERE id = ?1 AND status = 'published'")
            .bind(id)
            .execute(pool)
            .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn stats(pool: &SqlitePool) -> Result<PostStats, sqlx::Error> {
    // Decode the grouped status column straight into the enum. The exhaustive
    // match (no `_` arm) makes adding a lifecycle state a compile error here
    // rather than a silently dropped count, and a stored value outside the
    // vocabulary surfaces as a decode error instead of vanishing. `list` and
    // `get` decode `status` the same way, so this fail-loud behavior covers the
    // whole read surface; the DB CHECK constraint (migration 20260621000001)
    // keeps such a row from being written in the first place.
    let rows: Vec<(PostStatus, i64)> =
        sqlx::query_as("SELECT status, COUNT(*) FROM posts GROUP BY status")
            .fetch_all(pool)
            .await?;

    let mut stats = PostStats::default();
    for (status, count) in rows {
        match status {
            PostStatus::Draft => stats.draft = count,
            PostStatus::Published => stats.published = count,
            PostStatus::Archived => stats.archived = count,
        }
    }
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Pure, branchy logic (status parsing) is unit-tested here; the full
    // request/response path is covered by the black-box tests in tests/api.rs.
    // Use this pattern for any non-trivial stateless logic you add.

    #[test]
    fn parse_accepts_known_statuses_and_rejects_everything_else() {
        assert_eq!(PostStatus::parse("draft"), Some(PostStatus::Draft));
        assert_eq!(PostStatus::parse("published"), Some(PostStatus::Published));
        assert_eq!(PostStatus::parse("archived"), Some(PostStatus::Archived));
        assert_eq!(PostStatus::parse("Published"), None);
        assert_eq!(PostStatus::parse(""), None);
        assert_eq!(PostStatus::parse("bogus"), None);
    }

    #[test]
    fn as_str_round_trips_through_parse() {
        for status in [
            PostStatus::Draft,
            PostStatus::Published,
            PostStatus::Archived,
        ] {
            assert_eq!(PostStatus::parse(status.as_str()), Some(status));
        }
    }

    #[test]
    fn serde_round_trips_through_the_wire_string() {
        for status in [
            PostStatus::Draft,
            PostStatus::Published,
            PostStatus::Archived,
        ] {
            let json = serde_json::to_string(&status).expect("serialize");
            // The JSON form is exactly the bound/stored string, so the contract
            // (serde + utoipa) and the database (`as_str`) cannot drift apart.
            assert_eq!(json, format!("\"{}\"", status.as_str()));
            let parsed: PostStatus = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(parsed, status);
        }
    }

    #[test]
    fn deserialize_rejects_unknown_or_miscased_values() {
        assert!(serde_json::from_str::<PostStatus>("\"Published\"").is_err());
        assert!(serde_json::from_str::<PostStatus>("\"bogus\"").is_err());
        assert!(serde_json::from_str::<PostStatus>("\"\"").is_err());
    }
}
