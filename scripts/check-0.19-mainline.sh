#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

run() {
  echo
  echo "==> $*"
  (cd "$ROOT_DIR" && "$@")
}

run python3 scripts/check-text-encoding.py
run cargo test -q -p nuisc tests_control_flow
run cargo test -q -p nuisc tests_loop_flow
run cargo test -q -p nuisc tests_loop_post_flow
run cargo test -q -p nuisc generic_method_bounds
run cargo test -q -p nuisc checked_in_docs_do_not_embed_host_absolute_paths
run bash scripts/check-doc-links.sh
run cargo test -q -p nuisc --test state_compile
run cargo test -q -p nuisc --test task_compile
run cargo test -q -p nuisc --test memory_compile
run cargo test -q -p nuisc shader_nova_contracts
run cargo test -q -p nuisc --test network_compile

echo
echo "0.19.0 mainline regression matrix: ok"
