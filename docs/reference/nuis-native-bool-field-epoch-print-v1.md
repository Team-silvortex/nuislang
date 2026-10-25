# Native Bool Field Epoch Print V1

This bounded CPU worktree follow-up continues the [i64 field proof](nuis-native-field-epoch-print-v1.md)
on Git beta-0.17.8 at `2db5c82c`. It changes no version, native ABI or i64-v1
print policy. The native result being printed is still exactly i64.

## Exact Argument Position

The [shared field owner](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_field_reads.rs)
retains its original Var-rooted path, exact nominal shape and current entry-type
checks. The [epoch owner](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch.rs)
now requests an exact bool leaf separately from ordinary i64 field reads.
Neither request marks a record or bool binding generally ready.

The [completed signature query](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers.rs)
requires a completed pure helper, an exact i64 result and an exact bool parameter
at the original argument index. The
[print expression checker](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs)
uses this query only in that call-argument position, with the existing recursion
depth bound. Argument arity and every actual type still pass the normal complete
signature check. No provisional or effectful helper can authorize a read.

This is not ambient bool expression permission. Direct bool print, a compound
comparison wrapping the unavailable field, a bool-returning or record-returning
callee, implicit conversion and a whole-record argument remain outside this
grant. Existing ordinary expressions using already-ready inputs are unchanged.
Conditions, assignments, temporary initializers and child headers receive no
print certificate. A parent's print cannot initialize an unavailable ancestor
for a child or grandchild.

The common initialized/writable/changed entry rules, ready child-entry rules,
4,096-work/64-depth print-root budget and source-order renderer are reused.
No backend-specific case, foreign shim, new intrinsic or runtime allocation
is introduced.

## Execution Witnesses

Five new [compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/bool_field_print_epoch_tests.rs)
check exact helper parameter positions, pure completion, wrong result/argument
kinds, stale layouts, reference roots, whole-record laundering and non-export of
readiness. Raw NIR executes both induction orders, old/new true/false values,
skipped poison, zero trips and selected division failures without simplification.

The [Nuis application](../../tools/nuisc/tests/native_application_bridge/bool_field_epoch_print.ns)
keeps nested total/tag/flag records as parent and child carries. Each invocation
starts its flag from the selected input; parent and child updates flip it. Calls
to `observe(flag, value, divisor)` return the value for true and value plus ten
for false, with checked division on both branches. Thus stale bool reads have
observable consequences even when total/tag projections are correct.

The [independent scalar oracle](../../tools/nuisc/tests/native_application_bridge/bool_field_epoch_print_cases.rs)
models both loops and bool transitions directly. Healthy persisted State follows
64 -> 72 -> -2. Repeated entries, zero/negative child trips, skipped poison,
negative deltas, MIN/MAX divisors and the three-by-three case remain covered.
Its nine failure rows stop in reached child prints or final checked work before
State publication. Only the test probe adds real callee-entry observation markers.

[Reference/native checks](../../tools/nuisc/tests/native_application_bridge/bool_field_epoch_print.rs)
cover normal/reversed YIR, complete flushed failure prefixes, reached call counts,
canary outputs and last-State/faulted-session behavior. The fixture uses 12 loop
work, 128 helper entries and at most 32 functions. The explicit entry bound
allows repeated bool observer calls and their selected pure branches; it is not
a new backend admission limit or a promise of unrestricted loops.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/bool_field_epoch_print.rs)
checks fresh/cache and two independent source-free images, each with healthy
runs before and after all failures. Binary/container, host-object, envelope,
lifecycle and packaging identities stay fixed. Direct probes prove full flushed
fault prefixes; CLI traps need only preserve any stdout actually flushed.

## Validation

Bounded local M2 CPU validation passed 347 distinct targeted tests, including
eight new tests. Repeated focused runs are counted once:

- Compiler: 211 focused/adjacent tests in 57.51 seconds, including five new bool
  controls and the existing i64/root/child/record/word and shared shape controls.
- Native/reference: 53 tests in 141.39 seconds, including two new bool witnesses
  and earlier field/root/child/computed-print, record/join and literal families.
- CLI delivery: one new test in 60.44 seconds; fresh/cache and two independent
  source-free images each run healthy sessions before and after all nine failures.
- Producer policy: three LLVM build-policy and three packaging-option tests.
- Borrowed-buffer integration: 17 tests in 38.05 seconds; host failure state:
  three; repository maintenance: 26 in 20.340 seconds; host absolute-path policy:
  one; file-line limits: two; tensor tests: 27 in 6.09 seconds.

The final CLI-test compilation and ordinary CLI build passed without warnings
after delivery finished. Formatting, whitespace, UTF-8 across 5,795 files,
4,310 canonical documentation links and locked offline metadata for 29 packages
passed. All 3,800 drift checks and 25,729 required patterns match.

The actual rebuilt CLI matches all six source fields. Complete predecessor
SHA-256 identities and quoted inert history are verified without executing
historical commands. Coverage, hierarchy and lineage are clean, the next task
card is ready and the selected coordinate remains active/99.

These are bounded CPU checks, not a full workspace run or a performance claim.
Previous CLI delivery families, the independent/shared storage matrix, physical
float/performance tests and GPU/Linux execution were not rerun. Earlier receipts
remain historical evidence, not new bool-field acceptance. Use the active tensor
command, not commands inside its quoted predecessor.

## Remaining Frontier

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99. Next is a
pre-update exact i32 field passed to an exact i32 parameter of a completed pure
i64 print helper. It needs independent typed-argument and native word-transport
controls; this bool proof does not authorize it. Whole-record readiness,
reference/resource reads, general effects/I/O/async capture, outer writes,
stable ABI, GPU/Linux parity and self-hosting remain separate gates.

The subsequent [typed-i32 proof](nuis-native-i32-field-epoch-print-v1.md) shares
the completed parameter-type query and exact argument certificate with bool.
This receipt's test totals remain its own bool checkpoint, not cumulative.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- print_epoch:: iteration_prints:: iteration_effects:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- bool_field_epoch_print:: field_epoch_print:: child_epoch_print:: carry_epoch_print:: computed_iteration_print:: literal_print_policy:: literal_print_execution:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- bool_field_epoch_print --test-threads=1 --nocapture
```

## Predecessor Lineage

Six complete i64 field-epoch fields are retained, including quoted inert commands.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `8cb5a2e4fbeb96246a6c3431af47e1d7fd01f5a09cb55f0322b205397b4c4197` |
| `next_step` | `b5a25819d3690bd98b09b7b3397073bdd3805c73ddb44937fe2c8922291d40f9` |
| `blocker` | `84925954da430352e3057171f0dc72caba565e43756e13492e3161294872cced` |
| `next_action` | `9c8563d6280689ac76d03a6256018a5e926e1d24aed87cb38401a8f502350577` |
| `validation_command` | `3f06c175651727102b603d53d4ace6960a5686986f62dc91e6d2da38ee12981e` |
| `expected_artifact` | `b55b1894ac882b670b9df71275955ff057fe97c86ae14ebe66d1f7637a105f56` |
