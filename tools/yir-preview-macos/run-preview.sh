#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/../.." && pwd)"

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  echo "usage: run-preview.sh [module.yir output.ppm [scale]]" >&2
  echo "defaults: examples/yir/host_ui_sphere.yir /tmp/host_ui_sphere.ppm 4" >&2
  exit 0
fi

if [ "$#" -eq 0 ]; then
  MODULE_PATH="examples/yir/host_ui_sphere.yir"
  OUTPUT_PATH="/tmp/host_ui_sphere.ppm"
  PLAN_PATH="/tmp/host_ui_sphere.plan"
  SCALE="4"
elif [ "$#" -ge 2 ]; then
  MODULE_PATH="$1"
  OUTPUT_PATH="$2"
  PLAN_PATH="${OUTPUT_PATH%.ppm}.plan"
  SCALE="${3:-4}"
else
  echo "usage: run-preview.sh [module.yir output.ppm [scale]]" >&2
  exit 1
fi

cd "${ROOT_DIR}"
cargo build -p yir-export-ui-plan -p yir-export-frame >/dev/null
"${ROOT_DIR}/target/debug/yir-export-ui-plan" "${MODULE_PATH}" "${PLAN_PATH}"
"${ROOT_DIR}/target/debug/yir-export-frame" "${MODULE_PATH}" "${OUTPUT_PATH}" "${SCALE}"
BINARY_PATH="$(bash "${SCRIPT_DIR}/build-preview.sh")"
EXPORT_BINARY_PATH="${ROOT_DIR}/target/debug/yir-export-frame"
"${BINARY_PATH}" "${PLAN_PATH}" "${MODULE_PATH}" "${OUTPUT_PATH}" "${SCALE}" "${ROOT_DIR}" "${EXPORT_BINARY_PATH}"
