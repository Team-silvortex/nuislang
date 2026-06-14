#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "check-0.18-mainline.sh is kept as a compatibility wrapper."
echo "Forwarding to check-0.19-mainline.sh."

exec bash "$ROOT_DIR/scripts/check-0.19-mainline.sh"
