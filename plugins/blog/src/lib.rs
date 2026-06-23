//! The `blog` plugin -- the second worked-example plugin.
//!
//! A blog-posts resource with a draft -> published -> archived lifecycle,
//! filtered + paginated list queries, and an aggregate stats endpoint -- the
//! `todo` plugin's shape at higher complexity. Routes are under `/api/v1/blog`;
//! the table is `blog_posts` (plugin-name-prefixed). Mirrors docs/plugin-framework.md §3.

use app_starter_plugin_api::{AppError, AppState, Plugin, SeedFuture};
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::migrate::Migrator;
use sqlx::{AssertSqlSafe, SqlitePool};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

// ---- domain -------------------------------------------------------------

/// Lifecycle state of a post. One closed vocabulary expressed once: the same
/// lowercase strings are the stored TEXT (`sqlx::Type`), the wire JSON (`serde`),
/// and the OpenAPI/TypeScript enum (`utoipa::ToSchema`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, sqlx::Type)]
#[schema(as = blog_PostStatus)]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum PostStatus {
    Draft,
    Published,
    Archived,
}

impl PostStatus {
    /// The stored/wire string for this status.
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
#[schema(as = blog_Post)]
pub struct Post {
    pub id: String,
    pub title: String,
    pub body: String,
    pub status: PostStatus,
    pub created_at: DateTime<Utc>,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[schema(as = blog_CreatePost)]
pub struct CreatePost {
    pub title: String,
    #[serde(default)]
    pub body: String,
}

/// Per-status counts for the stats endpoint.
#[derive(Debug, Default, Serialize, ToSchema)]
#[schema(as = blog_PostStats)]
pub struct PostStats {
    pub draft: i64,
    pub published: i64,
    pub archived: i64,
}

const SELECT_COLUMNS: &str =
    "SELECT id, title, body, status, created_at, published_at FROM blog_posts";

// The reads below reuse SELECT_COLUMNS via `format!`, so each query string is
// composed only from this compile-time constant and literal SQL -- nothing
// runtime is interpolated (every value is bound through a `?n` placeholder). sqlx
// 0.9's `SqlSafeStr` guard only accepts `&'static str` automatically, so these
// audited `format!` results are wrapped in `AssertSqlSafe`.

async fn list(
    pool: &SqlitePool,
    status: Option<PostStatus>,
    limit: i64,
    offset: i64,
) -> Result<Vec<Post>, sqlx::Error> {
    match status {
        Some(status) => {
            sqlx::query_as::<_, Post>(AssertSqlSafe(format!(
                "{SELECT_COLUMNS} WHERE status = ?1 ORDER BY created_at DESC LIMIT ?2 OFFSET ?3"
            )))
            .bind(status.as_str())
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
        }
        None => {
            sqlx::query_as::<_, Post>(AssertSqlSafe(format!(
                "{SELECT_COLUMNS} ORDER BY created_at DESC LIMIT ?1 OFFSET ?2"
            )))
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
        }
    }
}

async fn get(pool: &SqlitePool, id: &str) -> Result<Option<Post>, sqlx::Error> {
    sqlx::query_as::<_, Post>(AssertSqlSafe(format!("{SELECT_COLUMNS} WHERE id = ?1")))
        .bind(id)
        .fetch_optional(pool)
        .await
}

async fn create(pool: &SqlitePool, title: String, body: String) -> Result<Post, sqlx::Error> {
    let post = Post {
        id: Uuid::new_v4().to_string(),
        title,
        body,
        status: PostStatus::Draft,
        created_at: Utc::now(),
        published_at: None,
    };
    sqlx::query(
        "INSERT INTO blog_posts (id, title, body, status, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .bind(&post.id)
    .bind(&post.title)
    .bind(&post.body)
    // Bind status through PostStatus's own sqlx encoding (the DB CHECK keeps
    // the closed set for every write).
    .bind(post.status)
    .bind(post.created_at)
    .execute(pool)
    .await?;
    Ok(post)
}

/// Returns true when the post moved draft -> published.
async fn publish(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE blog_posts SET status = ?2, published_at = ?3 \
         WHERE id = ?1 AND status = ?4",
    )
    .bind(id)
    .bind(PostStatus::Published.as_str())
    .bind(Utc::now())
    .bind(PostStatus::Draft.as_str())
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

/// Returns true when the post moved published -> archived.
async fn archive(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("UPDATE blog_posts SET status = ?2 WHERE id = ?1 AND status = ?3")
        .bind(id)
        .bind(PostStatus::Archived.as_str())
        .bind(PostStatus::Published.as_str())
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

async fn stats(pool: &SqlitePool) -> Result<PostStats, sqlx::Error> {
    // Decode the grouped status column straight into the enum: the exhaustive
    // match makes adding a state a compile error here, and a stored value outside
    // the vocabulary surfaces as a decode error instead of vanishing.
    let rows: Vec<(PostStatus, i64)> =
        sqlx::query_as("SELECT status, COUNT(*) FROM blog_posts GROUP BY status")
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

// ---- handlers (/api/v1/blog) -------------------------------------------

#[derive(Debug, Deserialize, utoipa::IntoParams)]
pub struct ListPostsQuery {
    /// Filter by status: "draft", "published", or "archived".
    pub status: Option<String>,
    /// Page size, 1-100. Defaults to 50.
    #[serde(default = "default_limit")]
    pub limit: i64,
    /// Rows to skip, for pagination. Defaults to 0.
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

#[utoipa::path(
    get,
    path = "/api/v1/blog",
    tag = "blog",
    params(ListPostsQuery),
    responses(
        (status = 200, description = "Posts, newest first", body = [Post]),
        (status = 400, description = "Unknown status value")
    )
)]
async fn list_posts(
    State(state): State<AppState>,
    Query(query): Query<ListPostsQuery>,
) -> Result<Json<Vec<Post>>, AppError> {
    let status = match query.status.as_deref() {
        None => None,
        Some(value) => Some(PostStatus::parse(value).ok_or_else(|| {
            AppError::BadRequest("status must be draft, published, or archived".into())
        })?),
    };
    let posts = list(
        &state.pool,
        status,
        query.limit.clamp(1, 100),
        query.offset.max(0),
    )
    .await?;
    Ok(Json(posts))
}

#[utoipa::path(
    post,
    path = "/api/v1/blog",
    tag = "blog",
    request_body = CreatePost,
    responses(
        (status = 201, description = "Post created as a draft", body = Post),
        (status = 400, description = "Empty title")
    )
)]
async fn create_post(
    State(state): State<AppState>,
    Json(body): Json<CreatePost>,
) -> Result<(StatusCode, Json<Post>), AppError> {
    let title = body.title.trim().to_owned();
    if title.is_empty() {
        return Err(AppError::BadRequest("title must not be empty".into()));
    }
    let post = create(&state.pool, title, body.body).await?;
    Ok((StatusCode::CREATED, Json(post)))
}

#[utoipa::path(
    get,
    path = "/api/v1/blog/stats",
    tag = "blog",
    responses((status = 200, description = "Post counts per status", body = PostStats))
)]
async fn post_stats(State(state): State<AppState>) -> Result<Json<PostStats>, AppError> {
    Ok(Json(stats(&state.pool).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/blog/{id}",
    tag = "blog",
    params(("id" = String, Path, description = "Post id")),
    responses(
        (status = 200, description = "The post", body = Post),
        (status = 404, description = "Unknown id")
    )
)]
async fn get_post(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Post>, AppError> {
    let post = get(&state.pool, &id).await?.ok_or(AppError::NotFound)?;
    Ok(Json(post))
}

#[utoipa::path(
    post,
    path = "/api/v1/blog/{id}/publish",
    tag = "blog",
    params(("id" = String, Path, description = "Post id")),
    responses(
        (status = 200, description = "Published post", body = Post),
        (status = 400, description = "Post is not a draft"),
        (status = 404, description = "Unknown id")
    )
)]
async fn publish_post(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Post>, AppError> {
    let post = get(&state.pool, &id).await?.ok_or(AppError::NotFound)?;
    if post.status != PostStatus::Draft {
        return Err(AppError::BadRequest(
            "only draft posts can be published".into(),
        ));
    }
    if !publish(&state.pool, &id).await? {
        return Err(AppError::NotFound);
    }
    let post = get(&state.pool, &id).await?.ok_or(AppError::NotFound)?;
    Ok(Json(post))
}

#[utoipa::path(
    post,
    path = "/api/v1/blog/{id}/archive",
    tag = "blog",
    params(("id" = String, Path, description = "Post id")),
    responses(
        (status = 200, description = "Archived post", body = Post),
        (status = 400, description = "Post is not published"),
        (status = 404, description = "Unknown id")
    )
)]
async fn archive_post(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Post>, AppError> {
    let post = get(&state.pool, &id).await?.ok_or(AppError::NotFound)?;
    if post.status != PostStatus::Published {
        return Err(AppError::BadRequest(
            "only published posts can be archived".into(),
        ));
    }
    if !archive(&state.pool, &id).await? {
        return Err(AppError::NotFound);
    }
    let post = get(&state.pool, &id).await?.ok_or(AppError::NotFound)?;
    Ok(Json(post))
}

// ---- seed ---------------------------------------------------------------

/// Inserts a mix of draft/published/archived posts when the table is empty,
/// driving each row through the real lifecycle transitions.
async fn seed_blog(pool: &SqlitePool) -> anyhow::Result<u64> {
    let existing = stats(pool).await?;
    if existing.draft + existing.published + existing.archived > 0 {
        return Ok(0);
    }

    // (title, body, target status)
    let fixtures = [
        (
            "Welcome to App Starter",
            "This post was created by the optional --seed routine. It is example \
             data -- delete the seed seam to remove it.",
            PostStatus::Published,
        ),
        (
            "How the worked examples fit together",
            "The todo plugin shows minimal CRUD; the blog plugin adds a draft -> \
             published -> archived lifecycle, filtered list queries with \
             pagination, and a stats endpoint.",
            PostStatus::Published,
        ),
        (
            "A work-in-progress draft",
            "Drafts stay unpublished until you publish them. This row demonstrates \
             the draft state and the status filter.",
            PostStatus::Draft,
        ),
        (
            "An archived announcement",
            "Archived posts are retained but kept out of the active flow. This row \
             demonstrates the archived state.",
            PostStatus::Archived,
        ),
    ];

    let mut inserted = 0;
    for (title, body, target) in fixtures {
        let post = create(pool, title.to_owned(), body.to_owned()).await?;
        match target {
            PostStatus::Draft => {}
            PostStatus::Published => {
                publish(pool, &post.id).await?;
            }
            PostStatus::Archived => {
                publish(pool, &post.id).await?;
                archive(pool, &post.id).await?;
            }
        }
        inserted += 1;
    }
    Ok(inserted)
}

// ---- plugin -------------------------------------------------------------

struct BlogPlugin;

impl Plugin for BlogPlugin {
    fn name(&self) -> &'static str {
        "blog"
    }

    fn host_api(&self) -> &'static str {
        "^1"
    }

    fn api(&self) -> OpenApiRouter<AppState> {
        OpenApiRouter::new()
            .routes(routes!(list_posts, create_post))
            .routes(routes!(post_stats))
            .routes(routes!(get_post))
            .routes(routes!(publish_post))
            .routes(routes!(archive_post))
    }

    fn migrator(&self) -> Option<Migrator> {
        Some(sqlx::migrate!("./migrations"))
    }

    fn seed<'a>(&'a self, pool: &'a SqlitePool) -> SeedFuture<'a> {
        Box::pin(async move { seed_blog(pool).await })
    }
}

/// Registration hook the host's generated `src/plugins/mod.rs` calls.
pub fn register() -> Box<dyn Plugin> {
    Box::new(BlogPlugin)
}

#[cfg(test)]
mod tests {
    use super::*;

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
            assert_eq!(json, format!("\"{}\"", status.as_str()));
            let parsed: PostStatus = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(parsed, status);
        }
    }
}
