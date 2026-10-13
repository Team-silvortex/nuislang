# Native Nested Owner-Only Tail Selections V1

This bounded CPU checkpoint follows the
[complete 414-test predecessor](nuis-native-nested-sibling-field-selections-v1.md).
It adds nested owner-only choices in the single common continuing tail,
after the same one in-field source bool exit.

## Authority And Original Work

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_tail_in_fields.ns)
keeps an i64 owner and private i32/f32/f64/two-bool fields. The completed
current fields reach a once-only tail condition. Its selected branch has
a second bool choice with an empty retained leaf; the other updates the
same numeric owner. The final common work sees that owner. An original
source exit skips the complete tail, including its conditions and proof
divisions. A false returned bool is still a source exit, not a continuation.

The new tail adapter validates writes independently of generic bool-export
discovery. Every bound name except the exact certified numeric owner stays
read-only. Entering variables, private bool/numeric fields, earlier tail
bindings, sealed owners and const rebindings cannot acquire write authority.
Fresh arm-local bindings stay isolated. Return/resource/loop statements are
not continuing choices. The generic branch proof then charges original
source work with the inherited budget and exact owner, rather than selecting
another export. Earlier completed-field flat-tail eligibility is unchanged.
The inherited staging route permits fresh scalar lets and logical bool
constants; fresh non-logical scalar constants remain a separate boundary.

Five data-only six-field records are retained. One guarded bool continuation
owns the shared tail; no control fields, replayed calls or copied tails are
introduced. The complete fixture uses thirty-two original nodes and three
logical edges. Thirty-one nodes, one fewer expression or two logical edges
reject; inherited depth four passes and five rejects. A deeper sibling
selection requires thirty-three nodes. Whole-source preflight, seals, lexical
types and the 32-parameter capture bound remain in force.

## Execution And Delivery

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_tail_in_fields_oracle.rs)
covers sixteen finite gate/bool cases and sixty-four paired IEEE profiles:
signed zeros, subnormals, infinities and quiet/signaling NaNs. These are not
a full Cartesian product. Open has thirty-one arguments, public state
thirty-two words and event twenty-five arguments. Inactive original-arm,
sibling and common-tail proof operands remain zero.

Actual compiled callee arguments are independently observed at exit_work
and tail_work, with answer_work owner observations in both modes. They are
**not simultaneous observations** in a process. Source-position failure
cuts remain independent of raw words that equal event markers. Both real
source-arm polarities and normal/reversed inventories use previous callback
outputs, boundary canaries and flushed fatal traces.

The new family contributes **640 successful native processes**,
**184 fatal native processes**, and 2288 accepted native callbacks per
complete replay: two sources * two inventories * two independent modes *
eighty healthy / twenty-three selected failure cases.
The direct predecessor is freshly replayed separately.

Eight fresh/cache/two successive source-free images contribute
**184 CLI event traps** and **1304 accepted callbacks**. Each image runs
sixteen finite cases, twenty-three selected failures and healthy reentry.
Source, manifest and project cache are absent before both restores.
Executable, native objects, policy/envelope/lifecycle, container bytes,
full state and healthy stdout remain exact. Fatal CLI stdout can be buffered;
exact partial-effect order belongs to the independently flushed native
observer. AOT retains transitive guard-first Effect order, print grants and
no owned aggregate allocator/drop. This is not formal memory-safety acceptance.

## Scoped Acceptance

The 2026-10-13 serial macOS CPU cohort passed **420 distinct tests**: 378 compiler, eight
native/reference, four CLI, three host and 27 tensor. Twelve are new:
five units, one AOT, two reference, two native and two CLI.
The direct predecessor is freshly replayed for native/reference and CLI;
older runtime checkpoints remain historical, not fresh execution.

Run from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields return_in_fields return_nested_in_fields return_prefix_in_fields return_sibling_in_fields return_tail_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_sibling_in_fields return_tail_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_sibling_in_fields return_tail_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

The selected cell remains
`standard-library/ns-nova/persistent-application-session`, `active/99`.
All six complete predecessor strings remain after nested sibling selection
history markers, with freshly calculated hashes, not hash-only summaries.
The rebuilt ordinary CLI exposes six source-equal fields and six matching
complete predecessor hashes. All 3012 drift checks / 21095 patterns and
coverage, hierarchy and task-card lineage are clean. Changed Rust formatting
passes for 79 files; UTF-8, file caps, relative links and diff whitespace pass.
Whole-workspace formatting still reports four untouched historical files,
so this is not a whole-workspace formatting or strict-Clippy acceptance.

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `dfc68db06f804180574868ef42b5a64e573dc1a25fb1feac4c4feba5f85cebe4` |
| `next_step` | `b7069b4e53d9af671254e3f976ab4d15ae585cb6455d77b8a511a5e1181b2dcf` |
| `blocker` | `16fe515b4811b731db22d9640340fe83d12e3497548cc708e80582d7a4ccbaa3` |
| `next_action` | `7b662cef939c66b13018d7479fa3b6c057f34a1cb1aba1b0aec96a33ae338582` |
| `validation_command` | `412c97c14d8ea1f857a93f6165367efd1adab62da472a2b969a1ce0264e5f5c8` |
| `expected_artifact` | `46edb2fb4df3e7bf2129a80b49b08a7e9a627e9c7893000db0f12ecc850192b2` |

## Remaining Boundary

Bounded owner-only common-tail selections now compose with continuing
pre-exit, post-exit and sibling fields. Common-tail child writes and
multiple exits remain separate. Next prove selected child updates in that
common tail using the same certificate, without granting entering writes,
exporting branch-local values, manufacturing exit authority or copying work.
Resources and broader roots remain separate. This local CPU receipt does
not certify GPU/Linux/Windows execution, heterogeneous application closure,
performance, completed self-hosting or whole-workspace/strict-Clippy acceptance.
