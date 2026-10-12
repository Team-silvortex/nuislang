# Native Private Dual I64 Continuations V1

This is the frozen 315-test i64/i64 checkpoint. The subsequent
[i64/float proof](nuis-native-private-i64-float-continuations-v1.md) also admits
exact f32/f64 private children beside i64, retaining one outward target.

The persistent application-session CPU path carries one exact i64 outward
ancestor beside one exact private i64 child through bounded terminal mixed
selections and sequential continuations. This extends the
[float checkpoint](nuis-native-private-float-ancestor-continuations-v1.md)
using independently typed owned fields, not packing two words into one scalar.

The surrounding source helper stays **bool-returning**. Only the owner is
exported to the enclosing return prefix. Neither the private child nor the
carrier becomes a public tuple result or a new public ABI.

## Single Outward Target

The numeric/numeric admission predicate accepts only exact unqualified i64/i64.
It is separate from the generic two-field installation mechanism. Bool/numeric
and numeric/bool combinations keep their established rules; no broad numeric
cross-product is admitted by the existence of a typed carrier.

Eligibility requires a prefix-owned owner or the exact delegated target.
The child is already defined in the selected prefix, unsealed and absent from
the entering scope. Delegation does not create another source definition or
another outward write permission. Both write orders, retention and current-value
reads are preserved. Other entering writes, a third target, qualifier/resource
fields, hidden ordinary-leaf logical work, child leaks, seals and partial exits
remain atomic vetoes.

## Exact Private Transport

The compiler-private owned result has two full-width semantic fields:

```text
owner: i64
child: i64
```

Inactive helpers return independently typed zero/zero fields. Both projections
read the same actual selected record. No payload bit is borrowed, numeric
narrowing/packing or source replay is introduced. The observed native bridge
adds no owned aggregate allocation/drop calls. Single-target, effect-order,
literal-print policy and common backend authority are unchanged.

Whole-forest preflight retains the original limits: 64 source nodes,
256 expressions, 32 logical edges, 16 statements per body/suffix, depth eight
and 31 captures plus a gate. Unit proofs check the seven nodes / seven
expressions / zero logical edges boundary and adjacent rejection, 16/17
statements, 31/32 captures and depth 7/8.

## Source And Probes

[The Nuis source](../../tools/nuisc/tests/native_application_bridge/return_dual_i64_private_pair.ns)
uses exact i64 initialization, both write orders, nested selections, one shared
continuation, a sealed private result, unused effectful work and a bool return.
Owner updates consume the current child through the existing i64 XOR operation;
child updates consume its current full-width value through complement arithmetic.
These are source computations, not a transport codec.

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_dual_i64_oracle.rs)
replays the source imperatively without compiler transport logic. Seven seeds
cross both gates: signed extrema, values outside signed i32, minus one and
zero/one. Four additional raw native cases use positive/negative 96 and 97 so
carried values can equal trace markers.

The native probe observes both actual after-work arguments and the actual final
answer-work owner, while retaining source computations, private joins and real
traps. Failure prefixes use source-event positions, never searches through mixed
field/marker values. A failing argument traps before callee entry; observations
from earlier callees remain present for later failures. The unused/answer faults
carry child values 96/97, proving marker collisions during real late traps. Real callback state is
forwarded to its successor and output canaries are checked.

## Executed Acceptance

The 2026-10-12 serial scoped cohort contains **315 distinct tests**:
246 compiler unit/artifact tests, 36 native bridge tests, three CLI tests,
three host failure-policy tests and 27 tensor tests. Ten tests are new.
Repeated/repaired executions and the complete 307-test predecessor are
not added to the current count. The current CLI scope is dual-i64,
i64-ancestor and baseline selected-call build/cache/restore; the two other
float/i32 CLI tests in the predecessor are not counted as rerun here.

The new raw native matrix runs 64 successful native processes and
18 fatal native processes across normal/reversed inventories, proving
228 accepted callbacks, exact actual pair/final-owner values, source order
and output canaries. Nine original selected argument-proof stages trap
before callee entry. Reference fault cleanup is separate from fatal native
process policy and does not make native traps recoverable.

The new CLI matrix checks four images: fresh build, cache hit and two successive
source-free restores. It proves 36 CLI event traps and 516 accepted callbacks,
including healthy reentry after each failure. Only the last accepted open/event
state is published; later event/close/completion are absent after a fatal event.
Executable, native objects, packaging policy, envelope and lifecycle identity
remain unchanged. Each image and its input restoration container remain
byte-identical after use; fresh/cache diagnostic-bearing container bytes need
not match.

Existing i32/i64/float ancestor native, five-child-type event failure/budget
and helper-entry regressions remain included. The rebuilt ordinary CLI proves
2551 clean registered drift checks / 17723 required patterns, clean
coverage/hierarchy/task-card lineage, six source-equal fields and six complete
predecessor hashes. The selected cell remains `active/99`; this is not broad
persistent application-session closure.

Reproduce from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_i32_ancestor_private_bool return_i64_ancestor_private_bool private_event_budget private_event_failure helper_entries_share_exact_budget helper_entries_reject_before_body helper_entries_skip_unselected_paths helper_entries_use_unsigned_limits helper_entries_count_loop_free_roots helper_entries_include_scoped_iterations return_float_ancestor_private_bool return_dual_i64_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_i64_ancestor_private_bool native_effectful_selected_calls_build_cache_restore_and_execute_only_selected_arguments return_dual_i64_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
```

The tensor retains each complete 307-test predecessor field after its
float-ancestor marker:

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `0e7e8b2a8372bca9f93f1e78b627fd311843724f71c1d1e3557dd423b289756a` |
| `next_step` | `28ac881577eb478775b9be740c6a5799ee958288f9f0fb7805b7fad15662534d` |
| `blocker` | `179f3c3c11af4dee3335474e854f0e786ba43fdecfae65a4351d7c29bd721381` |
| `next_action` | `edae01ba27dcc5e141c2f0ae1bbf26dcbb6c0bde8bc8245f4b7bc26cff9bf718` |
| `validation_command` | `efbd56cebbb6eb30862ededba775390713dcc86dc94afc4c7a8a19d57f0e52df` |
| `expected_artifact` | `2166adecacef5e5ab1bdaeb7f22e17e961f7c233e07b9b042110c80b077edda1` |

## Remaining Boundary

Other numeric/numeric combinations and resource transport remain separate.
The next bounded proof should extend independently typed numeric fields to an
exact i64 outward owner beside an exact f32 or f64 private child, checking real
IEEE callee words and source-order traps before widening the admission predicate.
Readonly entering values, one outward target, seals and original budgets remain
mandatory.

This is a macOS CPU receipt, not new GPU/Linux/Windows execution, performance,
formal memory safety, self-hosting, full-workspace or strict-Clippy acceptance.
Public ABI stabilization and broader application-session closure remain open.
