#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
bash "${SCRIPT_DIR}/run-preview.sh" examples/window_controls_demo.yir /tmp/window_controls_demo.ppm 4
