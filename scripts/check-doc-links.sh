#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

BAD=0

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

while IFS= read -r src; do
  base_dir="$(dirname "$src")"

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

    if [[ "$target" == docs/* || "$target" == *.md || "$target" == examples/* || "$target" == stdlib/* || "$target" == "."* || "$target" == ".."* ]]; then
      :
    else
      continue
    fi

    if [[ -z "$base_dir" ]]; then
      base_dir="."
    fi
    resolved="$base_dir/$target"
    if [[ ! -e "$resolved" ]]; then
      echo "[docs-link] $src => missing target '$raw_target' (checked '$resolved')"
      BAD=1
    fi
  done < <(perl -nE 'while (/\\[[^\\]]*\\]\\(([^)\\s]+)\\)/g) { say $1 }' "$src")
done < <(find README.md docs -type f -name '*.md' -print | sort)

if [[ $BAD -ne 0 ]]; then
  echo "docs link verification: failed"
  exit 1
fi

echo "docs link verification: ok"
