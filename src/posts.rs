//! Second example domain: a "posts" resource demonstrating patterns beyond
//! `items` — a status lifecycle (draft -> published -> archived), filtered
//! list queries with pagination, and an aggregate stats query.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use utoipa::ToSchema;
use uuid::Uuid;

/// Lifecycle state of a post. Stored as TEXT; parsed from query params.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PostStatus {
    Draft,
    Published,
    Archived,
}

impl PostStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            PostStatus::Draft => "draft",
            PostStatus::Published => "published",
            PostStatus::Archived => "archived",
        }
    }

    /// Returns `None` for unknown values so handlers can reject them as 400.
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
    pub status: String,
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
        status: PostStatus::Draft.as_str().to_owned(),
        created_at: Utc::now(),
        published_at: None,
    };
    sqlx::query(
        "INSERT INTO posts (id, title, body, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .bind(&post.id)
    .bind(&post.title)
    .bind(&post.body)
    .bind(&post.status)
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
    let rows: Vec<(String, i64)> =
        sqlx::query_as("SELECT status, COUNT(*) FROM posts GROUP BY status")
            .fetch_all(pool)
            .await?;

    let mut stats = PostStats::default();
    for (status, count) in rows {
        match status.as_str() {
            "draft" => stats.draft = count,
            "published" => stats.published = count,
            "archived" => stats.archived = count,
            _ => {}
        }
    }
    Ok(stats)
}
