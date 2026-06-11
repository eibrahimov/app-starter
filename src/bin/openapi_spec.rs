//! Prints the OpenAPI spec as JSON. Used by `just typegen` to generate
//! TypeScript types for the frontend client.
use utoipa::OpenApi;

fn main() {
    print!(
        "{}",
        app_starter::api::ApiDoc::openapi()
            .to_pretty_json()
            .expect("serialize OpenAPI spec")
    );
}
