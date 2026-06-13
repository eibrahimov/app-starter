# Migrations

sqlx migrations, applied in lexicographic order by their `YYYYMMDDHHMMSS_` prefix.

**Append-only. Never edit or rename a committed migration.** sqlx records a
checksum of every applied file; changing one makes the app refuse to start
against any database that already ran it. To evolve the schema, add a NEW file
whose timestamp sorts after the latest. To fix a bad migration, add a corrective
forward migration — there are no down-migrations.

Conventions (see [AGENTS.md](../AGENTS.md) for the full recipe):

- TEXT uuid primary key, generated app-side (not AUTOINCREMENT).
- INTEGER for booleans (0/1), TEXT for timestamps (RFC 3339).
- Add an index on columns you sort or filter by.
