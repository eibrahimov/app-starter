//! Black-box API tests: build the router against an in-memory SQLite
//! database and drive it with tower's `oneshot`.

use app_starter::{AppState, api};
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use sqlx::sqlite::SqlitePoolOptions;
use tower::util::ServiceExt;

async fn test_app() -> Router {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("connect in-memory sqlite");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("run migrations");
    api::router(AppState { pool })
}

async fn body_json(response: axum::response::Response) -> serde_json::Value {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn health_returns_ok() {
    let app = test_app().await;
    let response = app
        .oneshot(Request::get("/api/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn items_crud_roundtrip() {
    let app = test_app().await;

    // Create
    let response = app
        .clone()
        .oneshot(
            Request::post("/api/items")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"title":"first item"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let created = body_json(response).await;
    let id = created["id"].as_str().unwrap().to_owned();
    assert_eq!(created["done"], false);

    // List
    let response = app
        .clone()
        .oneshot(Request::get("/api/items").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let list = body_json(response).await;
    assert_eq!(list.as_array().unwrap().len(), 1);

    // Toggle
    let response = app
        .clone()
        .oneshot(
            Request::post(format!("/api/items/{id}/toggle"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let toggled = body_json(response).await;
    assert_eq!(toggled["done"], true);

    // Delete
    let response = app
        .clone()
        .oneshot(
            Request::delete(format!("/api/items/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Empty title rejected
    let response = app
        .clone()
        .oneshot(
            Request::post("/api/items")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"title":"   "}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Unknown id is a 404
    let response = app
        .oneshot(
            Request::delete("/api/items/does-not-exist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
