#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_ROOT="${TMPDIR:-/tmp}"

echo "== Filesystem =="
df -h /System/Volumes/Data 2>/dev/null || df -h /

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
for path in \
  "$HOME/.cargo" \
  "$HOME/.rustup" \
  "$HOME/.gradle" \
  "$HOME/.vscode" \
  "$HOME/.codex" \
  "$HOME/Library/Caches" \
  "$HOME/Library/Developer" \
  "$HOME/Library/Application Support/Code" \
  "$HOME/Library/Containers/com.docker.docker"
do
  [ -e "$path" ] && du -sh "$path" 2>/dev/null || true
done | sort -h

echo
echo "== Nuis temp artifacts =="
find "$TMP_ROOT" -mindepth 1 -maxdepth 1 \( \
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
