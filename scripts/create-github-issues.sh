#!/usr/bin/env bash
# Create GitHub issues from docs/superpowers/issues/*.md
# Requires: gh auth with permission to create issues.
#
# Usage:
#   ./scripts/create-github-issues.sh           # all numbered issues
#   ./scripts/create-github-issues.sh post-mvp  # only 12–20
#   ./scripts/create-github-issues.sh roadmap   # only 21+
#   ./scripts/create-github-issues.sh mvp       # only 01–11
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ISSUES_DIR="$ROOT/docs/superpowers/issues"
MODE="${1:-all}"

if ! gh auth status >/dev/null 2>&1; then
  echo "error: gh is not authenticated" >&2
  exit 1
fi

# Ensure labels exist
gh label create "mvp" --color "0E8A16" --description "Sweeper MVP" 2>/dev/null || true
gh label create "post-mvp" --color "1D76DB" --description "Post-MVP enhancement" 2>/dev/null || true
gh label create "enhancement" --color "a2eeef" --description "New feature or request" 2>/dev/null || true

created=()

shopt -s nullglob
for file in "$ISSUES_DIR"/[0-9][0-9]-*.md; do
  base="$(basename "$file")"
  num="${base%%-*}"
  num=$((10#$num))

  case "$MODE" in
    mvp)
      (( num <= 11 )) || continue
      ;;
    post-mvp)
      (( num >= 12 && num <= 20 )) || continue
      ;;
    roadmap)
      (( num >= 21 )) || continue
      ;;
    all) ;;
    *)
      echo "usage: $0 [all|mvp|post-mvp|roadmap]" >&2
      exit 2
      ;;
  esac

  title="$(awk '/^title:/{sub(/^title:[[:space:]]*/,""); gsub(/^"/,""); gsub(/"$/,""); print; exit}' "$file")"
  labels_raw="$(awk '/^labels:/{sub(/^labels:[[:space:]]*/,""); print; exit}' "$file")"
  body="$(awk '
    BEGIN { in_fm=0; done_fm=0 }
    /^---$/ && !done_fm { if (!in_fm) { in_fm=1; next } else { in_fm=0; done_fm=1; next } }
    !in_fm && done_fm { print }
  ' "$file")"

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
