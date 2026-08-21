#!/usr/bin/env bash
# Create GitHub issues from docs/superpowers/issues/*.md
# Requires: gh auth with permission to create issues.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ISSUES_DIR="$ROOT/docs/superpowers/issues"

if ! gh auth status >/dev/null 2>&1; then
  echo "error: gh is not authenticated" >&2
  exit 1
fi

# Ensure labels exist
for label in mvp; do
  gh label create "$label" --color "0E8A16" --description "Sweeper MVP" 2>/dev/null || true
done

created=()

for file in "$ISSUES_DIR"/[0-9][0-9]-*.md; do
  title="$(awk '/^title:/{sub(/^title:[[:space:]]*/,""); gsub(/^"/,""); gsub(/"$/,""); print; exit}' "$file")"
  labels_raw="$(awk '/^labels:/{sub(/^labels:[[:space:]]*/,""); print; exit}' "$file")"
  # body = file without YAML frontmatter
  body="$(awk '
    BEGIN { in_fm=0; done_fm=0 }
    /^---$/ && !done_fm { if (!in_fm) { in_fm=1; next } else { in_fm=0; done_fm=1; next } }
    !in_fm && done_fm { print }
  ' "$file")"

  # Parse labels like [enhancement, mvp]
  labels_args=()
  labels_csv="$(echo "$labels_raw" | tr -d '[] ')"
  IFS=',' read -ra LABEL_ARR <<< "$labels_csv"
  for l in "${LABEL_ARR[@]}"; do
    [[ -n "$l" ]] && labels_args+=(--label "$l")
  done

  echo "Creating: $title"
  url="$(gh issue create --title "$title" --body "$body" "${labels_args[@]}")"
  echo "  -> $url"
  created+=("$url")
done

echo
echo "Created ${#created[@]} issues:"
printf '  %s\n' "${created[@]}"
