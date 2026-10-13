# Native In-Selection Field Exits V1

This bounded CPU implementation follows the
[complete 396-test predecessor](nuis-native-early-mixed-field-exits-v1.md).
It supports a source bool return inside one selected field arm, before that
arm's remaining updates and original field join.

## Control And Data Remain Separate

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_in_fields.ns)
initializes one local i64 owner and independent i32/f32/f64/two-bool children.
The selected arm updates a, b and d, then exit_work reads those current
versions beside retained c, e and owner. If d differs from e, it returns
d AND answer_work. Returning false still exits observe; start remains
allowed to publish its completed State. Selected proof traps instead
retain the last accepted callback state.

Only the continuing arm updates c from current b/d, performs the gated
peer update and advances the same owner. Shared after_work, another owner
update and the final bool return follow only if the function has not exited.
Unselected exit predicates and short-circuit RHSs keep zero proofs.
Exited paths keep zero wide/peer/parent/after proofs and do not evaluate them.

The whole original selection supplies real numeric-owner write evidence
and the existing private-child certificate. Removing the proved control-only
exit is solely a write-classification view; it is not an executable source
rewrite, permission inferred from field count or an entering-binding grant.
Original typed expressions, RHS versions, seals and readonly scope remain
proved from original slices. Branch-local child exports and resources
are not authorized by this shape.

The original arm condition is evaluated once into a collision-free bool
binding. Two existing pure field records carry completed prefix transport
and the non-exited suffix. The exit predicate is itself short-circuit-gated
by the original arm choice. A bool continuation owns the suffix and shared
tail exactly once, without copying a suffix into both arms or putting
control flags into data records.

Completed prefix projections do occur before exit: they carry real
completed work and retained values. Only the unfinished suffix join and
its projections are skipped on exit. This is not a claim that exited
paths perform no physical prefix transport. Six independently typed
fields and public twenty-one-word state are unchanged; open has twenty
arguments and event fourteen.

Both then-arm and mirrored else-arm exits use the same controller.
The mirrored source inverts the original input condition while moving
the arm to else, preserving the same effective gate and observable trace.
The first proof accepts one exit, flat before/after slices, a flat other
arm and an owner-only/fresh-binding shared tail.

Three units check two data-only six-field records, one guarded bool
continuation, one original condition call, collision-free generated names,
atomic authority/type/shape vetoes and original budgets. Eighteen original
body nodes and three original logical edges are charged once; seventeen
nodes, one fewer required expression or two logical edges reject the body.
The synthetic arm-choice edge is generated transport, not extra original
source work. Original forest/statement/expression/depth/capture limits do
not reset; depth five passes and depth six rejects the extra source nesting.

## Actual Observations And Delivery

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_in_fields_oracle.rs)
uses integer complements, separate IEEE sign XORs, boolean logic and
source-event traces. Eight paired IEEE profiles, four bool pairs and two
field gates add sixty-four cases to sixteen finite gate/bool cases:
eighty per inventory. This covers paired signed zeros, subnormals,
infinities and quiet/signaling NaNs plus the other integer extrema,
not a full IEEE Cartesian product.

The unchanged native driver runs normal/reversed inventories and actual
previous callback output with canaries and bounded flushed failure prefixes.
Two independent probe modes inspect actual exit_work prefix arguments or
after_work completed-field arguments, plus the called answer_work owner.
These are **not simultaneous observations** in one process. Both source
arm polarities run both modes; source computations are never replaced
with expected words.

A marker witness has prefix words `(70001, -70002, 76, 96, 1, 1)` and
completed words `(70001, -70002, 76, MIN|96, 1, 1)`. Floating words collide
with exit/after print markers. Failure cuts use source-event positions,
not observer word values. Exit and continuation snapshots stay distinct.

Per acceptance replay the new family has **640 successful native processes**,
**96 fatal native processes** and 2112 accepted callbacks:
two sources * two inventories * two probe modes * eighty healthy/twelve
failure cases. Focused repeats are not added. Twelve selected proofs trap
before the failing callee; earlier observations remain exact.

Eight new fresh/cache/two successive source-free images, four per source
polarity, contribute **96 CLI event traps** and **864 accepted callbacks**.
Each image runs sixteen finite cases and twelve failure/reentry cases.
Source, project manifest and cache are removed before both restores.
Public state/stdout, executable/native-object bytes, packaging/envelope/
lifecycle and input container identity remain exact. Raw IEEE/NaN claims
belong to reference/native observations, not the finite CLI parser.

AOT checks both polarities for data-only typed records, a single bool
continuation, guard-first transitive Effect order, explicit print grants
and no owned allocator/drop. Removed Effect paths or default/empty grants
remain vetoes. This is not formal memory safety.

## Scoped Acceptance

The 2026-10-13 serial macOS CPU cohort has **404 distinct tests**:
357 compiler, thirteen native/reference, four CLI, three host and 27 tensor.
Ten are new: three units, one AOT, two reference, two native and two CLI.
Both simultaneous-float and completed-field-exit native/reference families
are freshly replayed; the directly preceding completed-field-exit CLI
family is also freshly replayed. Other CLI/native histories remain
preserved rather than counted as new execution.

Reproduce from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields return_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_multi_float_field_vector return_early_fields return_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_early_fields return_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

The rebuilt ordinary CLI exposes six source-equal current fields and six
complete preceding hashes, 2942 clean drift checks / 20595 patterns,
clean coverage/hierarchy/task-card lineage and the same selected coordinate.
The cell remains `standard-library/ns-nova/persistent-application-session`,
`active/99`. Changed Rust formatting, UTF-8, file caps and relative links
are clean; four unchanged historical files still fail whole-workspace
formatting. This is not full-workspace/strict-Clippy acceptance.

All six complete predecessor fields remain verbatim after the early
mixed-field exit history markers:

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `077fe2a71de87ed4ec92f8947ea789a7875c607bab20e421f8348a3cf1c60a40` |
| `next_step` | `a496c138673bcc29e88e1b10cc78523c9c5a5dabb314a36789b2d3408bb05ddb` |
| `blocker` | `c23fc965c8dce802d6023154f16d151022e070394755ac141dd85261573ffa2c` |
| `next_action` | `1340d8c3bddb660f3d1a8d5a4b423beb0976af8c90f99e5e765646053df44704` |
| `validation_command` | `54ff9074b53b24a8a9b30e2d60e6af6f6fee24f68ce4b875cdebdd1b9b0e367a` |
| `expected_artifact` | `a0b3fc2eef4da282eaa4c4d9f344bb0b849d99e8453f69afcbfe0331afc1059d` |

## Remaining Boundary

One flat selected-arm exit is now executable in either polarity; nested/multiple exits remain separate.
Next certify a bounded nested continuing selection after that exit,
preserving current fields and inherited budgets without copying a shared
tail. Branch-local exports, resource/root grants, application-wide
heterogeneous closure, GPU/Linux/Windows execution, performance measurement
and completed self-hosting are not claims of this local CPU proof.
