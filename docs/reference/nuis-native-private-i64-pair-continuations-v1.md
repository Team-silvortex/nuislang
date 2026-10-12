# Native Private I64 Pair Continuations V1

Status: worktree follow-up after Git baseline `304c49cd` (`beta-0.16.4`).
This extends the [i32-pair proof](nuis-native-private-i32-pair-continuations-v1.md)
with a full-width signed child, not a public tuple or new return ABI.

## Exact Owned Carrier

The bool/bool and bool/i32 scalar codecs remain unchanged. One exact owned bool
ancestor and one prefix-private exact i64 child use a compiler-private owned
record with two fields: `owner: bool` and `child: i64`. All 64 child bits remain
independent of the flag, including MIN/MAX and values outside the i32 range.
There is no narrowing, arithmetic encoding, pointer, allocation or replay of
selected work. Two projections read the same selected result.

The record reuses existing NIR structs, YIR owned aggregate calls/returns and
backend scalar-layout admission. Its type and temporary names are allocated
only after the whole candidate proof succeeds; existing type names are reserved.
Each original selection installs two guarded helpers and one private layout.
The inactive helper returns only exact typed literal zeros. Existing guard seed
verification, complete layout checking and local Effect ordering remain active.
The aggregate is private; only the actual bool ancestor reaches the outward
result. The helper result does not authorize new source writes.

## Original Authority

The classifier still requires exactly the actual ancestor and one actual
prefix-private child write target. The child must be defined, unsealed and absent
from the entering scope. Incoming/third writes, resource values, modifiers,
optional/generic types, hidden logical leaves and partial exits remain excluded.
Numeric writes use the classified Child token, not a fabricated bool Target.
Capture discovery retains every current/old value required by reads or retention.

Whole-original-forest preflight, lexical isolation, constant seals and source
budgets remain unchanged: 64 nodes / 256 expressions / 32 logical edges, eight
selection levels, 16 statements per arm/continuation and 31 captures plus a gate.
No arm receives a copy of the remaining sequence. Six NIR units check both write
orders/polarities, nested/sequential joins, exact private layouts, collision-free
names (including actual suffix collisions and existing source types), all-rejected
module/name atomicity, source authority, lexical isolation and adjacent capture limits. Exact
six nodes / twelve expressions / zero logical edges passes; one-less node or
expression budgets reject. 2/5/9 original selections install 4/10/18 helpers and
2/5/9 private layouts. Generated transport does not consume source-work budgets.

## Native Evidence

The [Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_i64_pair_mixed_logical_effectful_scalar_selections.ns)
uses an exact thirteen-field state, nested/sequential current reads, signed
wrapping flips, retained arms, unused checked work and one source bool exit.
Two artifact units check the exact owned return layout, no narrowing casts,
leading guards, local transitive Effect paths and source-free policy. Removing
required ordering or print grants must still reject the artifact.

Two CLI workflows use an independent signed imperative oracle, not the compiler
transport or an IR evaluator. Seven seeds include i64 MIN/MAX, negative/zero,
and values above/below the i32 range; two bool flags give 28 combinations.
Fresh/cache/three source-free restorations define 420 lifecycle transitions;
all nine selected helper sites are checked for traps before later effects or
state publication. Inactive proof denominators are poisoned. Source, manifest
and project cache are deleted before restoration; policy and executable bytes
must remain identical.

## Acceptance

The 2026-10-11 selected six-cohort acceptance passes **679 distinct tests**:
417 compiler/outliner/artifact/direct-call/branch/async/loop/scalar-control,
eight typed-return/effect-boundary, 173 LLVM including ignored, 50 native bridge,
four CLI workflows and 27 tensor units. Actual name lists establish eight new
compiler units (six NIR/two artifact), four unique CLI names and zero
compiler/typed overlap. Final compiler reruns include strengthened actual-suffix
collisions, existing source types and all-rejected module/name atomicity.
Repeated/focused runs add no distinct units. The complete preceding 671-test
receipt remains frozen in all six task fields as history, not new evidence.

The i32/i64 CLI cohort passes 720 lifecycle transitions and 18 selected traps;
the new fixture contributes 420 transitions and nine traps. Variants, transitions
and traps are not additional test units.
Reuse the [preceding six-cohort recipe](nuis-native-private-i32-pair-continuations-v1.md#acceptance),
replacing the nested-sequential CLI filter with the i64-pair filter:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- native_return_i32_pair_mixed_logical_effectful_scalar_selections native_return_i64_pair_mixed_logical_effectful_scalar_selections --test-threads=1 --quiet
```

The drift contract is registered through
`tools/nuis/src/dev_tensor_drift_data_native_return_i64_pair_mixed_logical.rs`.

After final task-field publication the rebuilt ordinary CLI reports clean
coverage/hierarchy/task-card lineage/drift, 2442/2442 checks and zero failures.
All six fields equal source and all six complete predecessor hashes remain
intact. The selected cell remains `active/99`; exact bool/f32 and bool/f64
private continuations are next. The post-publication rerun passes all 27 tensor
tests and adds no distinct units to the 679-test receipt.

## Remaining Boundary

The subsequent [float-pair proof](nuis-native-private-float-pair-continuations-v1.md)
provides independent source-type and native bit-preservation evidence for exact
bool/f32 and bool/f64 children, including negative zero and NaN payloads. This
679-test receipt remains historical. Integer
ancestors, resources, hidden logical leaves and mixed/partial source exits remain
separate. This scalar-host evidence does not establish new GPU/Linux/Windows
execution, measured speedups, authentication, formal memory safety or completed
self-hosting. Full workspace and strict Clippy acceptance remain separate.
