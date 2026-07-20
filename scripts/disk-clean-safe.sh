#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_ROOT="${TMPDIR:-/tmp}"
APPLY=0
INCLUDE_DOCKER=0
CLEAN_WORKSPACE=0
CLEAN_CARGO_CACHE=0
VERBOSE=0
HOST_OS="$(uname -s 2>/dev/null || echo Unknown)"

usage() {
  cat <<'USAGE'
Usage: scripts/disk-clean-safe.sh [--apply] [--workspace] [--cargo-cache] [--docker] [--verbose]

Default mode is dry-run. It reports safe, regeneratable cleanup targets.

Targets:
  - Nuis temporary artifacts in TMPDIR.
  - Rust target directories for this workspace and known sibling projects.
  - Optional `cargo clean` for this workspace.
  - Optional cargo cache directories (~/.cargo) that can be rebuilt.
  - Common development caches that can be regenerated.
  - Optional Docker image/build-cache prune with --docker.
  - Add --verbose to print each file delete command.

It intentionally does not touch: source trees, git objects, Codex sessions, Docker
VM state, or unrelated user documents.
USAGE
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --apply) APPLY=1 ;;
    --docker) INCLUDE_DOCKER=1 ;;
    --workspace) CLEAN_WORKSPACE=1 ;;
    --cargo-cache) CLEAN_CARGO_CACHE=1 ;;
    --verbose) VERBOSE=1 ;;
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

echo_cmd() {
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
  if [ "$APPLY" -eq 1 ]; then
    if [ "$VERBOSE" -eq 1 ]; then
      echo_cmd rm -rf "$1"
    fi
    if ! rm -rf "$1"; then
      echo "warning: failed to remove $1"
    fi
  else
    if [ "$VERBOSE" -eq 1 ]; then
      printf '[dry-run] '
      printf '%q ' rm -rf "$1"
      printf '\n'
    fi
  fi
}

scan_size() {
  local path="$1"
  if [ -e "$path" ]; then
    du -sh "$path" 2>/dev/null || true
  fi
}

echo "== Before =="
df -h /

if [ "$CLEAN_WORKSPACE" -eq 1 ]; then
  echo
  echo "== workspace clean =="
  scan_size "$ROOT_DIR/target"
  (cd "$ROOT_DIR" && echo_cmd cargo clean)
else
  echo
  echo "== workspace clean =="
  echo "[dry-run] use --workspace to run: cargo clean"
  scan_size "$ROOT_DIR/target"
fi

echo
echo "== Nuis temp artifacts =="
ARTIFACT_LIST="$(mktemp)"
find "$TMP_ROOT" -mindepth 1 -maxdepth 1 \( \
  -name 'nuis_*' -o \
  -name 'run_artifact_*' -o \
  -name 'build_report_*' -o \
  -name 'gewyvern-pathology.*' -o \
  -name 'three-module-stack.*' \
\) -print 2>/dev/null > "$ARTIFACT_LIST"

ARTIFACT_COUNT=0
if [ -s "$ARTIFACT_LIST" ]; then
  ARTIFACT_COUNT="$(wc -l < "$ARTIFACT_LIST" | tr -d ' ')"
  echo "Found $ARTIFACT_COUNT matching temp artifacts in $TMP_ROOT"
  sed -n '1,80p' "$ARTIFACT_LIST"
  if [ "$ARTIFACT_COUNT" -gt 80 ]; then
    echo "... and $((ARTIFACT_COUNT - 80)) more (not shown)"
  fi
else
  echo "No matching temp artifacts found in $TMP_ROOT"
fi

while IFS= read -r item; do
  remove_path "$item"
done < "$ARTIFACT_LIST"
rm -f "$ARTIFACT_LIST"

echo
echo "== Rust target directories =="
remove_path "$ROOT_DIR/target"
for sibling in "$ROOT_DIR/../gewyvern" "$ROOT_DIR/../kyuubiki"; do
  remove_path "$sibling/target"
  remove_path "$sibling/workers/rust/target"
done

echo
echo "== Cargo cache =="
if [ "$CLEAN_CARGO_CACHE" -eq 1 ]; then
  remove_path "$HOME/.cargo/registry/cache"
  remove_path "$HOME/.cargo/registry/index"
  remove_path "$HOME/.cargo/registry/src"
  remove_path "$HOME/.cargo/git/db"
  remove_path "$HOME/.cargo/git/checkouts"
else
  echo "[dry-run] use --cargo-cache to remove regenerateable ~/.cargo cache directories"
  scan_size "$HOME/.cargo/registry/cache"
  scan_size "$HOME/.cargo/git/db"
fi

echo
echo "== Regeneratable caches =="
case "$HOST_OS" in
  Darwin)
    remove_path "$HOME/Library/Caches/ms-playwright"
    remove_path "$HOME/Library/Caches/Homebrew"
    remove_path "$HOME/Library/Caches/com.microsoft.VSCode.ShipIt"
    remove_path "$HOME/Library/Application Support/Code/CachedExtensionVSIXs"
    remove_path "$HOME/Library/Application Support/Code/Cache"
    remove_path "$HOME/Library/Application Support/Code/CachedData"
    remove_path "$HOME/Library/Application Support/Code/Service Worker/CacheStorage"
    remove_path "$HOME/Library/Developer/Xcode/DerivedData"
    remove_path "$HOME/Library/Developer/CoreSimulator/Caches"
    ;;
  *)
    :
    ;;
esac
remove_path "$HOME/.npm/_cacache"
remove_path "$HOME/.gradle/caches"
remove_path "$HOME/.gradle/daemon"
remove_path "$HOME/.gradle/native"
remove_path "$HOME/.cache/sccache"

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
df -h /
