# Native Private Nested Sequential Mixed Continuations V1

Status: worktree follow-up after Git baseline `304c49cd` (`beta-0.16.4`).
This composes the [sequential private carrier](nuis-native-private-sequential-mixed-continuations-v1.md)
with original nested branches. No release or public multi-value ABI is implied.

## Original Forest

Nested mixed branches may now contain their own sequential selections and
private scalar work before a common continuation. Every condition observes
the preceding current ancestor/child versions in its original lexical scope.
Fresh branch names remain private; a fresh preceding scalar may feed a nested
gate but may not escape that branch or collide with a later declaration.
An entering sealed binding or a third entering write target still rejects.
The actual source owner capability remains mandatory; generated joined versions
are not source-write authority. Only the exact bool ancestor crosses the
completed return boundary.

Preflight examines the complete original forest before proving or installing it.
One structural counter reserves 63 plus the first selection, each original
arm/continuation has at most 16 statements and source selections have at most
eight levels. The shared proof still charges each original node, expression and
logical edge once against 64/256/32; expression depth stays below 32 and each
helper has at most 31 captures plus a gate. Rejected proofs publish no partial
helper rewrite.

The same closed i64 carrier encodes ancestor + 2 * child in 0..=3. Canonical
bool conversion and pure equality/XOR/comparison decode add no trapping work,
source cast permission, public tuple or resource transport. Empty arms retain
both old values explicitly. For S original selections the route installs
2 * S guarded branch helpers, without copying any remaining continuation tree.
The earlier terminal-tree-plus-flat-tail route remains separate and can copy
only proved flat recipes, never a private carrier selection.

Six new NIR units cover both orders and polarities, branch-current conditions,
retention, lexical leaks/collisions, constant seals, extra writes, hidden leaves
and partial exits. Exact eight nodes / eight expressions / zero logical edges
passes while one-less budgets reject. Whole-forest 64/65 nodes, 16/17 body sizes,
8/9 selection levels and nested 31/32 captures have adjacent checks.
Original forests of 3/5/9 selections install exactly 6/10/18 branch helpers.
The existing scalar carrier's 256/257-expression and 32/33-logical-edge checks
remain part of the selected compiler regression.

## Native Evidence

The [native fixture](../../tools/nuisc/tests/native_application_bridge/return_nested_sequential_mixed_logical_effectful_scalar_selections.ns)
has an exact seventeen-field state and twelve independently checked helper sites.
Both first-selection arms contain nested gates, further branch-local selections
and trailing work before the shared outer sequence. One gate reads a fresh
current snapshot; another reads a sealed scalar. The source exit and caller
continuation remain distinct.

Two artifact units verify exact state, first-sensitive-node guards, local
transitive Effect paths, source-free policy and missing-Effect rejection.
Two CLI workflows use an independent imperative oracle, not the compiler
carrier or IR evaluator. All 16 truth variants poison inactive denominators
and compare complete state/stdout. Fresh/cache/three source-free restorations
exercise 240 lifecycle transitions, comparing policy and executable bytes after
source, manifest and project cache deletion. All twelve selected helper sites
must trap before later effects or state publication.

## Acceptance

The 2026-10-11 fresh selected six-cohort acceptance passes **663 distinct tests**:
401 compiler/outliner/artifact/direct-call/branch/async/loop/scalar-control,
eight typed-return/effect-boundary, 173 LLVM including ignored, 50 native bridge,
four CLI workflows and 27 tensor units. Actual test-name lists confirm eight
new compiler units (six NIR/two artifact), four unique CLI names and zero
compiler/typed overlap. Focus/repeat runs add no distinct units. The complete
preceding 655-test receipt is frozen in all six task fields as history, not
new acceptance. The intermediate tensor title failure was repaired by keeping
the stable parent task/evidence prefixes, without weakening its assertions.

The sequential/nested-sequential CLI cohort passes 480 lifecycle transitions
and 21 selected traps; the new fixture contributes 240 transitions and twelve
traps. Variants, lifecycle transitions and traps are not additional test units.
The selected commands reuse the [preceding six-cohort recipe](nuis-native-private-sequential-mixed-continuations-v1.md#acceptance),
substituting the nested-sequential CLI filter for the old nested-prefix filter:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- native_return_sequential_mixed_logical_effectful_scalar_selections native_return_nested_sequential_mixed_logical_effectful_scalar_selections --test-threads=1 --quiet
```

The contract is registered through
`tools/nuis/src/dev_tensor_drift_data_native_return_nested_sequential_mixed_logical.rs`.

After final task-field publication the rebuilt ordinary CLI reports clean
coverage/hierarchy/task-card lineage/drift, 2418/2418 checks and zero failures.
All six fields equal source and all six complete predecessor hashes remain
intact. The selected cell remains `active/99`; typed mixed private continuation
beyond bool pairs is next. The post-publication rerun passes all 27 tensor
tests and adds no distinct units to the 663-test receipt.

## Remaining Boundary

At this checkpoint typed mixed private continuation beyond bool pairs was next,
not arbitrary entering writes or a public multi-result ABI. The later
[i32-pair proof](nuis-native-private-i32-pair-continuations-v1.md) adds one exact
signed child without altering this historical receipt. Full-width i64 and other
scalar combinations still need their own exact codecs and helper return proof;
resources, hidden logical roots and mixed/partial source exits remain excluded.
This host scalar evidence does not claim new GPU/Linux/Windows execution,
measured speedups, authentication, formal memory safety or completed self-hosting.
Full workspace and strict Clippy acceptance remain separate.

