#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_ROOT="${TMPDIR:-/tmp}"
APPLY=0
INCLUDE_DOCKER=0

usage() {
  cat <<'USAGE'
Usage: scripts/disk-clean-safe.sh [--apply] [--docker]

Default mode is dry-run. It reports safe, regeneratable cleanup targets.

Targets:
  - Nuis temporary artifact directories in TMPDIR.
  - Rust target directories for this workspace and known sibling projects.
  - Common development caches that can be regenerated.
  - Optional Docker image/build-cache prune with --docker.

It intentionally does not remove Codex sessions, Docker VM state, app documents,
or user project source trees.
USAGE
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --apply) APPLY=1 ;;
    --docker) INCLUDE_DOCKER=1 ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
  shift
done

run() {
  if [ "$APPLY" -eq 1 ]; then
    echo "+ $*"
    "$@"
  else
    printf '[dry-run] '
    printf '%q ' "$@"
    printf '\n'
  fi
}

remove_path() {
  [ -e "$1" ] || return 0
  run rm -rf "$1"
}

echo "== Before =="
df -h /System/Volumes/Data 2>/dev/null || df -h /

echo
echo "== Nuis temp artifacts =="
find "$TMP_ROOT" -mindepth 1 -maxdepth 1 \( \
  -name 'nuis_*' -o \
  -name 'run_artifact_*' -o \
  -name 'build_report_*' -o \
  -name 'gewyvern-pathology.*' -o \
  -name 'three-module-stack.*' \
\) -print 2>/dev/null | while IFS= read -r item; do
  remove_path "$item"
done

echo
echo "== Rust target directories =="
remove_path "$ROOT_DIR/target"
for sibling in "$ROOT_DIR/../gewyvern" "$ROOT_DIR/../kyuubiki"; do
  remove_path "$sibling/target"
  remove_path "$sibling/workers/rust/target"
done

echo
echo "== Regeneratable caches =="
remove_path "$HOME/Library/Caches/ms-playwright"
remove_path "$HOME/Library/Caches/Homebrew"
remove_path "$HOME/Library/Caches/com.microsoft.VSCode.ShipIt"
remove_path "$HOME/Library/Application Support/Code/CachedExtensionVSIXs"
remove_path "$HOME/Library/Application Support/Code/Cache"
remove_path "$HOME/Library/Application Support/Code/CachedData"
remove_path "$HOME/Library/Application Support/Code/Service Worker/CacheStorage"
remove_path "$HOME/Library/Developer/Xcode/DerivedData"
remove_path "$HOME/Library/Developer/CoreSimulator/Caches"
remove_path "$HOME/.npm/_cacache"
remove_path "$HOME/.gradle/caches"
remove_path "$HOME/.gradle/daemon"
remove_path "$HOME/.gradle/native"

echo
echo "== Docker =="
if [ "$INCLUDE_DOCKER" -eq 1 ]; then
  if [ "$APPLY" -eq 1 ]; then
    docker image prune -a -f || true
    docker builder prune -a -f || true
  else
    echo "[dry-run] docker image prune -a -f"
    echo "[dry-run] docker builder prune -a -f"
  fi
else
  echo "skipped; pass --docker to prune unused images and build cache"
fi

echo
echo "== After =="
df -h /System/Volumes/Data 2>/dev/null || df -h /
