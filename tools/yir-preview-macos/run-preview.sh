#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  echo "usage: run-preview.sh [module.yir output.ppm [scale]]" >&2
  echo "defaults: examples/nsnova_ball_frame.yir /tmp/nsnova_ball_frame.ppm 12" >&2
  exit 0
fi

if [ "$#" -eq 0 ]; then
  MODULE_PATH="examples/nsnova_ball_frame.yir"
  OUTPUT_PATH="/tmp/nsnova_ball_frame.ppm"
  SCALE="12"
elif [ "$#" -ge 2 ]; then
  MODULE_PATH="$1"
  OUTPUT_PATH="$2"
  SCALE="${3:-16}"
else
  echo "usage: run-preview.sh [module.yir output.ppm [scale]]" >&2
  exit 1
fi

cd "${ROOT_DIR}"
cargo run -p yir-export-frame -- "${MODULE_PATH}" "${OUTPUT_PATH}" "${SCALE}"
BINARY_PATH="$(bash "${SCRIPT_DIR}/build-preview.sh")"
"${BINARY_PATH}" "${OUTPUT_PATH}"
