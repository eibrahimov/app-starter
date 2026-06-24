# Migrations

sqlx migrations, applied in lexicographic order by their `YYYYMMDDHHMMSS_` prefix.

This directory holds **core** migrations only. Each plugin owns its migrations
under `plugins/<name>/migrations/`, run into a separate `_sqlx_migrations_<name>`
keyspace (see `docs/plugin-framework.md` §5). The worked-example resources (`todo`,
`blog`) are plugins, so their tables are created there, not here.

**Append-only. Never edit or rename a committed migration.** sqlx records a
checksum of every applied file; changing one makes the app refuse to start
against any database that already ran it. To evolve the schema, add a NEW file
whose timestamp sorts after the latest. To fix a bad migration, add a corrective
forward migration — there are no down-migrations.

> Note: the one-time removal of the old central `items`/`posts` migrations when
> they moved into plugins is a deliberate **example reset**, not an edit to a
> live-data migration. An existing dev database created before that change needs a
> one-time recreation — see [UPGRADING.md](../UPGRADING.md) ("Adopting the plugin
> framework").

Conventions (see [AGENTS.md](../AGENTS.md) for the full recipe):

- TEXT uuid primary key, generated app-side (not AUTOINCREMENT).
- INTEGER for booleans (0/1), TEXT for timestamps (RFC 3339).
- Add an index on columns you sort or filter by.
