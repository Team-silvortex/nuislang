#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

run() {
  echo
  echo "==> $*"
  (cd "$ROOT_DIR" && "$@")
}

run cargo test -q -p nuisc tests_control_flow
run cargo test -q -p nuisc tests_loop_flow
run cargo test -q -p nuisc tests_loop_post_flow
run cargo test -q -p nuisc generic_method_bounds
run cargo test -q -p nuisc --test state_compile
run cargo test -q -p nuisc --test task_compile
run cargo test -q -p nuisc --test memory_compile
run cargo test -q -p nuisc shader_nova_contracts
run cargo test -q -p nuisc --test network_compile

echo
echo "0.18.0 mainline regression matrix: ok"
