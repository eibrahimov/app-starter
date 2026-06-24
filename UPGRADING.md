# Upgrading generated projects

After `scripts/setup.sh`, generated projects own their product code, but they
can still pull useful App Starter fixes. Upgrade deliberately: cherry-pick
reusable changes, review conflicts, and never rerun setup.

## Principles

- **Do not rerun `scripts/setup.sh`** (or `scripts/setup.ps1` on Windows) in an initialized project. It is a one-time rename script for fresh clones.
- **Cherry-pick template fixes; do not blindly copy the repo.** Generated apps
  have their own domain, migrations, names, icons, secrets, and deployments.
- **Preserve generated-app identity.** Template references such as `app-starter`,
  `app_starter`, `App Starter`, package names, bundle IDs, and example resources
  may be renamed or removed.
- **Port the pattern, not the example.** If a template change improves `items` or
  `posts`, adapt it to your app's real resources.
- **Run the app's gates after every upgrade.** At minimum, run equivalents of
  `just lint`, `just test`, and `just check-typegen`.

## Track the upstream template

If your generated app does not already have a template remote, add one:

```bash
git remote add template https://github.com/eibrahimov/app-starter.git
git fetch template
```

Inspect recent template changes:

```bash
git log --oneline --decorate template/master
```

If the template default branch changes, update the remote branch in your local commands.

## What is usually safe to cherry-pick

Review every change, but these categories are often portable:

- CI fixes that still match your app's tooling.
- Documentation improvements you want to keep in sync.
- Typegen workflow fixes.
- Build, Docker, or release fixes that do not change your deployment identity.
- Bug fixes in shared infrastructure (`src/db.rs`, `src/error.rs`, `src/frontend.rs`, API wiring patterns).
- Security hardening that matches your app's exposure model.

## What requires extra care

- Migrations: never apply example migrations blindly to an app with real data.
- `interface/src/api/schema.d.ts`: regenerate from your app's backend instead of copying the template file.
- `Cargo.toml`, package names, Tauri bundle identifiers, icons, and Docker image names.
- Example resources: your app may have deleted or replaced `items` and `posts`.
- CORS, auth, request limits, and release publishing defaults.
- Anything that changes database paths, volume names, or persistent data behavior.
- Frontend styling conventions: the UI layer is **Radix Themes**
  (`@radix-ui/themes`) -- there is no Tailwind, `cn` / `twMerge`, or design-token
  CSS. Re-theme from `interface/src/theme/theme.config.ts` (accent, gray, radius,
  scaling), or the `configure-theme` skill, rather than editing components, and
  keep light/dark on the `.dark` class that `ThemeProvider` toggles. The full
  vocabulary is in `docs/radix-reference.md`.

## Adopting the plugin framework

The template's example resources moved from central modules into self-contained
**plugins** (`plugins/todo/`, `plugins/blog/`; see
[docs/authoring-a-plugin.md](docs/authoring-a-plugin.md)). Their migrations now run
in per-plugin keyspaces (`_sqlx_migrations_<name>`), so an **existing database
created before the change fails to start**:

```
Error: migration 20260611000001 was previously applied but is missing in the resolved migrations
```

sqlx sees the old `items`/`posts` rows in the core `_sqlx_migrations` table, which
the now-empty core migrator no longer owns. A fresh database (no `data/app.db`) is
unaffected — this is a one-time recreation for existing dev databases.

The example tables hold only demo data, so recreate the database (back up first if
yours holds anything you care about):

```bash
mv data/app.db data/app.db.bak     # back up the old DB
cargo run                          # creates a fresh DB on the plugin migrations
# or `SEED=1 cargo run` to reseed example rows; `mv` the .bak back to restore
```

If your app already replaced `items`/`posts` with your own resources, this change
does not apply — port the *plugin pattern* to your resources (`just new-plugin`)
rather than copying the example plugins, and migrate any real data deliberately:
the old tables are **not** auto-migrated into the new `<name>_*` plugin tables.

## Cherry-pick workflow

```bash
git fetch template
git cherry-pick <template-commit-sha>
# resolve conflicts carefully
git diff
git status --short
just lint
just test
just check-typegen
```

If the cherry-picked change modifies API annotations or schemas, run
`just typegen` before `just check-typegen` and commit the regenerated schema.

For release, embedded UI, Docker, or desktop changes, also run the relevant build gate:

```bash
just build
just docker-build      # if Docker changed
just desktop-build     # if desktop/Tauri changed
```

## When not to upgrade

Skip or defer a template change when:

- it only improves examples you have removed;
- it changes defaults that conflict with your production posture;
- it requires a migration you cannot safely apply;
- it expands scope without solving a problem your app has;
- it would overwrite app-specific names, branding, icons, or deployment settings.

## Feed improvements back

If an upgrade or generated-app fix reveals something reusable:

- open an adoption-friction issue when the template was unclear;
- open a generated-app backport issue when a real app found a reusable bug or workflow improvement;
- open a feature/pattern proposal before adding new reusable layers or defaults;
- keep domain-specific code out of the template PR.

See `docs/contribution-prompts.md` for copyable issue and PR prompts.
