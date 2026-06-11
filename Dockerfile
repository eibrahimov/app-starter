# ---- Frontend stage ----
FROM oven/bun:1 AS frontend
WORKDIR /app/interface
COPY interface/package.json interface/bun.lock* ./
RUN bun install --frozen-lockfile || bun install
COPY interface/ ./
RUN bun run build

# ---- Backend stage ----
FROM rust:1-bookworm AS backend
WORKDIR /app

# Cache dependencies: build with stub sources first so the dependency
# layer survives source changes.
COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src/bin \
    && echo 'fn main() {}' > src/main.rs \
    && touch src/lib.rs \
    && echo 'fn main() {}' > src/bin/openapi_spec.rs \
    && SKIP_FRONTEND_BUILD=1 cargo build --release \
    && rm -rf src

COPY build.rs ./
COPY migrations/ migrations/
COPY src/ src/
COPY --from=frontend /app/interface/dist interface/dist
RUN touch src/main.rs src/lib.rs \
    && SKIP_FRONTEND_BUILD=1 cargo build --release --bin app-starter

# ---- Runtime stage ----
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=backend /app/target/release/app-starter /usr/local/bin/app-starter

ENV PORT=8080
ENV DATABASE_URL="sqlite:///data/app.db?mode=rwc"
VOLUME /data
EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD curl -fsS http://localhost:8080/api/health || exit 1

CMD ["app-starter"]
