#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../../.." && pwd)"
DEMO_DIR="$ROOT/examples/projects/tooling/cli_wc_demo"
TMP_DIR="${TMPDIR:-/tmp}/nuis-wc-compare"
RUST_BIN="$TMP_DIR/reference_wc"

mkdir -p "$TMP_DIR"

rustc -O "$DEMO_DIR/reference_wc.rs" -o "$RUST_BIN"
NUIS_CONST_JSON="$(
  cd "$ROOT" &&
  cargo run -q -p nuis -- bench --json --exact "$DEMO_DIR" wc_const_fixture
)"
NUIS_BRIDGE_JSON="$(
  cd "$ROOT" &&
  cargo run -q -p nuis -- bench --json --exact "$DEMO_DIR" wc_bridge_fixture
)"
NUIS_NUMBER_TEXT_JSON="$(
  cd "$ROOT" &&
  cargo run -q -p nuis -- bench --json --exact "$DEMO_DIR" wc_number_text_fixture
)"
NUIS_CONCAT_JSON="$(
  cd "$ROOT" &&
  cargo run -q -p nuis -- bench --json --exact "$DEMO_DIR" wc_concat_fixture
)"
NUIS_SCAN_JSON="$(
  cd "$ROOT" &&
  cargo run -q -p nuis -- bench --json --exact "$DEMO_DIR" wc_scan_fixture
)"

RUST_CONST_JSON="$("$RUST_BIN" const 64)"
RUST_BRIDGE_JSON="$("$RUST_BIN" bridge 64)"
RUST_NUMBER_TEXT_JSON="$("$RUST_BIN" number_text 64)"
RUST_CONCAT_JSON="$("$RUST_BIN" concat 64)"
RUST_SCAN_JSON="$("$RUST_BIN" scan 64)"

python3 - "$NUIS_CONST_JSON" "$NUIS_BRIDGE_JSON" "$NUIS_NUMBER_TEXT_JSON" "$NUIS_CONCAT_JSON" "$NUIS_SCAN_JSON" "$RUST_CONST_JSON" "$RUST_BRIDGE_JSON" "$RUST_NUMBER_TEXT_JSON" "$RUST_CONCAT_JSON" "$RUST_SCAN_JSON" <<'PY'
import json
import sys

def collect(name, nuis_raw, rust_raw):
    nuis = json.loads(nuis_raw)
    rust = json.loads(rust_raw)
    record = nuis["benchmarks"][0]
    if record["status"] not in {"OK", "COMPLETED"}:
        raise SystemExit(f"nuis benchmark did not complete for {name}")
    nuis_avg = record["avg_ns"]
    rust_avg = rust["avg_ns"]
    ratio = None if rust_avg == 0 else nuis_avg / rust_avg
    return {
        "name": name,
        "nuis_avg_ns": nuis_avg,
        "rust_avg_ns": rust_avg,
        "ratio_vs_rust": ratio,
        "nuis_status": record["status"],
        "nuis_run_mode": record["run_mode"],
        "sample_count": record["sample_count"],
    }

profiles = [
    collect("const", sys.argv[1], sys.argv[6]),
    collect("bridge", sys.argv[2], sys.argv[7]),
    collect("number_text", sys.argv[3], sys.argv[8]),
    collect("concat", sys.argv[4], sys.argv[9]),
    collect("scan", sys.argv[5], sys.argv[10]),
]

print(json.dumps({
    "kind": "wc_compare",
    "profiles": profiles,
}, ensure_ascii=False))
PY
