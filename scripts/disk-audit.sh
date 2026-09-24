#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -P "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [ "$#" -ne 0 ]; then
  echo 'Usage: scripts/disk-audit.sh (read-only, current workspace only)' >&2
  exit 2
fi
echo '== Workspace filesystem =='
df -h "$ROOT_DIR"
echo '== Largest workspace directories =='
du -h -d 2 "$ROOT_DIR" | sort -h | tail -30
echo '== Regeneratable workspace outputs =='
bash "$ROOT_DIR/scripts/disk-clean-safe.sh" --build-binaries
echo 'Use disk-clean-safe.sh --build-binaries --verbose to inspect candidate paths.'
