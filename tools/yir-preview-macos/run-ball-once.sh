#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DEFAULT_OUTPUT_PATH="${TMPDIR:-/tmp}/window_controls_demo_once.ppm"
bash "${SCRIPT_DIR}/run-preview.sh" examples/yir/demos/window_controls_demo.yir "${DEFAULT_OUTPUT_PATH}" 4
