//! Black-box API tests: build the router against an in-memory SQLite
//! database and drive it with tower's `oneshot`.

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use sqlx::sqlite::SqlitePoolOptions;
use tower::util::ServiceExt;

use app_starter::{AppState, api};

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
    // Readiness probe also confirms the database is reachable.
    assert_eq!(json["database"], "ok");
}

/// Guards the typegen loop's biggest footgun: a handler is registered in
/// `paths(...)` but its request/response type is missing from
/// `components(schemas(...))` in src/api/mod.rs. That produces a dangling
/// `$ref` in the served spec, which silently breaks the generated TypeScript.
/// This fails CI before the broken types reach the frontend.
#[tokio::test]
async fn openapi_spec_has_no_dangling_schema_refs() {
    let app = test_app().await;
    let response = app
        .oneshot(
            Request::get("/api/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let spec = body_json(response).await;

    let defined: std::collections::HashSet<String> = spec
        .pointer("/components/schemas")
        .and_then(|s| s.as_object())
        .map(|m| m.keys().cloned().collect())
        .unwrap_or_default();

    let mut referenced = Vec::new();
    collect_schema_refs(&spec, &mut referenced);

    let dangling: Vec<&String> = referenced
        .iter()
        .filter(|r| !defined.contains(*r))
        .collect();
    assert!(
        dangling.is_empty(),
        "OpenAPI references schemas missing from components(schemas(...)) in \
         src/api/mod.rs: {dangling:?}"
    );
}

/// Collects every `#/components/schemas/<Name>` referenced anywhere in the spec.
fn collect_schema_refs(value: &serde_json::Value, out: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, val) in map {
                if key == "$ref"
                    && let Some(name) = val
                        .as_str()
                        .and_then(|s| s.strip_prefix("#/components/schemas/"))
                {
                    out.push(name.to_owned());
                } else {
                    collect_schema_refs(val, out);
                }
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_schema_refs(item, out);
            }
        }
        _ => {}
    }
}

/// POSTs a category and returns its id.
async fn create_category(app: &Router, body: &str) -> String {
    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v1/categories")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_owned()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    body_json(response).await["id"].as_str().unwrap().to_owned()
}

/// POSTs an expense and returns its id.
async fn create_expense(app: &Router, body: &str) -> String {
    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v1/expenses")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_owned()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    body_json(response).await["id"].as_str().unwrap().to_owned()
}

#[tokio::test]
async fn categories_crud_roundtrip() {
    let app = test_app().await;

    let id = create_category(
        &app,
        r##"{"name":"Food","color":"#ef4444","monthly_budget_cents":50000}"##,
    )
    .await;

    // List has the one category
    let response = app
        .clone()
        .oneshot(
            Request::get("/api/v1/categories")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_json(response).await.as_array().unwrap().len(), 1);

    // Update renames and rebudgets
    let response = app
        .clone()
        .oneshot(
            Request::put(format!("/api/v1/categories/{id}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r##"{"name":"Groceries","color":"#10b981","monthly_budget_cents":60000}"##,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let updated = body_json(response).await;
    assert_eq!(updated["name"], "Groceries");
    assert_eq!(updated["monthly_budget_cents"], 60000);

    // A duplicate name is a 400, not a 500 from the UNIQUE constraint
    let response = app
        .clone()
        .oneshot(
            Request::post("/api/v1/categories")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r##"{"name":"Groceries","color":"#000"}"##))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Empty name and negative budget are 400s
    for bad in [
        r##"{"name":"   ","color":"#fff"}"##,
        r##"{"name":"X","color":"#fff","monthly_budget_cents":-1}"##,
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::post("/api/v1/categories")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(bad))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    // Unknown id is a 404
    let response = app
        .clone()
        .oneshot(
            Request::get("/api/v1/categories/nope")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Delete succeeds
    let response = app
        .oneshot(
            Request::delete(format!("/api/v1/categories/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn expenses_crud_roundtrip_with_filters() {
    let app = test_app().await;
    let cat = create_category(&app, r##"{"name":"Food","color":"#ef4444"}"##).await;

    // Create one categorized June expense and one uncategorized May expense
    let id = create_expense(
        &app,
        &format!(
            r#"{{"amount_cents":1299,"description":"Coffee","category_id":"{cat}","spent_on":"2026-06-01"}}"#
        ),
    )
    .await;
    create_expense(
        &app,
        r#"{"amount_cents":4200,"description":"Books","spent_on":"2026-05-15"}"#,
    )
    .await;

    // The categorized expense carries the joined category name
    let response = app
        .clone()
        .oneshot(
            Request::get(format!("/api/v1/expenses/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let got = body_json(response).await;
    assert_eq!(got["amount_cents"], 1299);
    assert_eq!(got["category_name"], "Food");

    // Full list, month filter, and category filter
    for (query, expected) in [
        ("", 2usize),
        ("?month=2026-06", 1),
        (&format!("?category_id={cat}"), 1),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::get(format!("/api/v1/expenses{query}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            body_json(response).await.as_array().unwrap().len(),
            expected,
            "query {query}"
        );
    }

    // Update changes the amount
    let response = app
        .clone()
        .oneshot(
            Request::put(format!("/api/v1/expenses/{id}"))
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"amount_cents":1500,"description":"Coffee","spent_on":"2026-06-01"}"#,
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_json(response).await["amount_cents"], 1500);

    // Invalid inputs: non-positive amount, bad date, bad month filter, unknown category
    let bad_requests = [
        (
            "/api/v1/expenses",
            r#"{"amount_cents":0,"spent_on":"2026-06-01"}"#,
        ),
        (
            "/api/v1/expenses",
            r#"{"amount_cents":100,"spent_on":"06-2026"}"#,
        ),
        (
            "/api/v1/expenses",
            r#"{"amount_cents":100,"spent_on":"2026-06-01","category_id":"ghost"}"#,
        ),
    ];
    for (path, body) in bad_requests {
        let response = app
            .clone()
            .oneshot(
                Request::post(path)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "body {body}");
    }
    let response = app
        .clone()
        .oneshot(
            Request::get("/api/v1/expenses?month=bogus")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    // Unknown id 404, then delete
    let response = app
        .clone()
        .oneshot(
            Request::get("/api/v1/expenses/nope")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let response = app
        .oneshot(
            Request::delete(format!("/api/v1/expenses/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn deleting_a_category_uncategorizes_its_expenses() {
    let app = test_app().await;
    let cat = create_category(&app, r##"{"name":"Travel","color":"#3b82f6"}"##).await;
    let expense = create_expense(
        &app,
        &format!(
            r#"{{"amount_cents":9900,"description":"Train","category_id":"{cat}","spent_on":"2026-06-10"}}"#
        ),
    )
    .await;

    // Delete the category
    let response = app
        .clone()
        .oneshot(
            Request::delete(format!("/api/v1/categories/{cat}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // The expense survives but is now uncategorized
    let response = app
        .oneshot(
            Request::get(format!("/api/v1/expenses/{expense}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let got = body_json(response).await;
    assert!(got["category_id"].is_null());
    assert!(got["category_name"].is_null());
}

#[tokio::test]
async fn summary_aggregates_by_month_and_category() {
    let app = test_app().await;
    let food = create_category(
        &app,
        r##"{"name":"Food","color":"#ef4444","monthly_budget_cents":10000}"##,
    )
    .await;
    let fun = create_category(&app, r##"{"name":"Fun","color":"#a855f7"}"##).await;

    for body in [
        format!(r#"{{"amount_cents":3000,"category_id":"{food}","spent_on":"2026-06-02"}}"#),
        format!(r#"{{"amount_cents":2000,"category_id":"{food}","spent_on":"2026-06-09"}}"#),
        format!(r#"{{"amount_cents":1000,"category_id":"{fun}","spent_on":"2026-06-20"}}"#),
        r#"{"amount_cents":500,"spent_on":"2026-06-25"}"#.to_owned(),
        format!(r#"{{"amount_cents":4000,"category_id":"{food}","spent_on":"2026-05-11"}}"#),
    ] {
        create_expense(&app, &body).await;
    }

    let response = app
        .oneshot(
            Request::get("/api/v1/summary?month=2026-06")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let summary = body_json(response).await;

    // June total = 3000 + 2000 + 1000 + 500
    assert_eq!(summary["total_cents"], 6500);

    let cats = summary["categories"].as_array().unwrap();
    let food_row = cats
        .iter()
        .find(|c| c["name"] == "Food")
        .expect("food row present");
    assert_eq!(food_row["spent_cents"], 5000);
    assert_eq!(food_row["budget_cents"], 10000);
    assert!(
        cats.iter().any(|c| c["name"] == "Uncategorized"),
        "uncategorized bucket present"
    );

    // Trend spans May and June, oldest first
    let months = summary["recent_months"].as_array().unwrap();
    assert_eq!(months.len(), 2);
    assert_eq!(months[0]["month"], "2026-05");
    assert_eq!(months[0]["total_cents"], 4000);
    assert_eq!(months[1]["month"], "2026-06");
    assert_eq!(months[1]["total_cents"], 6500);
}

#[tokio::test]
async fn settings_get_and_update() {
    let app = test_app().await;

    // Seeded default
    let response = app
        .clone()
        .oneshot(
            Request::get("/api/v1/settings")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_json(response).await["base_currency"], "USD");

    // Update normalizes case
    let response = app
        .clone()
        .oneshot(
            Request::put("/api/v1/settings")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"base_currency":"eur"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_json(response).await["base_currency"], "EUR");

    // Invalid code rejected
    let response = app
        .oneshot(
            Request::put("/api/v1/settings")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"base_currency":"toolong"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
