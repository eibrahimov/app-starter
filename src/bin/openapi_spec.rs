//! Prints the OpenAPI spec as JSON. Used by `just typegen` to generate
//! TypeScript types for the frontend client.
fn main() {
    print!(
        "{}",
        app_starter::api::api_spec()
            .to_pretty_json()
            .expect("serialize OpenAPI spec")
    );
}
