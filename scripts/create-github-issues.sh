#!/usr/bin/env bash
# Create GitHub issues from docs/superpowers/issues/*.md
#
# Auth (priority):
#   1. GITHUB_PAT  — personal access token (recommended for Cloud Agents)
#   2. Existing `gh` login
#
# Do NOT put the token in the repo. For Cursor Cloud Agents, add GITHUB_PAT
# as a Runtime Secret at https://cursor.com/dashboard/cloud-agents
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ISSUES_DIR="$ROOT/docs/superpowers/issues"

if [[ -n "${GITHUB_PAT:-}" ]]; then
  export GH_TOKEN="$GITHUB_PAT"
  echo "Using GITHUB_PAT for gh authentication"
elif ! gh auth status >/dev/null 2>&1; then
  echo "error: set GITHUB_PAT or run \`gh auth login\`" >&2
  echo "  Cursor Cloud: add Runtime Secret GITHUB_PAT in the dashboard" >&2
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
