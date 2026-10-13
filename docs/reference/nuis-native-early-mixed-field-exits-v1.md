# Native Early Mixed Field Exits V1

This bounded CPU implementation follows the
[complete 386-test predecessor](nuis-native-simultaneous-float-field-vector-continuations-v1.md).
It adds a source-selected bool early return after a completed mixed-field
selection, while keeping function-level control separate from field records.

## Bounded Source Shape

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_early_fields.ns)
starts one local i64 owner and independent i32/f32/f64/two-bool children.
One selected field update complements the integer, toggles original-width
float signs, consumes the updated narrow value, updates bool siblings and
computes the owner. Its inactive arm retains all six fields.

exit_work consumes the actual joined fields, not entering seeds.
If its current bool siblings differ, the source returns immediately.
The source's d AND answer_work also skips the RHS when d is false.
Otherwise after_work reads all six current fields, parent_work updates
the same numeric owner, and the final bool return is evaluated.
The outer inactive arm skips all private work. Proof inputs for all
unselected calls are zero, including after_work on every exited path and
answer_work on the short-circuited early path.

Normal early return exits observe, not its caller: start still prints 79
and publishes its completed State, even when the returned bool is false.
Only a selected proof trap retains the last accepted callback state.

The new proof accepts one completed field selection followed by one exact
single-Return(Some(bool)) arm with an empty alternate and a flat continuation.
The selected owner must have real prefix-local numeric write evidence;
matching terminal targets, ordinary private-field classification, exact
scalar types and original entering/sealed-name vetoes still apply.
The flat continuation can advance only that owner or fresh bindings.
It cannot redefine a private child or manufacture a new numeric root.

The continuation becomes one guarded bool-return helper through the existing
Plan/StagedNested interface. The field carrier is unchanged and contains
only independently typed fields, never an exit flag or function return.
No continuation is copied into both branches. Each source condition, field
join, call and return is evaluated once, in its selected source position.
Rejected proof candidates install no helper, type or name.

Three units assert this structure and atomic type/authority/shape vetoes.
The fixture's eighteen original body nodes and three logical edges are
charged exactly once: seventeen nodes, one fewer required expression, or
two logical edges reject it. The depth-six witness passes and depth seven
rejects it without resetting the inherited budget. Existing 64-node,
256-expression, 32-logical-edge, 16-statement, depth-eight and 31-capture
limits remain shared; a complete compiler cohort replays prior limits.

## Actual Execution

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_early_fields_oracle.rs)
uses integer complements, separate IEEE sign XORs, bool logic and explicit
source traces. Six public seeds remain inputs; private fields are not
added to public state. Open has twenty arguments, event fourteen and
state twenty-one words.

Each inventory has eighty cases: sixteen finite cases covering four gate
pairs and four initial bool pairs, plus eight paired IEEE profiles times
four bool pairs and two field gates. The paired profiles include signed
zero, subnormal, infinity, quiet/signaling NaN words, with the other signed
integer extrema. This is not a full IEEE Cartesian product.

Normal and reversed function inventories run the same reference and native
cases. A native observer dumps the actual six exit_work callee arguments
before its print; another observes the final answer_work owner if called.
No source computation or expected state replaces those arguments.
Failure-prefix cuts use source-event positions, not floating payload values.
The unchanged native driver forwards actual previous callback output,
checks canaries, flushes print prefixes and bounds subprocesses.

The new family contributes **160 successful native processes** and
**24 fatal native processes**, with 528 accepted callbacks per two-inventory
acceptance replay; focused reruns are not added. Twelve proof
failures trap before their selected callee; completed earlier calls remain
visible, later calls and completion remain absent. Reference failed events
retain the last accepted state and fault the session.

Four fresh/cache/two successive source-free images contribute
**48 CLI event traps** and **432 accepted callbacks**, including healthy
reentry. Each image runs sixteen finite cases and twelve failure/reentry
cases. Raw IEEE/NaN observations belong to reference/native tests, not the
finite CLI parser. Source, manifest and cache are removed before both
restores. Executable/native-object bytes, public state/stdout, packaging,
envelope, lifecycle and input container bytes remain exact.

AOT checks assert the six exact field kinds, guard-first transitive Effect
order, a single bool continuation and explicit print-policy grants.
Removed Effect paths, default/empty print grants still reject; owned
allocator/drop calls remain absent. This is not formal memory safety.

## Scoped Acceptance

The 2026-10-13 serial macOS CPU cohort has **396 distinct tests**:
353 compiler, nine native/reference, four CLI, three host and 27 tensor.
Ten are new: three units, one AOT, two reference, two native and two CLI.
Because the return-prefix proof changes, the complete simultaneous-float
native/reference and CLI predecessor families are freshly replayed.
Other native/CLI histories are preserved, not counted as fresh execution.

Reproduce from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_multi_float_field_vector return_early_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_multi_float_field_vector return_early_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

The rebuilt ordinary CLI exposes six source-equal fields, all complete
preceding field hashes, 2923 clean drift checks / 20461 required patterns
and clean coverage/hierarchy/task-card lineage. The selected cell remains
`standard-library/ns-nova/persistent-application-session`, `active/99`.
Changed Rust files pass scoped formatting checks; whole-workspace
formatting still has four unchanged historical-file failures.

All six full predecessor fields are preserved verbatim after simultaneous
float field-vector history markers. Fresh complete-field hashes are:

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `a1d678b59cf760ff0ad706bdb8d4bf2979471b39e4f62198726f23cf78cb4c3e` |
| `next_step` | `99b71ec6a3e12cbd2ef51328a8fa6d946e852b64a99ecdcac28a3b826be5d334` |
| `blocker` | `2d61208e8a5b041f7ef770fb58167d86fc238422ba923e4dc49cfae4386c954c` |
| `next_action` | `a5760de3756f52c5f4f4ab100c7a99376294bc04fcfecb95e7d9e83347baf360` |
| `validation_command` | `f04e5a620537e1972a92c480ff6304d8d47713b27a7290092ce0f6f8da010a21` |
| `expected_artifact` | `80d7681736af4ecc0df52dc870915fdb4c7cba7340bfbfc9d1a20bfb34fa4d52` |

## Remaining Boundary

This proves a bool exit **after** a completed field join. Exits inside a
selected field arm before a field join remain separate: they must suppress
unfinished transport/projections and subsequent work while continuing
arms retain authorized current fields. Multiple exits, branch-local child
exports, resources and broader numeric roots are not authorized here.

Next prove one bounded in-selection source bool exit through existing
controller/field contracts without copying a shared continuation or using
neutral record capacity as exit authority. Application-wide heterogeneous
closure, GPU/Linux/Windows execution, performance measurement and completed
self-hosting are not claims of this local CPU receipt.
