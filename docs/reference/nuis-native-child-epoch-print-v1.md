# Native Child Epoch Print V1

This bounded CPU follow-up continues the [root carry-epoch checkpoint](nuis-native-carry-epoch-print-v1.md)
on Git beta-0.17.8 at 2db5c82c without changing the version or native print policy.

## Fresh Child Entry

The [epoch owner](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch.rs)
now builds a child's print-only certificate from its own changed names that are
already ordered-ready at the original entry. The existing entry classifier then
requires initialized scope, source write authority and exact non-reference i64
type. A parent's print-only certificate is not an input to this operation.

The [child-loop validator](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/literal_loops.rs)
constructs that certificate before resetting changed-name availability for the
new iteration. Source conditions, assignments and temporary initializers still
use the reset availability set. Only the existing single-print validation view
may borrow the certified entry read; it never makes the real set ready.

An unavailable ancestor stays unavailable even if it was just printed in the
parent. A grandchild must independently pass the same original-entry proof; it
cannot inherit readiness from a child's print. Already-ready mutable ancestors
can enter a child certificate only if that child updates them. Unknown/future,
constant, wrong-kind and reference names gain no permission. Merely matching a
name or a parent grant is insufficient.

Complete pure-i64 value admission, exact expression checks and the shared
4,096-work/64-depth print-root budget are unchanged. Pure/buffer profiles and the
legacy literal native policy remain strict. The existing source-order renderer,
i64-v1 producer identity and runtime are reused. No new renderer, ABI, foreign
shim, application case or runtime allocation is introduced.

## Execution Controls

Five new [source controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/child_print_epoch_tests.rs)
check ready/mutable/changed/type intersections, stale type rejection, unavailable
ancestors, ordinary-read leakage and grandchild isolation. All eight combinations
of leading/trailing parent and child induction with selected/skipped checked
print work retain complete old/new value sequences and callee counts. Each
parent iteration initializes a new child-local carry from its current entry.

A raw-NIR control bypasses simplification and executes both zero and positive
child trips with poisoned print arguments. Zero trips perform no child work;
selected positive trips fail at the original print value. NIR verification,
reference execution and LLVM emission apply to that unsimplified module.

## Nested Application Witness

The [Nuis application](../../tools/nuisc/tests/native_application_bridge/child_epoch_print.ns)
prints and updates a parent carry, initializes `local` at child entry, then prints
`observe(local, divisor - index - child)` before each child update and the new
local value afterward. A later parent iteration starts from its newly computed
entry rather than the previous child's stale local. Checked final work precedes
State publication.

The [independent scalar oracle](../../tools/nuisc/tests/native_application_bridge/child_epoch_print_cases.rs)
models both loops directly. Healthy State progresses 64 -> 72 -> -2. Further rows
cover repeated parent entries, zero/negative child trips, skipped poison, negative
deltas, MIN/MAX divisors and a full three-by-three loop case. Its nine failure rows
trap in the first/second/third child print, later parent entries, or final checked
work after zero or multiple loops. Observer markers pin real pure-callee entries
without substituting for any source computation.

[Native/reference tests](../../tools/nuisc/tests/native_application_bridge/child_epoch_print.rs)
check normal/reversed YIR, exact flushed native prefixes, reached call counts,
canary-guarded output, last accepted State and faulted completion. Further events
are rejected after a reference-session failure. This witness uses 12 total loop
work and 64 helper entries, at most 32 functions, and no bridge aggregate heap
allocation/drop. The 12-work choice accommodates three parent plus nine child
iterations; it is an explicit test bound, not a new backend profile.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/child_epoch_print.rs)
checks fresh/cache and two independent source-free images. Each runs healthy
sessions before and after all nine failures. Binary/container, host-object,
envelope, lifecycle and packaging identities remain fixed after removing source,
manifest and project cache. Native probes pin complete flushed failure prefixes;
CLI checks State, trap status and any flushed bytes without assuming that a
trapping process flushes stdout.

## Validation

Bounded local M2 CPU validation passed 328 distinct tests, including eight new
tests. Repeated focused runs are counted once:

- Compiler: 198 focused/adjacent checks in 64.37 seconds. Five new child controls
  cover original entry readiness, exact kinds, missing/immutable names, stale
  types, ordinary-read leakage, ancestor/grandchild isolation, all eight loop
  order/selection combinations and raw-NIR zero-trip poison suppression. Existing
  root print, temporary, carry, word, literal-loop and record controls stay green.
- Native/reference: 49 checks in 162.53 seconds, including two new witnesses,
  previous root/computed-print and record/join families, and six legacy literal
  policy/execution checks. The new witness runs all nine failures under normal
  and reversed YIR order, including later parent and child entries.
- Producer policy: three LLVM build-policy and three packaging-option checks,
  reusing the existing i64-v1 identity without widening legacy v1.
- CLI delivery: one new check in 47.18 seconds. Fresh/cache and two independent
  source-free images each run healthy sessions before and after all nine failures.
- Borrowed-buffer integration: 17 checks in 40.44 seconds; host failure state:
  three; repository maintenance: 24 in 21.256 seconds; host absolute-path policy:
  one; file-line limits: two; status tensor: 27 in 7.09 seconds.

The final CLI-test compilation passed without warnings and the ordinary CLI build
passed in 0.02 seconds after delivery finished. Formatting, whitespace, UTF-8
across 5,780 files, 4,441 local documentation links and locked offline metadata
for 29 packages passed. All 3,780 drift checks and 25,624 required patterns match.

The actual rebuilt CLI matches all six source fields. Complete predecessor
hashes are verified, historical commands remain inert, coverage/hierarchy/lineage
are clean, the next task card is ready and the coordinate remains active/99.

These are targeted local CPU checks, not a full workspace run or a performance
benchmark. Earlier CLI delivery families, the independent/shared storage matrix,
GPU/Linux execution and physical float/performance tests were not rerun here.
Their receipts remain historical evidence rather than new child-epoch results.

## Remaining Frontier

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. Next is a bounded pre-update i64 field projection from an initialized
owned scalar-record carry. Require exact nominal layout, original field path and
current source version without making the whole record generally ready. Keep
those reads rejected until an independent field-level certificate exists.
Reference/resource grants, hidden logical print roots, general effects/I/O/async
capture, outer writes, stable ABI, full GPU/Linux parity and self-hosting remain
separate gates.

The subsequent [field-epoch proof](nuis-native-field-epoch-print-v1.md) advances
that i64 field frontier with an independent path certificate. The validation
counts above remain this child-scalar checkpoint's own results.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- print_epoch:: iteration_prints:: iteration_effects:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- child_epoch_print:: carry_epoch_print:: computed_iteration_print:: literal_print_policy:: literal_print_execution:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- child_epoch_print --test-threads=1 --nocapture
```

The tensor retains the full adjacent validation chain and quoted inert history.

## Predecessor Lineage

All six complete root carry-epoch fields are preserved by SHA-256. Historical
commands remain inert rather than being executed again.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `a3eb241e6915f8c55448922207cb096528fe1aba7bb7f02c4bfcd0bd4a529ad4` |
| `next_step` | `4b5c8ce5172ca504317a472b4d039e90645a7a3dbe2b35026526fc36095765bd` |
| `blocker` | `efb1b2c97fa8761817ee0640f72566536498ff1a814e6b63aadf4af0df9c7734` |
| `next_action` | `7c2a55ec1331d2a5819b584f7b266faf18430f5d445c87ec6582645092b12216` |
| `validation_command` | `f5d891ed289f1661c469b6894b3bc082586c03b5dcd78522e95b0c588ad211d9` |
| `expected_artifact` | `dd9f18e034b1ee918788a2e69b0c7492eb6919ea7cef5ae858f06af36b44726e` |
