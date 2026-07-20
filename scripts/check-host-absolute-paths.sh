#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

run() {
  echo
  echo "==> $*"
  (cd "$ROOT_DIR" && "$@")
}

run cargo test -p nuisc checked_in_docs_do_not_embed_host_absolute_paths --offline
run echo "host absolute path policy check: ok"
