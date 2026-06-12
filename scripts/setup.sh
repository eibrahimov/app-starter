#!/usr/bin/env bash
# One-time project rename. Replaces the template's default identity
# (App Starter / app-starter / app_starter) with your project's name in
# every tracked file, then deletes itself. Idempotent: running it twice
# finds nothing left to replace.
set -euo pipefail
cd "$(dirname "$0")/.."

DEFAULT_NAME="App Starter"
DEFAULT_SLUG="app-starter"
DEFAULT_CRATE="app_starter"

NAME="${1:-}"
if [[ -z "$NAME" ]]; then
  read -rp "Project display name (e.g. \"Invoice Ninja\"): " NAME
fi
if [[ -z "$NAME" ]]; then
  echo "error: no name given" >&2
  exit 1
fi

SLUG=$(printf '%s' "$NAME" | tr '[:upper:]' '[:lower:]' | sed -E 's/[^a-z0-9]+/-/g; s/^-+//; s/-+$//')
CRATE=${SLUG//-/_}
if [[ -z "$SLUG" ]]; then
  echo "error: could not derive a slug from \"$NAME\"" >&2
  exit 1
fi

# Escape sed metacharacters in the display name.
NAME_ESC=$(printf '%s' "$NAME" | sed 's/[&\/]/\\&/g')

echo "display name: $NAME"
echo "slug:         $SLUG"
echo "crate:        $CRATE"
echo

FILES=$(grep -rl \
  --exclude-dir=.git \
  --exclude-dir=target \
  --exclude-dir=node_modules \
  --exclude-dir=dist \
  --exclude-dir=binaries \
  --exclude-dir=gen \
  --exclude=setup.sh \
  --exclude='*.png' --exclude='*.ico' --exclude='*.icns' \
  -e "$DEFAULT_CRATE" -e "$DEFAULT_SLUG" -e "$DEFAULT_NAME" . || true)

if [[ -z "$FILES" ]]; then
  echo "Nothing to rename. Already set up?"
  exit 0
fi

for f in $FILES; do
  sed -i.bak \
    -e "s/$DEFAULT_CRATE/$CRATE/g" \
    -e "s/$DEFAULT_SLUG/$SLUG/g" \
    -e "s/$DEFAULT_NAME/$NAME_ESC/g" \
    "$f"
  rm -f -- "$f.bak"
  echo "updated $f"
done

echo
echo "Done. Next steps:"
echo "  1. Review: git diff"
echo "  2. Optional: change the bundle identifier in desktop/src-tauri/tauri.conf.json"
echo "     (currently com.example.$SLUG)"
echo "  3. Commit: git add -A && git commit -m 'rename project to $SLUG'"

rm -- "$0"
echo "(setup.sh removed itself)"
