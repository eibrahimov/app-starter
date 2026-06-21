//! HTTP handlers for the posts example: filtered list with pagination,
//! status transitions with validation, and an aggregate stats endpoint.

use crate::AppState;
use crate::error::AppError;
use crate::posts::{self, CreatePost, Post, PostStats, PostStatus};
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;

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
    path = "/api/v1/posts",
    tag = "posts",
    params(ListPostsQuery),
    responses(
        (status = 200, description = "Posts, newest first", body = [Post]),
        (status = 400, description = "Unknown status value")
    )
)]
pub async fn list_posts(
    State(state): State<AppState>,
    Query(query): Query<ListPostsQuery>,
) -> Result<Json<Vec<Post>>, AppError> {
    let status = match query.status.as_deref() {
        None => None,
        Some(value) => Some(PostStatus::parse(value).ok_or_else(|| {
            AppError::BadRequest("status must be draft, published, or archived".into())
        })?),
    };
    let posts = posts::list(
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
    path = "/api/v1/posts",
    tag = "posts",
    request_body = CreatePost,
    responses(
        (status = 201, description = "Post created as a draft", body = Post),
        (status = 400, description = "Empty title")
    )
)]
pub async fn create_post(
    State(state): State<AppState>,
    Json(body): Json<CreatePost>,
) -> Result<(StatusCode, Json<Post>), AppError> {
    let title = body.title.trim().to_owned();
    if title.is_empty() {
        return Err(AppError::BadRequest("title must not be empty".into()));
    }
    let post = posts::create(&state.pool, title, body.body).await?;
    Ok((StatusCode::CREATED, Json(post)))
}

#[utoipa::path(
    get,
    path = "/api/v1/posts/stats",
    tag = "posts",
    responses((status = 200, description = "Post counts per status", body = PostStats))
)]
pub async fn post_stats(State(state): State<AppState>) -> Result<Json<PostStats>, AppError> {
    Ok(Json(posts::stats(&state.pool).await?))
}

#[utoipa::path(
    get,
    path = "/api/v1/posts/{id}",
    tag = "posts",
    params(("id" = String, Path, description = "Post id")),
    responses(
        (status = 200, description = "The post", body = Post),
        (status = 404, description = "Unknown id")
    )
)]
pub async fn get_post(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Post>, AppError> {
    let post = posts::get(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(post))
}

#[utoipa::path(
    post,
    path = "/api/v1/posts/{id}/publish",
    tag = "posts",
    params(("id" = String, Path, description = "Post id")),
    responses(
        (status = 200, description = "Published post", body = Post),
        (status = 400, description = "Post is not a draft"),
        (status = 404, description = "Unknown id")
    )
)]
pub async fn publish_post(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Post>, AppError> {
    let post = posts::get(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;
    if post.status != PostStatus::Draft {
        return Err(AppError::BadRequest(
            "only draft posts can be published".into(),
        ));
    }
    if !posts::publish(&state.pool, &id).await? {
        return Err(AppError::NotFound);
    }
    let post = posts::get(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(post))
}

#[utoipa::path(
    post,
    path = "/api/v1/posts/{id}/archive",
    tag = "posts",
    params(("id" = String, Path, description = "Post id")),
    responses(
        (status = 200, description = "Archived post", body = Post),
        (status = 400, description = "Post is not published"),
        (status = 404, description = "Unknown id")
    )
)]
pub async fn archive_post(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Post>, AppError> {
    let post = posts::get(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;
    if post.status != PostStatus::Published {
        return Err(AppError::BadRequest(
            "only published posts can be archived".into(),
        ));
    }
    if !posts::archive(&state.pool, &id).await? {
        return Err(AppError::NotFound);
    }
    let post = posts::get(&state.pool, &id)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(post))
}
