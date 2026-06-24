#!/usr/bin/env bash
# Scaffold a new plugin end to end (docs/plugin-framework.md §2, §7 Phase 4 /
# docs/authoring-a-plugin.md). Generates a guard-compliant starting point:
#
#   plugins/<name>/                       backend crate
#     Cargo.toml  plugin.toml  src/lib.rs  migrations/<ts>_create_<name>_items.sql
#   interface/src/plugins/<name>/         frontend (resolves shared deps normally)
#     plugin.tsx  <Type>.tsx
#
# ...and wires the two central edits the author would otherwise make by hand:
#   - appends `<name> = { path = "plugins/<name>" }` to the root Cargo.toml
#   - appends `<name>::register(),` to src/plugins.rs (the generated registry)
#
# After scaffolding, run `just typegen` then `just verify`.
#
# Usage: just new-plugin <name>     (name: ^[a-z][a-z0-9_]*$, e.g. guestbook)
set -euo pipefail

NAME="${1:-}"
if [ -z "${NAME}" ]; then
    echo "usage: just new-plugin <name>   (name matches ^[a-z][a-z0-9_]*$)" >&2
    exit 1
fi
if ! printf '%s' "${NAME}" | grep -qE '^[a-z][a-z0-9_]*$'; then
    echo "ERROR: plugin name '${NAME}' must match ^[a-z][a-z0-9_]*$ (lowercase)." >&2
    exit 1
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PLUGDIR="${ROOT}/plugins/${NAME}"
FEDIR="${ROOT}/interface/src/plugins/${NAME}"

if [ -e "${PLUGDIR}" ] || [ -e "${FEDIR}" ]; then
    echo "ERROR: plugins/${NAME} or interface/src/plugins/${NAME} already exists." >&2
    exit 1
fi

# The name becomes a crate name, a [dependencies] key in the root Cargo.toml, AND
# an extern-crate path root in src/plugins.rs (`<name>::register()`). So it must
# not be a Rust keyword or std crate (would not compile as a crate name / path
# segment), nor collide with a crate the host already depends on (would produce a
# duplicate Cargo key and an ambiguous `<name>::` path). Reject early, with a
# clear message, instead of letting `cargo` fail confusingly after files exist.
RESERVED="as async await break const continue crate dyn else enum extern false fn \
for if impl in let loop match mod move mut pub ref return self static struct super \
trait true type unsafe use where while std core alloc test proc_macro build main"
for word in ${RESERVED}; do
    if [ "${NAME}" = "${word}" ]; then
        echo "ERROR: plugin name '${NAME}' is reserved (a Rust keyword or std crate); it cannot be a crate name." >&2
        exit 1
    fi
done

# Read the current [dependencies] keys rather than hardcoding them, so this stays
# correct as the host's dependency set changes.
EXISTING_DEPS="$(awk '/^\[dependencies\]/{d=1;next} /^\[/{d=0} d && /^[A-Za-z0-9_-]+[[:space:]]*=/{sub(/[[:space:]]*=.*/,"");print}' "${ROOT}/Cargo.toml")"
for dep in ${EXISTING_DEPS}; do
    if [ "${NAME}" = "${dep}" ]; then
        echo "ERROR: plugin name '${NAME}' collides with an existing dependency in the root Cargo.toml." >&2
        exit 1
    fi
done

# PascalCase type name from the snake_case plugin name (guestbook -> Guestbook,
# guest_book -> GuestBook).
TYPE="$(printf '%s' "${NAME}" | awk -F_ '{ out=""; for (i=1;i<=NF;i++) out=out toupper(substr($i,1,1)) substr($i,2); print out }')"
TS="$(date -u +%Y%m%d%H%M%S)"

mkdir -p "${PLUGDIR}/src" "${PLUGDIR}/migrations" "${FEDIR}"

# --- backend crate -------------------------------------------------------

cat > "${PLUGDIR}/Cargo.toml" <<EOF
[package]
name = "${NAME}"
version = "0.1.0"
edition = "2024"
rust-version = "1.94"
license = "MIT"
publish = false

[dependencies]
app-starter-plugin-api = { path = "../../plugin-api" }
axum = "0.8"
sqlx = { version = "0.9", features = ["runtime-tokio", "tls-rustls", "sqlite", "migrate", "chrono"] }
serde = { version = "1", features = ["derive"] }
utoipa = { version = "5", features = ["axum_extras", "chrono"] }
utoipa-axum = "0.2"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
EOF

cat > "${PLUGDIR}/plugin.toml" <<EOF
# Informational manifest. The runtime contract is the Rust \`impl Plugin\` in
# src/lib.rs (name/host_api/api/migrator); this file is NOT parsed at runtime, so
# keep its \`name\`/\`host_api\` in sync with the impl (the impl's values are what
# the host actually enforces).
[plugin]
name = "${NAME}"                        # namespace key: route/schema/table prefixes derive from it
version = "0.1.0"
description = "TODO: describe the ${NAME} plugin"
author = "TODO"
host_api = "^1"                         # semver range; host checks the impl's host_api() at startup
frontend = "interface/src/plugins/${NAME}/plugin.tsx"
EOF

cat > "${PLUGDIR}/migrations/${TS}_create_${NAME}_items.sql" <<EOF
-- Plugin-owned migration (its own _sqlx_migrations_${NAME} keyspace). The table
-- is prefixed with the plugin name (${NAME}_*) per docs/plugin-framework.md §5.
-- Replace these columns with your real schema; keep the ${NAME}_ prefix.
-- Plugins are independent: FK only to core tables, never to another plugin's --
-- there is no cross-plugin migration ordering guarantee (docs/plugin-framework.md §5).
CREATE TABLE ${NAME}_items (
    id TEXT PRIMARY KEY,
    label TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_${NAME}_items_created_at ON ${NAME}_items (created_at);
EOF

cat > "${PLUGDIR}/src/lib.rs" <<EOF
//! The ${NAME} plugin. A minimal CRUD starting point -- replace the domain with
//! yours. Component names are prefixed (${NAME}_*) and routes live under
//! /api/v1/${NAME}, so it satisfies the §6 namespacing guards out of the box.

use app_starter_plugin_api::{AppError, AppState, Plugin};
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use sqlx::migrate::Migrator;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow, ToSchema)]
#[schema(as = ${NAME}_${TYPE})]
pub struct ${TYPE} {
    pub id: String,
    pub label: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[schema(as = ${NAME}_Create${TYPE})]
pub struct Create${TYPE} {
    pub label: String,
}

async fn list(pool: &SqlitePool) -> Result<Vec<${TYPE}>, sqlx::Error> {
    sqlx::query_as::<_, ${TYPE}>("SELECT id, label, created_at FROM ${NAME}_items ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
}

async fn create(pool: &SqlitePool, label: String) -> Result<${TYPE}, sqlx::Error> {
    let row = ${TYPE} {
        id: Uuid::new_v4().to_string(),
        label,
        created_at: Utc::now(),
    };
    sqlx::query("INSERT INTO ${NAME}_items (id, label, created_at) VALUES (?1, ?2, ?3)")
        .bind(&row.id)
        .bind(&row.label)
        .bind(row.created_at)
        .execute(pool)
        .await?;
    Ok(row)
}

async fn remove(pool: &SqlitePool, id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM ${NAME}_items WHERE id = ?1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

#[utoipa::path(
    get,
    path = "/api/v1/${NAME}",
    tag = "${NAME}",
    responses((status = 200, description = "All rows, newest first", body = [${TYPE}]))
)]
async fn list_${NAME}(State(state): State<AppState>) -> Result<Json<Vec<${TYPE}>>, AppError> {
    Ok(Json(list(&state.pool).await?))
}

#[utoipa::path(
    post,
    path = "/api/v1/${NAME}",
    tag = "${NAME}",
    request_body = Create${TYPE},
    responses(
        (status = 201, description = "Created", body = ${TYPE}),
        (status = 400, description = "Empty label")
    )
)]
async fn create_${NAME}(
    State(state): State<AppState>,
    Json(body): Json<Create${TYPE}>,
) -> Result<(StatusCode, Json<${TYPE}>), AppError> {
    let label = body.label.trim().to_owned();
    if label.is_empty() {
        return Err(AppError::BadRequest("label must not be empty".into()));
    }
    Ok((StatusCode::CREATED, Json(create(&state.pool, label).await?)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/${NAME}/{id}",
    tag = "${NAME}",
    params(("id" = String, Path, description = "Row id")),
    responses(
        (status = 204, description = "Deleted"),
        (status = 404, description = "Unknown id")
    )
)]
async fn delete_${NAME}(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, AppError> {
    if !remove(&state.pool, &id).await? {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

struct ${TYPE}Plugin;

impl Plugin for ${TYPE}Plugin {
    fn name(&self) -> &'static str {
        "${NAME}"
    }

    fn host_api(&self) -> &'static str {
        "^1"
    }

    fn api(&self) -> OpenApiRouter<AppState> {
        OpenApiRouter::new()
            .routes(routes!(list_${NAME}, create_${NAME}))
            .routes(routes!(delete_${NAME}))
    }

    fn migrator(&self) -> Option<Migrator> {
        Some(sqlx::migrate!("./migrations"))
    }
}

/// Registration hook the host's generated src/plugins.rs calls.
pub fn register() -> Box<dyn Plugin> {
    Box::new(${TYPE}Plugin)
}
EOF

# --- frontend (under interface/src so it resolves shared deps normally) ---

cat > "${FEDIR}/plugin.tsx" <<EOF
import type { PluginRoute } from "../contract";

export default {
  path: "/${NAME}",
  label: "${TYPE}",
  component: () => import("./${TYPE}").then((module) => module.${TYPE}Page),
} satisfies PluginRoute;
EOF

cat > "${FEDIR}/${TYPE}.tsx" <<EOF
import { Flex, Text } from "@radix-ui/themes";
import { useState } from "react";
import { DataList } from "../../components/sections/DataList";
import { PageHeader } from "../../components/sections/PageHeader";
import { Toolbar } from "../../components/sections/Toolbar";
import { Button } from "../../components/ui/Button";
import { Card } from "../../components/ui/Card";
import { Input } from "../../components/ui/Input";
import { useApiMutation } from "../../hooks/useApiMutation";
import { useApiQuery } from "../../hooks/useApiQuery";

export function ${TYPE}Page() {
  const [label, setLabel] = useState("");

  const rows = useApiQuery("/api/v1/${NAME}", undefined, {
    queryKey: ["${NAME}"],
  });

  const create = useApiMutation("post", "/api/v1/${NAME}", {
    invalidateKeys: [["${NAME}"]],
    onSuccess: () => setLabel(""),
  });
  const remove = useApiMutation("delete", "/api/v1/${NAME}/{id}", {
    invalidateKeys: [["${NAME}"]],
  });

  const submit = () => {
    if (label.trim()) create.mutate({ body: { label } });
  };

  return (
    <Flex direction="column" gap="5">
      <PageHeader title="${TYPE}" />

      <Toolbar>
        <Input
          value={label}
          onChange={(e) => setLabel(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") submit();
          }}
          placeholder="Add a ${NAME}"
          aria-label="New ${NAME} label"
          style={{ flex: 1 }}
        />
        <Button onClick={submit} disabled={create.isPending}>
          Add
        </Button>
      </Toolbar>

      <DataList
        query={rows}
        emptyMessage="Nothing here yet. Add one above."
        errorMessage="Could not load."
        renderItem={(row) => (
          <Card as="li" key={row.id}>
            <Text size="2" style={{ flex: 1 }}>
              {row.label}
            </Text>
            <Button
              variant="danger"
              onClick={() => remove.mutate({ params: { path: { id: row.id } } })}
            >
              Delete
            </Button>
          </Card>
        )}
      />
    </Flex>
  );
}
EOF

# --- central wiring (the only two host edits, done for the author) --------

# 1. Add the crate to the root Cargo.toml [dependencies] (after the plugin-api dep).
awk -v line="${NAME} = { path = \"plugins/${NAME}\" }" '
  { print }
  /^app-starter-plugin-api = / { print line }
' "${ROOT}/Cargo.toml" > "${ROOT}/Cargo.toml.tmp" && mv "${ROOT}/Cargo.toml.tmp" "${ROOT}/Cargo.toml"

# 2. Register it in the generated registry (before the scaffolder marker).
awk -v line="        ${NAME}::register()," '
  /<scaffolder inserts register/ { print line }
  { print }
' "${ROOT}/src/plugins.rs" > "${ROOT}/src/plugins.rs.tmp" && mv "${ROOT}/src/plugins.rs.tmp" "${ROOT}/src/plugins.rs"

echo "Scaffolded plugin '${NAME}' (type ${TYPE}):"
echo "  plugins/${NAME}/                       (backend crate)"
echo "  interface/src/plugins/${NAME}/         (frontend page)"
echo "  + wired into Cargo.toml + src/plugins.rs"
echo
echo "Next: just typegen   # regenerate the TS client for /api/v1/${NAME}"
echo "      just verify    # lint + tests + frontend + cargo-deny"
