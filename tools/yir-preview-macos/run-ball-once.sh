#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
bash "${SCRIPT_DIR}/run-preview.sh" examples/yir/host_ui_sphere.yir /tmp/host_ui_sphere.ppm 4
