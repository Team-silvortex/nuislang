# Native Nested Continuing Field Selections V1

This bounded CPU checkpoint follows the
[complete 404-test predecessor](nuis-native-in-selection-field-exits-v1.md).
It admits bounded nested selections only in the continuing remainder after
one source bool exit inside a selected mixed-field arm.

## Implementation

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_nested_in_fields.ns)
keeps one lexical i64 owner and private i32/f32/f64/two-bool children.
The original selector is evaluated once. Its selected prefix updates a, b
and d; exit_work observes those current versions beside retained owner/c/e.
A true exit returns d AND answer_work, including a genuine false return.
No later nested condition, field update, owner update or shared tail runs.

Only a continuing path calls nested_work on current d. The true branch
updates c from current b/d and performs the gated peer call; the false
branch retains c and changes e. Both retain a/b/d and share the same owner
update and final tail. Inactive wide work and all nested work after an exit
receive zero proofs. There are thirteen separately fallible call boundaries.
Public state has twenty-two words, open twenty-one arguments, event fifteen.

The implementation replaces the flat-after shape check with a recursive
Let/Const/If-only continuing check, after whole-original-forest preflight.
Return, resource work and other statements cannot pass this check. Before
the exit, the other field arm and the common tail stay flat. The original
private-field certificate, typed arm proof, seal/root restrictions and
generic recursive field transport remain responsible for write authority.
No backend recipe, record exit flag or new function eligibility is added.

Three data-only six-field records carry prefix, nested choice and suffix
values; one guarded bool continuation owns the shared tail. Actual source
work is charged once: twenty nodes and three logical edges. Nineteen nodes,
one fewer required expression or two edges reject. Depth five passes and
six rejects. A second nested level with an empty retained arm uses twenty-one
nodes; depth four passes and five rejects. No depth/budget reset is allowed.
Nested source returns, entering writes, sealed children, width changes and
future-name collisions reject atomically. Lexical leakage rejects already
in the frontend, before installation.

## Execution And Delivery

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_nested_in_fields_oracle.rs)
combines integer complements, independent IEEE sign words, bool updates and
source-event positions. Sixteen finite gate/bool cases plus sixty-four
paired IEEE cases give eighty cases per inventory. IEEE coverage includes
paired signed zeros, subnormals, infinities and quiet/signaling NaNs; it is
not the full Cartesian product.

Both original then and mirrored else source polarities run normal/reversed
function inventories with actual previous callback output and canaries.
Two independent probe modes observe real exit_work prefix or after_work
completed-field arguments, plus answer_work owner arguments. These are
**not simultaneous observations** in a process. Source computations and
checks are not replaced by oracle outputs. Marker-valued float words also
exercise source-position failure cuts rather than value-based truncation.

The new family contributes **640 successful native processes**,
**104 fatal native processes** and 2128 accepted callbacks per complete replay:
two sources * two inventories * two probe modes * eighty healthy/thirteen
failure cases. Selected traps occur before the failing callee and preserve
exact prior observations. Repeated focused tests are not added to these totals.

Eight fresh/cache/two successive source-free images, four per source
polarity, contribute **104 CLI event traps** and **904 accepted callbacks**.
Each runs sixteen finite cases and thirteen failure/reentry cases. Source,
manifest and cache are removed before restoration. State/stdout, executable
and native-object bytes, policy/envelope/lifecycle, and container bytes stay
exact. IEEE bit claims belong to native/reference tests, not CLI text input.
AOT tests preserve guard-first transitive Effect ordering, explicit print
grants and no owned aggregate allocator/drop. Removing grants or Effect
paths still rejects; this is not a formal memory-safety proof.

## Scoped Acceptance

The 2026-10-13 serial macOS CPU acceptance cohort has
**415 distinct tests**: 362 compiler, seventeen native/reference, six CLI,
three host and 27 tensor. Eleven are new: four units, one AOT, two reference,
two native and two CLI. The simultaneous-float, completed-field-exit and
in-selection-exit native/reference predecessors are replayed; both exit CLI
predecessors are replayed. Other historical runtime evidence is preserved,
not counted as new execution.

Run from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields return_in_fields return_nested_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_multi_float_field_vector return_early_fields return_in_fields return_nested_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_early_fields return_in_fields return_nested_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

The selected tensor cell remains
`standard-library/ns-nova/persistent-application-session`, `active/99`.
The rebuilt ordinary CLI exposes six source-equal fields, six complete
predecessor hashes, 2960 clean drift checks / 20712 patterns, and clean
coverage/hierarchy/task-card lineage. Changed Rust formatting, UTF-8, file
caps and relative links are clean. Whole-workspace formatting still fails
in four unchanged historical files; no full-workspace acceptance is claimed.
All six complete predecessor strings are preserved verbatim after their
in-selection field exit history markers, not replaced with hash-only summaries.

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `9549c7939757eaf07dffba557b045433d787f5419ab1a97d22f496dce602df79` |
| `next_step` | `44e5b6be27513caf5eb444f06f25b42f0660a89a072ecd1600e9199465514251` |
| `blocker` | `283a28e267c9c50860ba2f5c995143b0d15a470a7b0feec3160520f1110f90d0` |
| `next_action` | `1f7dc502e01da6ce22405ab815a8eff304b548bbfc023c0fe4a0d3bb92f4da4f` |
| `validation_command` | `94401ac21ed18607003a77176ea7f00fcac33dc2d48fb5205be18ed2d74958fb` |
| `expected_artifact` | `76d36680943e124a2c74f9d3247cdaa8e1bef2d11e5a6447b337a78ffe8c477a` |

## Remaining Boundary

One original exit plus bounded nested continuing work is supported;
multiple exits remain separate. Next prove a bounded nested prefix before
that same single exit, with current versions and original-source accounting.
Nested other/common-tail selections, branch-local exports, resources and
broader roots need separate proof. This local CPU checkpoint does not claim
GPU/Linux/Windows execution, heterogeneous application closure, performance
measurement, completed self-hosting or whole-workspace/strict-Clippy acceptance.
