# Native Epoch-Local Initializer Plan V1

Audit date: 2026-10-28. Git baseline: beta-0.18.4, `ceaa2caa`.
This worktree follows the [ready-local audit](nuis-native-local-scalar-argument-print-v1.md).
That receipt retains its own execution/delivery results; this phase does not
create a release or inherit those results as a fresh run.

## Analysis Only

The [private planner](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch_local_plan.rs)
produces borrowed analysis data for one fresh i64 definition immediately followed
by one printing observer. It is deliberately not called by the ordinary
initializer dispatcher. Existing source/native refusals remain refusals; this
is not yet a new executable language admission, runtime value or ABI.

The planner and the existing float-argument validator share the same iterative
[expression proof](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs).
The ordinary validator still starts with a non-lazy root and no bound-local
proof, requires the exact expected float kind, and rejects a comparison-only
final result. Seven historical structural assertions now follow the equivalent
refactored anchors; their check identities and semantic constraints are retained.

## Definition And Context

The candidate must be a fresh NIR `Let`, inferred or annotated as plain i64.
Its initializer must derive a comparison-only i64 value from at least one
certified original i32/i64 field. Canonical signed widening, wrapping/checked
arithmetic, XOR and completed same-kind pure helpers reuse the existing proof
rules. Whole-record, reference, unknown-name and effectful/cyclic helper paths
do not gain permission.

The result borrows the supplied epoch, whole block, scope, layouts and pure-helper
catalog. It records definition/consumer indices and exact root/field paths and
types, including nested records. These are in-memory context identities, not a
serializable certificate or a numeric timestamp substituted for field provenance.
An unready child or changed layout cannot reuse an ancestor/original projection.
Uncertified fields cannot fall back through the ordinary readiness callback.

The caller must supply the actual complete block associated with that epoch.
Only literal print markers may precede the candidate in this first profile;
arbitrary prefixes and rebinding the analysis to another block are not proved.
Connecting the source owner to these context identities is part of the next
admission gate, not an assumption that the current plan can authorize lowering.

## Consumption And Evaluation

The next statement must be a pure i64 observer with one exact f32/f64 argument.
Inside that argument the new word has exactly one lazy comparison-scoped use.
Same-kind forwarding and arithmetic keep its comparison-only provenance;
narrowing, cross-kind helpers, an ordinary/non-lazy use, a direct print, missing
consumption and duplicate occurrences are rejected.

The initializer remains eager at its source definition. A later false AND/OR
gate can skip the read, not the preceding initialization or its failure. The
plan records initializer fallibility separately from the total read of an
already-computed word. It borrows source statements; it does not substitute,
clone, hoist or reevaluate the initializer at the use.

The [bounded non-use scan](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch_local_scope.rs)
rejects later aliases, assignments, returns, field/record/call escapes and nested
redeclarations of the same name. Unhandled forms fail closed. It proves only
non-use of this binding, not that all remaining statements are admissible;
the source owner must still validate the rest of the block normally. NIR does
not preserve a source `let mut` modifier here: immutability means a fresh,
single-definition/read-only name over the supplied block, not a new syntax check.

## Shared Bounds

One 4096-work counter covers initializer and consumer expression traversal plus
the defining Let, outer observer Call and Print. The original conservative
pending-call reservation remains unchanged. Separate expression roots keep
their real AST depths, bounded at 64; no initializer tree is duplicated beneath
the consumer to fake accounting or scheduling.

The combined capture union is bounded at 31. It conservatively includes original
field paths, ready leaves and the new word itself. A separate 4096-node scope
scan, 32 statement nesting and 64 expression nesting bound the non-use audit;
this is not a second runtime expression allowance. Existing logical-root,
function, loop-work and helper-entry admission gates remain independent.

## Validation

295 distinct selected checks pass: 213 lowering/print/temporary/proof controls,
12 native/reference/oracle regressions, 40 Python maintenance tests, two
file-budget tests, one exact host-path integration and 27 tensor tests. Seven
raw-NIR controls are new; they are already included in the 213, not counted
again. They cover context identity/positions, nested field paths, eager
fallibility/once-only reads, escape and kind vetoes, child/layout/purity refusals,
exact shared budgets and unchanged ordinary availability. Failed preliminary
runs and repeated checks are not added to acceptance totals.

Native regressions execute existing ready-local and arithmetic/XOR paths and
retain actual compile/emit rejection of unproved epoch-local definitions. They
are not execution of a newly admitted definition. Formatting, whitespace,
UTF-8 across 5,990 tracked/unignored text files, 4,665 local documentation links
and locked offline metadata pass. Architecture policy remains clean across 29
packages, six domain crates and 74 normal/build edges. No file exception is
added; the proof index stays within its 300-line router limit.

The ordinary CLI is rebuilt with incremental overrides removed and reports
4,155/4,155 drift checks passing. Coverage, manifest, milestone, hierarchy and
task lineage are clean; the selected active/99 task is ready. All six current
source/runtime fields agree and preserve complete predecessor values and their
SHA-256 identities. Historical command syntax/inertness checks use stdin without
replay. No commit, push, tag or version bump is performed.

The earlier native/source-free ready-local delivery remains a separate receipt.
This phase neither executes a newly admitted epoch-local definition nor claims
GPU/provider parity, performance gains, stable ABI or whole-engine completion.

## Next Gate

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. Connect one proven definition/observer pair to its actual source-owned
epoch/block and normalization path without marking the record or unrelated
locals ready. Then prove once-only eager initialization, guarded consumption,
failure before State publication and actual native execution before widening
the profile or adding a new artifact-delivery claim.

## Reproduce

The subsequent [root-owned admission](nuis-native-epoch-local-argument-print-v1.md)
connects this analysis to a narrowly gated execution route. It records its own
native/source-free validation without rewriting this earlier analysis checkpoint.

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- epoch_local_plan --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- temporaries:: print_epoch:: iteration_prints:: iteration_effects:: --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- local_scalar_argument_print:: i64_xor_argument_print_epoch:: i64_arithmetic_argument_print_epoch:: --test-threads=1
```

## Predecessor Lineage

The six complete preceding functional tensor fields remain lossless alongside
their SHA-256 identities. Historical commands are quoted no-op here-document
data, checked through stdin without replay.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `97c8f8b8b57a4609a19872f3b2579f85aea21f0d1d654d6981d61a83ff6d65ba` |
| `next_step` | `6d0ee03193c9b8a1fba2ccf06fe2db9f7ecce69aea77d06c7fe1770dfcafea8f` |
| `blocker` | `c76bebb5c79ac9318d957124e7ff4d3a3a6af0690ad6e2df37209928a1176606` |
| `next_action` | `8ce798e81b3188fb4afac0e8e85288b97f756fefd7b34bd536aafe3d516e4f5d` |
| `validation_command` | `137a960a0517527c2eea8bddf7e35c0bf00fcd38e2d3ef18176bfc61e983db27` |
| `expected_artifact` | `bd0a4c775e955f8704d176777da9ef5b7035a117d0b5378dfb21e52f3be67e53` |
