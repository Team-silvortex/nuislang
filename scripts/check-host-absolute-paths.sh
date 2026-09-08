#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

run() {
  echo
  echo "==> $*"
  (cd "$ROOT_DIR" && "$@")
}

run cargo test --locked -p nuisc --test examples_mainline_compile \
  checked_in_docs_do_not_embed_host_absolute_paths -- --exact --test-threads=1
run echo "host absolute path policy check: ok"
