#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

run() {
  echo
  echo "==> $*"
  (cd "$ROOT_DIR" && "$@")
}

run cargo fmt --all --check
run bash scripts/check-0.18-mainline.sh
run cargo test -q -p nuisc multidomain_async
run cargo test -q -p nuisc tests_async_runtime
run cargo test -q -p nuisc tests_async_network_runtime

echo
echo "0.18.0 compiler release gate: ok"
