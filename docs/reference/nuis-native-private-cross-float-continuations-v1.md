# Native Private Cross-Width Float Continuations V1

This is the frozen 373-test cross-width floating receipt. The later
[i32/i64 proof](nuis-native-private-i32-i64-continuations-v1.md)
separately admits that exact integer orientation without rewriting this checkpoint.

The persistent application-session CPU path carries an exact floating outward
owner beside an independently initialized private floating child of the other
width through bounded terminal mixed selections and sequential continuations.
This extends the [same-width checkpoint](nuis-native-private-same-float-continuations-v1.md)
without numeric conversion, narrowing or a public tuple ABI.

## Single Outward Target

The separate numeric-pair predicate admits exact unqualified f32/f64 and f64/f32.
Previously proved bool/numeric, dual-i64, integer/float and same-width floating
routes retain their authority. Independently typed carrier installation is not
an admission rule. The source helper is still **bool-returning**, and only the
owner reaches its enclosing return prefix.

One delegated/prefix-owned outward target, readonly entering values, unsealed
prefix-defined private children, both write orders, current/retained versions
and lexical child scope remain mandatory. Third/unrelated entering writes,
seals, qualifier/resource fields, hidden ordinary logical work, child leaks
and partial exits reject the complete candidate atomically. i32 numeric pairs
remain outside this proof, not silently packed or widened.

## Exact Private Transport

The closed compiler-private record has exact independent original types:

```text
owner: f32 / child: f64
owner: f64 / child: f32
```

Inactive helpers return separately typed positive zeros. Both typed projections
read the same selected record. No sign/payload bit is borrowed, numeric
conversion/packing, source replay, new owned allocation/drop, public result
or common effect/backend permission is introduced.

The original whole-forest limits remain 64 nodes, 256 expressions, 32 logical
edges, 16 statements per body/suffix, depth eight and 31 captures plus a gate.
Both orientations prove the seven-node/seven-expression/zero-logical-edge
boundary, adjacent rejection, 16/17 statements, 31/32 captures and depth 7/8.

## Source And Probes

[The Nuis source](../../tools/nuisc/tests/native_application_bridge/return_cross_float_private_pair.ns)
has a f32 owner seed and a separate f64 child seed. The reverse test instantiation
swaps type roles, never input values or floating representations. Each seed
retains its original kind through open/event/close and fourteen-word state.

Current child signs control subsequent owner reads; both write orders, nested
selections, a shared continuation, seals, unused work and caller continuation
remain real source computations. No result can be validated by final bool
equality alone.

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_cross_float_oracle.rs)
uses the owner and child's separate width-specific sign masks and raw IEEE input
inventories. Twelve words per field cross both gates: 576 cases per orientation.
Both fields independently cover signed zeros, smallest/largest subnormals,
finite values, infinities and quiet/signaling NaN payloads.
Actual pair snapshots may differ from the final owner.

Native-only entry probes bitcast each actual after-work argument at its own
original width and the actual final answer-work owner. They replace neither
source computations nor stored state with oracle-manufactured values.
Normal/reversed function inventories retain real calls, joins and traps.
The existing width-driven shared driver forwards actual accepted callback
outputs through volatile loads/stores and checks both output canaries.

Late native faults carry actual floating child word 96, obtained by negating
a negative subnormal of the child's own width. Failure boundaries use original
event positions, not searches through observed field words. Bad arguments
trap before callee entry; neither its observations nor later work may appear.

Existing scalar lowering can emit auxiliary f32 widening followed by integer
last-value bookkeeping. This receipt does not claim a conversion-free entire
LLVM unit. The AOT check requires each widening result to feed only one integer
conversion whose result is unused, and rejects floating narrowing. Synthetic
live floating/integer returns reject the same checker. Original field/result
data remain independently typed; raw native callee probes establish their exact
bits. Removing legacy bookkeeping is separate optimization work.

Reference and CLI acceptance uses four asymmetric finite/signed-zero seed pairs
crossed with both gates. Each field is encoded and displayed at its own width;
full raw IEEE payload evidence belongs to native probes, not decimal parsing.

## Executed Acceptance

The 2026-10-12 serial scoped cohort has **373 distinct tests**:
282 compiler unit/artifact tests, 60 native bridge tests, one CLI test spanning
both orientations, three host failure-policy tests and 27 tensor tests.
Sixteen tests are new. Repaired/repeated runs and the complete 358-test predecessor
are not added to the current count. The new cross-width CLI matrix replaces,
rather than counts as rerunning, the predecessor's same-width CLI receipt.

The new matrix runs 2304 successful native processes and
36 fatal native processes across both orientations and normal/reversed inventories.
It proves 6984 accepted callbacks, independent original-width actual pair words,
final owners, current reads, source order and width-driven canaries.
Nine original proof stages per orientation trap before callee entry, including
marker-valued late faults. Reference cleanup does not make native process traps
recoverable.

Eight CLI images, four per orientation, cover fresh build, cache reuse and two
successive source-free restores. They prove 72 CLI event traps and 744 accepted
callbacks, including healthy reentry after each failure. Only the last accepted
open/event state is published. Later event/close requests and completion remain
absent after fatal events. Executable, native objects, packaging, envelope and
lifecycle identity stay unchanged. Each image and restoration input container
remain byte-identical after use; diagnostic-bearing fresh/cache container bytes
need not match.

The prior i32/i64/float-ancestor, dual-i64, both integer/float orientations,
same-width float, five-child-type event failure/budget and helper-entry native
regressions remain included. The rebuilt ordinary CLI reports 2635 clean drift
checks / 18362 required patterns and clean coverage/hierarchy/task-card lineage.
It checks six source-equal fields and six complete predecessor hashes.
The selected cell stays `active/99`, not broad application-session closure.

Reproduce from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_i32_ancestor_private_bool return_i64_ancestor_private_bool private_event_budget private_event_failure helper_entries_share_exact_budget helper_entries_reject_before_body helper_entries_skip_unselected_paths helper_entries_use_unsigned_limits helper_entries_count_loop_free_roots helper_entries_include_scoped_iterations return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_cross_float_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
```

The tensor retains all six complete 358-test predecessor fields after their
same-float history markers:

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `c0946c68e66e3d13a951219244c6c7ad7530fa8ea385cb376643f90871f03995` |
| `next_step` | `378a0408a76de7be6c14b9e9b5e07e4370a8a5ddc69b028110f9b6c4632cc0e2` |
| `blocker` | `477f1234e2839963a058bbec7f49a32d1b54b74b525a6b2aa0742444a9fa71a2` |
| `next_action` | `3e226ab8fa91223c009a07321f7a040d9cf6bd826c5a57324d4c434c931c2c51` |
| `validation_command` | `555a44f8ed99b81fe792d3771b1bb7b8fd1072a037fcc057c99b7cfbb4c57b16` |
| `expected_artifact` | `727167121f77c84dea8fc1c279bffa0e49fd783b5249cfc87989ba778d6fa3f4` |

## Remaining Boundary

The next independent proof is an exact i32 outward owner beside a private i64
child through these bounded continuations. It needs an independently typed i32
neutral/projection path and original-width extrema/current-value evidence;
existing bool/i32 packing does not grant numeric-pair authority. Keep one
outward target, readonly entering values, private scope/seals and original
budgets. Other i32 orientations, resources, general exits and public ABI
stabilization remain separate.

This is a macOS CPU receipt, not new GPU/Linux/Windows execution, measured
performance, formal memory safety, self-hosting, full-workspace or strict-Clippy
acceptance. Broader application-session closure remains open.
