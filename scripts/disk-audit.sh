#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_ROOT="${TMPDIR:-/tmp}"
HOST_OS="$(uname -s 2>/dev/null || echo Unknown)"

common_cache_path() {
  echo "$HOME/.cargo"
  echo "$HOME/.rustup"
  echo "$HOME/.gradle"
  echo "$HOME/.vscode"
  echo "$HOME/.codex"
}

os_cache_paths() {
  case "$HOST_OS" in
    Darwin)
      echo "$HOME/Library/Caches"
      echo "$HOME/Library/Developer"
      echo "$HOME/Library/Application Support/Code"
      echo "$HOME/Library/Containers/com.docker.docker"
      ;;
  esac
}

echo "== Filesystem =="
df -h /

echo

echo "== Nuis workspace =="
du -xhd 2 "$ROOT_DIR" 2>/dev/null | sort -h | tail -30

echo

echo "== Sibling dev projects =="
if [ -d "$ROOT_DIR/.." ]; then
  du -xhd 2 "$ROOT_DIR/.." 2>/dev/null | sort -h | tail -40
fi

echo

echo "== User developer caches =="
cache_paths=()
while IFS= read -r path; do
  cache_paths+=("$path")
done < <(common_cache_path)
while IFS= read -r path; do
  cache_paths+=("$path")
done < <(os_cache_paths)

for path in "${cache_paths[@]}"; do
  [ -e "$path" ] && du -sh "$path" 2>/dev/null || true
done | sort -h

echo

echo "== Nuis temp artifacts =="
find "$TMP_ROOT" -mindepth 1 -maxdepth 1 \(
  -name 'nuis_*' -o \
  -name 'run_artifact_*' -o \
  -name 'build_report_*' -o \
  -name 'gewyvern-pathology.*' -o \
  -name 'three-module-stack.*' \
\) -print 2>/dev/null | while IFS= read -r item; do
  du -sh "$item" 2>/dev/null || true
done | sort -h | tail -40

echo

echo "== Docker =="
docker system df 2>/dev/null || true
