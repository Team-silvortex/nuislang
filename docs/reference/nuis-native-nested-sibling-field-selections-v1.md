# Native Nested Sibling Field Selections V1

This bounded CPU checkpoint follows the
[complete 409-test predecessor](nuis-native-nested-pre-exit-field-selections-v1.md).
It permits bounded nested continuing work in the non-exiting sibling field
arm, beside the same single source bool exit and its prefix/remainder.

## Source Work And Authority

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_sibling_in_fields.ns)
retains one lexical i64 owner and private i32/f32/f64/two-bool children.
The original arm keeps the completed prefix and guarded exit/remainder.
In the sibling arm a once-only condition reads e: one leaf updates a/d,
the other updates c from current b/d. Both retain b/e, then update the
same numeric owner using completed current fields. The common tail remains
single and flat. No sibling work executes when the original exit arm wins.

The sibling arm contains no source return. Its current d/e can nonetheless
satisfy the exit predicate; the original saved arm gate must still skip
exit_work and its zero proof. This is a genuine poisoned-gate witness,
not a tautological condition or an oracle-supplied compiled value. Normal
false source returns still exit observe; start may publish the resulting
State. Selected proof traps preserve the last accepted callback instead.

Only the sibling shape checks change from flat to recursive continuing.
Whole-source preflight and both continuing forests around the exit remain
unchanged. Existing private_field_certificate discovers matching numeric
owner targets from both original arms and certifies private children.
A different sibling owner, entering writes, sealed children, wrong types,
future-name collisions and sibling/deeper/extra exits reject atomically.
Lexical leakage is rejected by the frontend before installation. Resources,
broader roots and general function eligibility do not become authorized.

The same recursive field transport stages either arm and retains untouched
values. Five data-only six-field records represent both prefix choices,
completed prefix, continuing choice and suffix. A once-bound original
selector gates the source exit, and one guarded bool continuation owns the
shared tail. Neither record control flags nor tail/call copying are added.
Completed field joins/projections legitimately occur before the exit;
unfinished remainder work and sibling source work stay behind their gates.

Original work is charged once: twenty-eight nodes and three logical edges.
Twenty-seven nodes, one fewer required expression or two logical edges
reject. Inherited depth five passes and six rejects. A deeper sibling
choice with an empty retained leaf uses twenty-nine nodes; depth four
passes and five rejects. Original forest/depth/expression/logical budgets,
exact lexical types and the 32-parameter capture limit do not restart.
The shared bool_work proof is used in disjoint original/sibling arms;
all newly introduced sibling proofs and unrelated original-arm proofs
are zero whenever their work is inactive.

## Execution And Delivery

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_sibling_in_fields_oracle.rs)
models integer complements, separate IEEE words, bool updates and source
event positions. Sixteen finite gate/bool cases plus sixty-four paired IEEE
cases give eighty per inventory. Paired signed zeros, subnormals, infinities
and quiet/signaling NaNs are included, not a full Cartesian product.
Public state has twenty-nine words, open twenty-eight arguments, event
twenty-two. The mirrored source swaps both real arms while inverting the
source condition, preserving the effective arm and expected behavior.

Normal/reversed function inventories and both source polarities run real
previous callback outputs with canaries. Independent modes observe actual
exit_work prefix or after_work completed-field arguments, plus answer_work
owner arguments; these are **not simultaneous observations** in a process.
Sibling a/d and c changes/retention are checked, including gate-false cases
whose current fields would otherwise exit. Marker-valued words exercise
source-position failure cuts, not truncation by an observed word value.

The new family contributes **640 successful native processes**,
**160 fatal native processes** and 2240 accepted callbacks per complete replay:
two sources * two inventories * two modes * eighty healthy/twenty failure
cases. Selected sibling condition, integer/float work, bool work and owner
checks trap before the failing callee, preserving exact earlier effects.
Focused repeats do not increase these totals.

Eight fresh/cache/two successive source-free images, four per polarity,
contribute **160 CLI event traps** and **1184 accepted callbacks**. Each image
runs sixteen finite cases and twenty failure/reentry cases. Source, project
manifest and cache are removed before both restores. Full state/stdout,
executable/native-object bytes, policy/envelope/lifecycle and container bytes
remain exact. IEEE claims belong to native/reference observers, not CLI text
parsing. AOT checks preserve transitive guard-first Effect order, explicit
print grants and no owned aggregate allocator/drop. Missing grants or Effect
paths remain vetoes; this is not formal memory-safety acceptance.

## Scoped Acceptance

The 2026-10-13 serial macOS CPU acceptance cohort has
**414 distinct tests**: 372 compiler, eight native/reference, four CLI,
three host and 27 tensor. Eleven are new: four units, one AOT, two reference,
two native and two CLI. The direct predecessor's native/reference and CLI
families are freshly replayed. Older runtime histories remain intact,
rather than being counted as fresh execution.

Run from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields return_in_fields return_nested_in_fields return_prefix_in_fields return_sibling_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_prefix_in_fields return_sibling_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_prefix_in_fields return_sibling_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

The selected cell stays
`standard-library/ns-nova/persistent-application-session`, `active/99`.
The rebuilt ordinary CLI exposes six source-equal current fields and six
complete predecessor hashes, 2996 clean drift checks / 21000 patterns, and
clean coverage/hierarchy/task-card lineage. Changed Rust formatting, UTF-8,
file caps and relative links are clean. Whole-workspace formatting retains
four failures in untouched historical files; it is not a full-workspace pass.
All six complete predecessor strings remain verbatim after their nested
pre-exit selection history markers, not replaced with hash-only summaries.

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `3e6efde4226eb9712fba7337f258dcc54b6899b3b7244adc5e615530b56ea314` |
| `next_step` | `5e0735f0c77b55082cb36d0fc876a9ce1cea64557e9af5e08d0c2d229798717d` |
| `blocker` | `55a2518cd5be2489f5147690173f2ea515a59cd40755a460cbfbbdbad617b3d5` |
| `next_action` | `164a682fa6d77f5510ffa91179f2166c75d94726066480cf1db6fdb1e874a4ae` |
| `validation_command` | `7e2af13e3d041a341e01ea9a82acf8b275b2d76becbff40dfca311c7086b0bb6` |
| `expected_artifact` | `9a1a400b56cd93f196c712d1f2d1cc862575230f13a1b1b4376664fc1bf9522e` |

## Remaining Boundary

One original exit plus bounded continuing work before/after it and in its
sibling field arm is supported; multiple exits remain separate. Next prove
bounded owner-only nested choices in the common continuing tail, without
granting common-tail child updates or copying that tail into both arms.
Deeper/multiple source exits, child-scope exports and resources/broader roots
remain separate. This local CPU proof does not certify GPU/Linux/Windows
execution, heterogeneous application closure, performance, completed
self-hosting or whole-workspace/strict-Clippy acceptance.
