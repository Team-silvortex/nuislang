# Native Floating Owner Field Vector Continuations V1

This bounded CPU proof follows the
[complete 402-test predecessor](nuis-native-two-bool-field-vector-continuations-v1.md).
The existing exact-scalar field list now has executed evidence for an
original-width f32/f64 outward owner beside i32, floating and two private
bool children. Production admission and transport are unchanged.

## Original Width And Current Values

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_float_owner_field_vector.ns)
takes five independent seeds. The state has 23 words, open has 22
arguments and event has 17 arguments. Initial child-only selections retain
the authorized owner; subsequent parent computations toggle its own IEEE
sign rather than converting it to an integer or widening f32 to f64.

The two bool children read updated siblings through AND/OR, with poisoned
inactive RHS proofs. Nested stages retain and update distinct children.
after_work observes all five real callee parameters; answer_work observes
the final owner separately. Public seeds are not exported private versions.

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_float_owner_field_vector_oracle.rs)
uses i32 complements, raw IEEE sign XORs and bool logic. Per width/inventory
it runs 12 owner IEEE words * 2 companion child profiles * 4 i32 values *
4 bool pairs * 4 gate pairs, plus 8 masking witnesses: 1544 cases.
The two child profiles are independently selected by offsets 1 and 5;
this is not a full IEEE Cartesian product. Each original width includes
signed zero, subnormal, finite, infinity and quiet/signaling NaN words.

Eight explicit NaN/zero witnesses per width demonstrate why the final
bool is insufficient: current and stale-sibling computations have identical
selected calls and identical results, but different final owner bits.
Both the original and deliberately stale source versions are compiled
and executed under normal/reversed inventories. Observers dump actual
callee arguments, not oracle-manufactured values. Separate finite mutants
also change the public result in reference execution.

The late raw snapshot is `(sign | 95, -70002, 96, 1, 1)`, with final
owner word `95`. Source-event positions distinguish word/marker collisions.
Actual previous callback output is forwarded into the next callback,
with boundary canaries, not an oracle replacement state.

## Authority And Shared Checks

Two new unit families require exact floating Target authority, original-
width neutral owner/child values and five distinct typed projections.
Qualified, sealed, bool, unsigned and resource owners remain vetoed;
removing genuine local owner authority remains an atomic source rejection.
Existing child lexical rules and shared forest/expression/logical/capture
bounds are replayed unchanged, not reset for the floating owner.

The shared AOT checker now accepts an explicit expected owner type.
Its legacy check_layout wrapper still requires i64; initialization
parameter/result types and the supplied full record layout are asserted.
The new layout is
`{owner:FLOAT;child_0:i32;child_1:FLOAT;child_2:bool;child_3:bool}`.
Guard-first transitive Effects, removed-order/default/empty-grant vetoes,
absence of owned allocation/drop, safe narrowing and absence of live
float-width conversion remain checked. This is not formal memory safety.

## Executed Acceptance

The 2026-10-12 serial macOS CPU cohort has **385 distinct tests**:
343 compiler, 10 native/reference bridge, two CLI, three host boundary
and 27 tensor tests. Thirteen are new: two units, one AOT, two reference,
six actual native and two CLI. Focused reruns are not added to the count.

Production field admission/transport and the shared native driver are
unchanged. Native replay selects the new floating-owner family and the
two predecessor reference tests. Prior native field families and prior
CLI workflows remain historical, not counted as freshly replayed.
The shared checker and structural/control-flow compiler cohort are replayed.

The new original/mutant fixture contributes 6208 successful native processes
and 60 fatal native processes, with 18744 accepted callbacks. This includes
6176 original and 32 deliberately stale successful processes. Both widths
and inventories observe original owner/child IEEE words and distinct
bool words. Fifteen selected failures stop before the failing callee,
exclude subsequent work/publication, and preserve earlier observations.

Eight fresh/cache/two successive source-free images contribute
120 CLI event traps and 1032 accepted callbacks, including healthy reentry.
Each image runs 18 finite-decimal cases: four independent seed tuples
across all four gate pairs and two extra nested updates. NaN/raw payload
claims belong to the native/reference matrix, not the finite CLI parser.

Source, manifest and project cache are removed before both restores.
Public state/stdout, binary/native-object bytes, packaging mode, envelope
and lifecycle remain exact. A failed event exposes only last accepted
open/event state, with no later event, close or completion. Reentry remains
healthy; execution/restoration do not mutate the image or input container.

Reproduce from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_float_owner_field_vector return_two_bool_field_vector_reference return_two_bool_field_vector_independent --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_float_owner_field_vector --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
```

The ordinary CLI is rebuilt separately. It exposes six source-equal fields
and six complete predecessor hashes, with 2867 clean drift checks,
clean coverage, hierarchy and task-card lineage. The selected cell stays
`active/99`, not broad application closure.

The complete six 402-test predecessor fields remain verbatim after the
two-bool field-vector history markers:

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `05a346e908c0052f8e3010020cfd7ff872aa66c4a3a452e58a4fb618dce20d02` |
| `next_step` | `f53788eb5a1353afdeddb33b731c6e64f9280b436b758f1bd693569195268415` |
| `blocker` | `f66432099a9aaa1b3323d30ce64e3dc2e6fe21d22f72f8a7c4573fe2bd474c76` |
| `next_action` | `7cdd580214a6f47acb90a43365f10b1ba72a93d45a66c00a7e95b8a0fedbc68d` |
| `validation_command` | `14f85048c745d43fbef6e3161adb6ef3f41a84989d19218adba81b1209404961` |
| `expected_artifact` | `d238b2ce7dfade73c82a1278d0169e25568c5d507f6a1e9a9261405184c8735a` |

## Remaining Boundary

Next execute independently typed cross-width floating owner/child lists
(f32/f64 and f64/f32) with current/retained numeric and bool children.
Keep the same numeric Target authority, field-driven assembler, original
budgets and real-bit observations; do not introduce widening, integer
carriers, backend/type recipes or new source permission.

Resources, broader roots, mixed exits and application-wide closure remain
separate. This receipt is not fresh GPU/Linux/Windows execution, measured
performance, full-workspace/strict-Clippy acceptance or completed self-hosting.
