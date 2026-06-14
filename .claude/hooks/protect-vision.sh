#!/usr/bin/env bash
# PreToolUse hook: hard-block agent writes to VISION.md.
#
# VISION.md is human-maintained (AGENTS.md hard rules + .github/CODEOWNERS). The permission
# deny-rules in .claude/settings.json are advisory pattern matches; this hook is the enforced
# backstop. It reads the PreToolUse JSON on stdin, extracts ONLY tool_input.file_path (so a
# mention of "VISION.md" inside another file's content is never blocked), and exits 2 -- which
# tells Claude Code to block the tool call and feed the message below back to the agent --
# when the target's basename is VISION.md.

input="$(cat)"

# Prefer a real JSON parse (python3); fall back to a file_path-only grep when python3 is
# absent. Both read tool_input.file_path, never old_string/new_string content.
fp="$(printf '%s' "$input" | python3 -c 'import sys, json
try:
    print(json.load(sys.stdin).get("tool_input", {}).get("file_path", ""))
except Exception:
    pass' 2>/dev/null)"

if [ -z "$fp" ]; then
  fp="$(printf '%s' "$input" \
    | grep -oE '"file_path"[[:space:]]*:[[:space:]]*"[^"]*"' \
    | head -1 \
    | sed -E 's/.*:[[:space:]]*"([^"]*)"$/\1/')"
fi

# Normalize before matching: lowercase and strip whitespace. The default macOS filesystem
# (APFS) is case-insensitive, so a write to "vision.md" resolves to the same file as
# VISION.md and must be blocked too.
base="$(basename "$fp" 2>/dev/null | tr 'A-Z' 'a-z' | tr -d '[:space:]')"
case "$base" in
  vision.md)
    echo "Blocked: VISION.md is human-maintained and must not be edited by agents. Reference it for intent only; raise any change with a human maintainer (AGENTS.md hard rules, .github/CODEOWNERS)." >&2
    exit 2
    ;;
esac

exit 0
