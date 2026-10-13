# Native Certified Child Tail Selections V1

This bounded CPU checkpoint follows the
[complete 420-test predecessor](nuis-native-nested-owner-tail-selections-v1.md).
It permits one certified child binding to advance in nested selections
within the single common continuing tail of one in-field source bool exit.

## Certificate And Original Work

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_child_tail_in_fields.ns)
retains the i64 owner and private i32/f32/f64/two-bool fields. A tail
condition reads completed fields; its nested selected leaf flips the f64
child c and updates the same owner. The empty leaf retains both. The
other branch updates only the owner. Later after_work and parent_work read
the completed tail fields, and answer_work observes the final owner.

The original PrivateFields certificate remains the only authority source.
A bounded scan of the complete shared tail excludes owner writes, rejects
any other bound name outside that certificate, and permits at most one
distinct certified child. Each child-writing selection must match this same
child. It reuses existing field staging/transport; selections without child
writes still use the exact-owner adapter. Flat child writes remain rejected.
Fresh lexical names cannot become additional certified children.
No root/bool write authority is rediscovered from current values.

Existing seal/type/lexical/future-name checks remain in the field proof.
Unit variants cover independently selected i32, f32 and both bool children;
native/reference/CLI execution in this new family covers f64. Two different
children in the same or separate tail choices, entering writes, flat child
writes, const rebindings, additional returns, lexical leakage and future-name
collisions reject atomically. Resources and broader roots remain separate.
Fresh scalar lets and logical bool constants retain their inherited staging
rules; non-logical scalar constants are not newly authorized.

Seven data-only six-field records replace the predecessor's five: two new
nested tail selections carry current/retained values. One guarded bool
continuation still owns the common tail. Neither control fields nor copied
calls/tails are introduced. The complete fixture consumes thirty-three
original nodes and three logical edges. Thirty-two nodes, one fewer required
expression or two logical edges reject. Inherited depth four passes and five
rejects; a deeper sibling choice requires thirty-four nodes. Whole-source
preflight, lexical types and the 32-parameter capture limit do not restart.

## Current-Value Witness And Execution

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_child_tail_in_fields_oracle.rs)
keeps separate before_tail and completed-field tuples. A finite witness has
first=true, second=false, bool_seed=true and peer_seed=false. Its changed c
reaches the final owner calculation, yielding the correct true bool.
Retaining stale c instead flips that owner away from seed and gives false;
e is false, so it cannot mask the wrong result. Native after_work arguments
and answer_work owner arguments independently verify the actual compiled
data, rather than inserting oracle values into the program.

Sixteen finite gate/bool cases and sixty-four paired IEEE profiles cover
signed zeros, subnormals, infinities and quiet/signaling NaNs, not a full
Cartesian product. Open has thirty-two arguments, state thirty-three words,
and event twenty-six arguments. Inactive original/sibling/tail/child proofs
remain zero. Both real source-arm polarities and normal/reversed inventories
use real previous callback outputs, canaries and source-position failure
cuts, including raw words equal to stage markers. Exit-work prefix and
after-work completed-field modes are **not simultaneous observations**
in a process. A false source bool still exits observe, skipping all tail work.

The new family contributes **640 successful native processes**,
**192 fatal native processes** and 2304 accepted native callbacks per full
replay: two sources * two inventories * two independent modes * eighty
healthy / twenty-four selected failure cases. The direct predecessor is
freshly replayed separately; focused repeats do not increase totals.

Eight fresh/cache/two successive source-free images contribute
**192 CLI event traps** and **1344 accepted callbacks**. Each image checks
sixteen finite healthy cases, twenty-four selected failures and healthy
reentry. Source, project manifest and cache are absent before both restores.
Executable/native objects, policy/envelope/lifecycle, container bytes, full
state and healthy stdout remain exact. Fatal CLI stdout can be buffered;
partial-effect order belongs to the independently flushed native observer.
AOT retains transitive guard-first Effect order, explicit print grants and
no owned aggregate allocator/drop. This is not formal memory-safety acceptance.

## Scoped Acceptance

The 2026-10-13 serial macOS CPU cohort passed **426 distinct tests**: 384 compiler, eight
native/reference, four CLI, three host and 27 tensor. Twelve are new:
five units, one AOT, two reference, two native and two CLI.
The direct predecessor's native/reference and CLI routes are freshly replayed.
Older runtime checkpoints remain historical, not fresh execution.
The final CLI oracle audit was rebuilt and repeated after its before_tail
field was connected to the stale-value witness, resolving the new unused-field
warning without suppression. This focused repeat does not increase the count.

Run from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields return_in_fields return_nested_in_fields return_prefix_in_fields return_sibling_in_fields return_tail_in_fields return_child_tail_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_tail_in_fields return_child_tail_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_tail_in_fields return_child_tail_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

The selected cell stays
`standard-library/ns-nova/persistent-application-session`, `active/99`.
All six complete predecessor strings remain after owner-only common-tail
history markers, with fresh SHA-256, not hash-only summaries.
The rebuilt ordinary CLI exposes six source-equal fields and six matching
complete predecessor hashes. All 3028 drift checks / 21193 patterns and
coverage, hierarchy and task-card lineage are clean. Changed Rust formatting
passes for 86 files; UTF-8, file caps, relative links and diff whitespace pass.
Whole-workspace formatting still reports four untouched historical files,
so this is not whole-workspace formatting or strict-Clippy acceptance.

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `e498d601274f2da727a6a026c5d1c192250b24acfa3701a6083d59a91a203030` |
| `next_step` | `5faf9f32ef27be6dd31c41180a0f49dae1179f8b9acd7d3022695be551300dc5` |
| `blocker` | `e535d0fdb91b80e46ae97c0841465d8fd2ed5f03c517f7e618ac23d6e884d5b6` |
| `next_action` | `cd6b924cc2ee270497a41f6719149f097b61c06564064ce5cb7f17ee14caac48` |
| `validation_command` | `322fb63f9c6b5f84226aea83351246cffe2b72ddd5f509308ed2639259c54078` |
| `expected_artifact` | `803453a314fe2181162f33eaa038eb7f4ca49e6bfd3a4c10305cf035e4b00baf` |

## Remaining Boundary

One certified child binding may be written in nested common-tail selections;
two distinct tail children and multiple exits remain separate. Next prove
two independently updated child bindings in that same tail, with current-value
interaction, retained empty leaves and the original certificate, without
copying work, exporting arm-local values or widening entering writes.
Resources, deeper/multiple source exits and broader roots remain separate.
This scoped CPU proof does not certify GPU/Linux/Windows execution,
heterogeneous application closure, performance, completed self-hosting or
whole-workspace formatting/strict-Clippy acceptance.
