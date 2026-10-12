# Native Private I32 Pair Continuations V1

Status: worktree follow-up after Git baseline `304c49cd` (`beta-0.16.4`).
This extends the [nested sequential proof](nuis-native-private-nested-sequential-mixed-continuations-v1.md)
with exact bool/i32 private transport, not a public tuple or return ABI.

## Exact Carrier

The existing bool/bool carrier remains unchanged. An exact owned bool ancestor
and one prefix-private exact i32 child may now cross terminal, nested and
sequential private joins through the same guarded i64 helper ABI.

The codec is `sign_extend(child) + 2147483648 + bool(owner) * 4294967296`.
The biased child occupies 32 bits and the flag occupies bit 32, giving exactly
0..=8589934591. All original i32 values, including MIN/MAX, negative and zero,
remain representable. Decode compares against 4294967296, subtracts that flag
and the bias, then narrows an already in-range signed i32. It uses no division,
remainder, pointer, allocation, trapping arithmetic, lost sign bit or second
public result. Generated names remain collision-free at installation.

This is an exact 33-bit codec, not a recipe for squeezing arbitrary full-width
bool/i64 into one i64. Other scalar families, references, optional/generic
types and resource carriers remain rejected by this route.

## Original Authority

Classification still requires exactly the actual ancestor and one actual
prefix-private child source-write target. The child must be defined, unsealed
and absent from the entering scope. It does not authorize a third target,
incoming child or root with no actual ancestor write. Its numeric scalar write
uses the classified Child token without manufacturing a bool Target capability.

Whole-original-forest preflight, lexical isolation, constant seals, retention
captures, original condition guards and local Effect ordering stay mandatory.
Each original selection installs two branch helpers without copying the remaining
tail. Source limits remain 64 nodes / 256 expressions / 32 logical edges, eight
selection levels, 16 statements per arm/continuation and 31 captures plus a gate.
A terminal i32 selection may have no suffix because both private values are
decoded once before returning only the ancestor; legacy bool routing is retained.

Six new NIR units check both write orders/polarities, nested sequences, retained
versions, exact type/capture/lexical vetoes, collision-free names and linear
helper counts. The codec test checks seven signed boundary samples with both
flags and uses checked i32 conversion so a truncation cannot hide a failure.
Exact six nodes / fourteen expressions / zero logical edges passes and one-less
node/expression budgets reject. Capture unions of 31/32 have adjacent checks;
2/5/9 original selections install 4/10/18 guarded branch helpers. The earlier
whole-forest structural and logical-budget regression remains selected.

## Native Evidence

The [Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_i32_pair_mixed_logical_effectful_scalar_selections.ns)
uses an exact thirteen-field state and nine independently checked helper sites.
Its child changes through existing wrapping i32 arithmetic, including MIN/MAX
sign flips, nested/sequential joins, retained arms and final comparison against
the original seed. Unused work, source exit and caller continuation are checked.

Two artifact units verify scalar cast kinds, ancestor guards, exact state,
local transitive Effect paths, source-free policy and missing-Effect rejection.
Two CLI workflows use an independent signed imperative oracle, never the codec
or an IR evaluator. Five signed seeds and two bool flags cover twenty input
combinations. Poisoned inactive denominators, complete state/stdout and source
return behavior are compared through fresh/cache/three source-free restorations:
300 lifecycle transitions and nine selected helper-site traps. Source, manifest
and project cache are deleted before restoration; policy and executable bytes
must remain identical.

## Acceptance

The 2026-10-11 fresh selected six-cohort acceptance passes **671 distinct tests**:
409 compiler/outliner/artifact/direct-call/branch/async/loop/scalar-control,
eight typed-return/effect-boundary, 173 LLVM including ignored, 50 native bridge,
four CLI workflows and 27 tensor units. Actual test-name lists confirm eight
new compiler units (six NIR/two artifact), four unique CLI names and zero
compiler/typed overlap. Focus/repeat runs add no distinct units. The complete
preceding 663-test receipt remains frozen history in all six task fields, not
new evidence.

The i32-pair/nested-sequential CLI cohort passes 540 lifecycle transitions and
21 selected traps; the new fixture contributes 300 transitions and nine traps.
Variants, lifecycle transitions and traps are not additional test units.
Reuse the [preceding selected recipe](nuis-native-private-nested-sequential-mixed-continuations-v1.md#acceptance),
replacing the sequential CLI filter with the i32-pair filter:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- native_return_i32_pair_mixed_logical_effectful_scalar_selections native_return_nested_sequential_mixed_logical_effectful_scalar_selections --test-threads=1 --quiet
```

The drift contract is registered through
`tools/nuis/src/dev_tensor_drift_data_native_return_i32_pair_mixed_logical.rs`.

After task-field publication the rebuilt ordinary CLI reports clean
coverage/hierarchy/task-card lineage/drift, 2430/2430 checks and zero failures.
All six fields equal source and all six complete predecessor hashes remain
intact. The selected cell remains `active/99`; full-width bool/i64 private
continuation transport is next. The post-publication rerun passes all 27 tensor
tests and adds no distinct units to the 671-test receipt.

## Remaining Boundary

At this checkpoint the next weak boundary was full-width bool/i64 private
continuation transport. The later [i64-pair proof](nuis-native-private-i64-pair-continuations-v1.md)
reuses exact guarded owned aggregate returns, not lossy narrowing, repeated
selected work or cloning the remaining tail. This historical receipt is unchanged.
Integer ancestors, other scalar families, resources, hidden logical leaves
and mixed/partial exits remain separate. Source owner authority and one public
result are unchanged. This scalar-host proof does not claim new GPU/Linux/Windows
execution, measured speedups, authentication, formal memory safety or completed
self-hosting. Full workspace and strict Clippy acceptance remain separate.

