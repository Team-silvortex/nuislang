#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

run() {
  echo
  echo "==> $*"
  (cd "$ROOT_DIR" && "$@")
}

run cargo test -q -p nuisc tests_generics
run cargo test -q -p nuisc tests_higher_order
run cargo test -q -p nuisc tests_generic_constraints
run cargo test -q -p nuisc tests_control_flow
run cargo test -q -p nuisc tests_async_runtime
run cargo test -q -p nuisc --test task_compile
run cargo test -q -p nuisc --test network_compile
run cargo test -q -p nuisc --test state_compile

echo
echo "0.17.0 mainline regression matrix: ok"
