#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

BAD=0
CHECKED=0

is_external_or_anchor() {
  case "$1" in
    http://*|https://*|mailto:*|tel:*|data:*|javascript:*)
      return 0
      ;;
  esac
  case "$1" in
    \#*)
      return 0
      ;;
  esac
  return 1
}

sources="$(find README.md docs -type f -name '*.md' -print | sort)"
while IFS= read -r src; do
  [[ -z "$src" ]] && continue
  base_dir="$(dirname "$src")"
  # Do not hide extractor failures behind process substitution and a green exit.
  targets="$(perl -nE 'while (/\[[^\]]*\]\(([^)\s]+)\)/g) { say $1 }' "$src")"

  while IFS= read -r raw_target; do
    [[ -z "$raw_target" ]] && continue
    target="${raw_target#<}"
    target="${target%>}"
    target="${target%%#*}"

    [[ -z "$target" ]] && continue
    if is_external_or_anchor "$target"; then
      continue
    fi

    if [[ "$target" == /* ]]; then
      echo "[docs-link] $src => absolute markdown link '$raw_target'"
      BAD=1
      continue
    fi

    if [[ "$target" == *":"* ]]; then
      # Skip other URI-like forms we don't resolve here.
      continue
    fi

    if [[ -z "$base_dir" ]]; then
      base_dir="."
    fi
    resolved="$base_dir/$target"
    CHECKED=$((CHECKED + 1))
    if [[ ! -e "$resolved" ]]; then
      echo "[docs-link] $src => missing target '$raw_target' (checked '$resolved')"
      BAD=1
    fi
  done <<< "$targets"
done <<< "$sources"

if [[ $BAD -ne 0 ]]; then
  echo "docs link verification: failed"
  exit 1
fi

echo "docs link verification: ok ($CHECKED local links checked)"
