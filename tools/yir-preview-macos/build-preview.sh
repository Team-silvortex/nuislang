#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
OUT_DIR="${SCRIPT_DIR}/build"
mkdir -p "${OUT_DIR}"

xcrun swiftc \
  -framework AppKit \
  "${SCRIPT_DIR}/PreviewFrame.swift" \
  -o "${OUT_DIR}/PreviewFrame"

echo "${OUT_DIR}/PreviewFrame"
