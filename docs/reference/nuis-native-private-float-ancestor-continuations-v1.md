# Native Private Float Ancestor Continuations V1

This is the frozen 307-test float/bool checkpoint. The subsequent
[float/i64 proof](nuis-native-private-float-i64-continuations-v1.md) also admits
exact f32/f64 outward owners beside a private i64 child.

The persistent application-session CPU path carries one exact `f32` or `f64`
prefix-owned ancestor beside one private `bool` child through bounded terminal
mixed selections and sequential continuations. This extends the
[i64 checkpoint](nuis-native-private-i64-ancestor-continuations-v1.md)
using typed owned fields, not numeric word conversion or scalar packing.

The surrounding source helper stays **bool-returning**. The internal selected
region exports one typed owner to its enclosing return prefix. The private child
and transport record are not public results or a new public tuple ABI.

## Single Outward Target

Exact unqualified bool, i32, i64, f32 and f64 owners are eligible. Numeric owner
eligibility still requires either a prefix-owned definition or the exact
delegated target. Delegation is one target's permission, not a source definition;
other entering values remain readonly. Earlier RHSs see entering versions;
later RHSs see proven current versions, and retained arms keep the current owner.

A numeric owner admits only one private exact bool child, already defined in
the selected prefix and absent from the entering scope. Numeric/numeric pairs
are not admitted. Third writes, other entering writes, seals, qualified or
resource fields, type mismatches, hidden ordinary-leaf logical work, child leaks
and partial exits remain atomic vetoes. Both source write orders are retained.

## Exact Private Transport

The existing compiler-private owned result has actual semantic fields:

```text
owner: f32 or f64
child: bool
```

Inactive helpers return the width-specific positive zero and false. Both typed
projections read the same selected record. No borrowed sign/payload bit, numeric
word cast, narrowing, replay, new division or fallible transport is introduced.
The observed bridge adds no owned aggregate allocation/drop calls.

Existing bool-owner numeric-child records, bool/bool and the two i32 codec
orientations retain their established layouts. Common effect and literal-print
authority is not widened.

Whole-forest preflight and original budgets stay ahead of installation:
64 original nodes, 256 expressions, 32 logical edges, 16 statements per local
body/suffix, depth eight, and 31 captures plus a gate. Both widths check the
seven nodes / seven expressions / zero logical edges boundary, adjacent
rejection, 16/17 statements, 31/32 captures and depth 7/8.

## Source And Probes

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_float_ancestor_private_bool.ns)
is specialized by exact scalar width, not by backend or a production test table.
It uses identity initialization, sign negation, IEEE comparisons, both write
orders, nested selections, a shared continuation, seals, unused effectful work,
a bool return and caller continuation. Unselected proof arguments are zero;
speculative evaluation would trap.

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_float_ancestor_oracle.rs)
replays IEEE comparisons and sign-bit toggles, without compiler carrier logic.
At each width, twelve raw words cross both gates: positive and negative zero,
smallest and largest subnormals of both signs, finite values, both infinities,
and quiet/signaling NaN payloads. The native harness observes the actual compiled
answer callee argument, not an oracle-injected result.

The shared harness retains real source work, helpers and traps, forwards actual
callback state to the successor, and checks output canaries. Decimal CLI and
reference cases use four exactly representable values at both widths; NaN
payload acceptance belongs to raw-word native probes, not decimal parsing.

## Executed Acceptance

The 2026-10-11 serial scoped cohort contains **307 distinct tests**:
241 compiler unit/artifact tests, 32 native bridge tests, four CLI tests,
three host failure-policy tests and 27 tensor tests. Fifteen units are new.
Repeated/repaired executions and the complete 292-test predecessor receipt
are not added to the current count.

The new raw native matrix covers 96 input cases across the two widths and
runs 192 successful native processes and 36 fatal native processes across
normal/reversed inventories. It proves 648 accepted callbacks, exact actual
callee bits, source effect order and output canaries. Nine original argument
proof stages per width trap before callee entry; an answer-argument failure
also precedes its callee observation.

The new CLI matrix checks eight images, four per width: fresh build, cache hit
and two consecutive source-free restores. It proves 72 CLI event traps and
744 accepted callbacks, including healthy reentry after each failure. Only the
last accepted open/event state is published. Later event/close requests and
completion are absent after a fatal event. Executable, native objects,
packaging policy, envelope and lifecycle identity are unchanged. Each image
and its input restoration container remain byte-identical after use;
fresh/cache diagnostic-bearing container bytes need not match.

Existing i32/i64-ancestor, five-child-type event failure/budget and helper-entry
regressions remain included. Reference fault cleanup does not make native
process traps recoverable.

The rebuilt ordinary CLI proves 2532 clean registered drift checks / 17593
required patterns, clean coverage/hierarchy/task-card lineage, six source-equal
fields and all six complete predecessor hashes. The selected cell remains
`active/99`; this bounded proof is not broad application-session closure.

Reproduce from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_i32_ancestor_private_bool return_i64_ancestor_private_bool private_event_budget private_event_failure helper_entries_share_exact_budget helper_entries_reject_before_body helper_entries_skip_unselected_paths helper_entries_use_unsigned_limits helper_entries_count_loop_free_roots helper_entries_include_scoped_iterations return_float_ancestor_private_bool --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_i32_ancestor_private_bool return_i64_ancestor_private_bool native_effectful_selected_calls_build_cache_restore_and_execute_only_selected_arguments return_float_ancestor_private_bool --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
```

The tensor preserves the entire predecessor fields:

| Field | Complete 292-test predecessor SHA-256 |
| --- | --- |
| `evidence` | `0d85f836dabc5282eacf5f762c6adccba683afaa16ae4e29fbc706fb7a3e23ed` |
| `next_step` | `d7079f97d9b9e8fb295356bdd78449aa4bb37e5c052424ff07a39a9cacdeeeaf` |
| `blocker` | `2dc0d31d25aa3ceb20e4236f4b16483297f19be644118b9c2a47992b0787537a` |
| `next_action` | `8dbc41112d0a366bb99965cfbc472c0b36c4460943867ceb0a2d5d8e27a0529e` |
| `validation_command` | `7b836bc91b707a715e524a312a120d2357632fbabc225f7797d67ab711313506` |
| `expected_artifact` | `8703d24f8c8396f6a1f1cff01388d5416217f4447ab2442b6251eb61cc88198e` |

## Remaining Boundary

The next selected proof is one exact numeric outward ancestor beside one exact
numeric private child using independently typed owned fields. Start with i64
and i64, retain one outward target, then prove width/type combinations rather
than extending bool codecs to steal payload bits.

This is a macOS CPU receipt, not new GPU/Linux/Windows execution, performance,
formal memory safety, self-hosting, full-workspace or strict-Clippy acceptance.
Broader persistent application-session closure and ABI stabilization remain
separate; the selected tensor cell remains active/99.
