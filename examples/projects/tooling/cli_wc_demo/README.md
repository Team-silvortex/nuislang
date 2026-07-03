# cli_wc_demo

Minimal performance-oriented CLI seed for the tooling lane.

Current scope:

* real project-form CLI path: `argv -> file.read -> buffer -> deserialize_text_from -> stdout`
* benchmark anchors:
  `wc_const_fixture`, `wc_bridge_fixture`, `wc_number_text_fixture`,
  `wc_concat_fixture`, `wc_scan_fixture`
* current metric focus:
  constant baseline, deserialize-text bridge, number-to-text bridge,
  text concat bridge, and the bridge-backed scan seed

This is intentionally still a compact `wc` seed, but it now performs a real
single-buffer ASCII scan for:

* `bytes`
* `text_len`
* `lines`
* `words`
* `scan_ns`

The scan treats space, tab, newline, and carriage return as word separators.
Unicode-aware tokenization is intentionally left to the future text layer rather
than hidden inside this CLI smoke demo.

Quick compare:

```bash
examples/projects/tooling/cli_wc_demo/compare_with_rust.sh
```

That script runs:

* `nuis bench --json --exact ... wc_const_fixture`
* `nuis bench --json --exact ... wc_bridge_fixture`
* `nuis bench --json --exact ... wc_number_text_fixture`
* `nuis bench --json --exact ... wc_concat_fixture`
* `nuis bench --json --exact ... wc_scan_fixture`
* matching tiny Rust reference benchmarks for the same five shapes

The output is one JSON summary with three profiles so we can see where the gap
actually starts:

* `const`: pure harness / call overhead floor
* `bridge`: deserialize-text bridge cost
* `number_text`: integer render + text handle path
* `concat`: text concat + text handle path
* `scan`: bridge plus the current tiny ASCII line/word stats seed

Current caveat:

* `nuis bench` now prefers an in-process monotonic clock emitted by the harness,
  which is much better than the old external process-shell timing
* sample counts are intentionally tiny right now because we are still
  stabilizing the benchmark harness path itself
