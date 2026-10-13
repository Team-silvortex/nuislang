# Native Nested Pre-Exit Field Selections V1

This bounded CPU checkpoint follows the
[complete 415-test predecessor](nuis-native-nested-continuing-field-selections-v1.md).
It admits bounded nested field work before the same single original bool
exit, as well as the already supported nested continuing remainder.

## Source Work And Authority

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_prefix_in_fields.ns)
keeps one lexical i64 owner and private i32/f32/f64/two-bool children.
After the original field gate, a and b update first. A prefix condition
reads original d once, then either updates c using current b/d or flips e.
The original bool update follows, and exit_work receives completed current
a/b/c/d/e beside the retained owner. Flipping e changes the actual exit
decision; reading stale e would select the opposite path. Reading stale b
would conceal the finite wide-sign change. On exit, all completed prefix
work remains, but no continuing condition, update or shared tail runs.

Continuing paths retain that completed snapshot and execute the existing
nested float-versus-peer choice, the owner update and one shared final tail.
Inactive prefix arms and all exited suffix work use zero proofs. Normal
false returns still exit observe while start may publish its completed
State. Selected proof traps instead preserve the last accepted callback.

The split locates the first non-continuing original statement, not the
first if. It must still be an if containing exactly one bool source return
and an empty other arm. Recursive Let/Const/If-only checks require both
prefix and suffix to be continuing forests: extra or nested returns cannot
pass. Whole-source 64-node/eight-depth/sixteen-statement preflight runs
before the split. The other field arm and common source tail stay flat.

Existing lexical numeric-owner/private-child certificates, typed arm
staging, seals and isolated names validate both forests from original
slices. Classification removes only the proved control exit, never source
work or checks. Neither field count nor the new source shape grants entering
writes, outward child exports, resources or broader function eligibility.
The old pre-exit-nesting veto is narrowed to a real additional-return veto,
not silently removed from the security boundary.

Four data-only six-field records carry the prefix choice, completed prefix,
continuing choice and suffix. The original field selector is still bound
once; one guarded bool continuation owns the common tail. Source calls and
conditions are not replayed, and no exit flag is added to a field record.
Completed prefix joins/projections legitimately occur before a source exit;
only unfinished suffix work is skipped.

Original work consumes twenty-three nodes and three logical edges once.
Twenty-two nodes, one fewer required expression or two logical edges reject.
Inherited depth five passes and six rejects. A deeper prefix choice with an
empty retained arm consumes twenty-four nodes; depth four passes and five
rejects. Numeric owner/field types and the 32-parameter capture limit are
unchanged. Prefix extra exits, entering writes, sealed children, width
changes and future-name collisions veto installation atomically. Lexical
leakage is already rejected in the frontend.

## Execution And Delivery

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_prefix_in_fields_oracle.rs)
uses signed integer complements, separate IEEE sign words, bool updates
and original source-event positions. Sixteen finite gate/bool cases plus
sixty-four paired IEEE cases give eighty per inventory. Paired signed zeros,
subnormals, infinities and quiet/signaling NaNs are covered, not a full
Cartesian product. Public state has twenty-five words, open twenty-four
arguments and event eighteen.

Two source polarities and normal/reversed function inventories run actual
previous callback outputs with canaries. Independent prefix/finished probe
modes inspect real exit_work or after_work arguments plus answer_work owner;
these are **not simultaneous observations** in a process. Prefix IEEE and
peer changes are not manufactured in the compiled source or driver.
Marker-valued words exercise source-position failure cuts.

The new family contributes **640 successful native processes**,
**128 fatal native processes** and 2176 accepted callbacks per complete replay:
two sources * two inventories * two probe modes * eighty healthy/sixteen
failure cases. Prefix-condition and both selected prefix-arm checks trap
before their callee; exact earlier work is retained. Focused repeats are not
added to these totals.

Eight fresh/cache/two successive source-free images, four per polarity,
contribute **128 CLI event traps** and **1024 accepted callbacks**. Each image
runs sixteen finite cases and sixteen failure/reentry cases. Source, project
manifest and cache are removed before both restorations. State/stdout,
executable/native-object bytes, policy/envelope/lifecycle and input container
identity remain exact. IEEE claims belong to reference/native bit observers,
not CLI text parsing. AOT checks retain guard-first transitive Effect order,
explicit print grants and no owned aggregate allocator/drop; absent grants
or Effect paths still reject. This is not formal memory-safety acceptance.

## Scoped Acceptance

The 2026-10-13 serial macOS CPU acceptance cohort has
**409 distinct tests**: 367 compiler, eight native/reference, four CLI,
three host and 27 tensor. Eleven are new: four units, one AOT, two reference,
two native and two CLI. The direct predecessor's native/reference and CLI
families are freshly replayed; older runtime histories are preserved rather
than added to this fresh cohort. The previous 415-test checkpoint therefore
remains intact even though this more focused fresh scope has 409 tests.

Run from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields return_in_fields return_nested_in_fields return_prefix_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_nested_in_fields return_prefix_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_nested_in_fields return_prefix_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

The selected cell remains
`standard-library/ns-nova/persistent-application-session`, `active/99`.
The rebuilt ordinary CLI exposes six source-equal current fields and six
complete predecessor hashes, 2978 clean drift checks / 20846 patterns, and
clean coverage/hierarchy/task-card lineage. Changed Rust formatting, UTF-8,
file caps and relative links are clean. Whole-workspace formatting retains
four failures in untouched historical files; it is not a full-workspace pass.
All six complete predecessor fields are preserved verbatim after their
nested continuing selection history markers, not replaced by hash summaries.

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `6b65cc2be2d6067f9063051391a433ac38ac864541fffaa53957614c863fd8be` |
| `next_step` | `15a7fb41e25c92b941202dc8ca46182aef9a3785f5552f5d924aa4a5e7835e86` |
| `blocker` | `97a325a65637e2493a77ed7a59d9fed4ba58a9349592078df56fb051da48a32f` |
| `next_action` | `1610541473636c70dfd58778004f10e2072743f2d652e05016162b0707c23801` |
| `validation_command` | `57aaf94e228b494a6693bc9c5a4bf70ebb3e018e4abb2dc3161e1618720603c5` |
| `expected_artifact` | `37f1f2286349d44291d8b437d950732cc337c654eda8e04739222a44c441aa63` |

## Remaining Boundary

One source exit with bounded continuing work on both sides is executable;
multiple exits remain separate. Next prove bounded nested work in the
non-exiting sibling field arm under the same owner/child authority. Common
tail choices, deeper/multiple source exits, branch-local exports, resources
and broader roots require separate proofs. No local CPU checkpoint certifies
GPU/Linux/Windows execution, heterogeneous application closure, performance
measurement or completed self-hosting. Whole-workspace/strict-Clippy
acceptance is not claimed.
