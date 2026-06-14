#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "check-0.18-release.sh is kept as a compatibility wrapper."
echo "Forwarding to check-0.19-release.sh."

exec bash "$ROOT_DIR/scripts/check-0.19-release.sh"
