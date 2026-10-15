# Native Stateful Field Reconstruction V1

This bounded CPU checkpoint follows the
[state-dependent bool session receipt](nuis-native-stateful-bool-sessions-v1.md).
It composes existing read-only state extraction, typed scalar computation and
owned State reconstruction. No production compiler special case, runtime ABI,
allocation mechanism or entering-field/root write authority is introduced.

## Dependent Values And Retained Bits

[The Nuis source](../../tools/nuisc/tests/native_application_bridge/return_stateful_fields.ns)
uses State{result:bool,total:i64,marker:f64}. An effectful advance call computes
the new total from State.total, then observe consumes that new value and the
previous State.result. Its selected answer route checks owner parity, exposing
use of the stale count instead of the newly computed count. State.marker is
read and placed unchanged into the newly constructed owned State.

Event inputs are not the state source, and the event marker deliberately
differs from the retained marker. Step must not substitute seed/flag/marker
arguments for previous fields. No in-place field update
is implied by constructing a new result. Seven ordinary bool constants and
the complete return tree retain their existing scopes, seals and budgets.
Four plain/wrapped original/mirrored sources share this behavior.

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_stateful_fields_cases.rs)
tracks bool parity, the monotonic i64 count and raw f64 bits. Sixteen source
cases cross four raw markers: positive zero, negative zero, the minimum
positive subnormal and a signed quiet-NaN payload. Every event advances the
count once. On the answer route, using the previous count flips the expected
bool result. Using the wrong prior bool, stale/open count or event marker is
also observably wrong. These are arithmetic counterfactuals, not compiler
mutation runs.

[AOT checks](../../tools/nuisc/src/aot_application_effect_call_return_stateful_fields_tests.rs)
require the exact three-field layout, guard-first private helpers, transitive
Effect order in helpers and both source entry functions, explicit print policy
and no owned aggregate allocation/drop. Entry calls start with i64 work before
bool work. The first test incorrectly assumed exactly two calls; valid expanded
bool calls made it fail. The corrected assertion checks semantic ordering and
types without fixing a helper count. Removing Effect order or print permission
still rejects under the existing verifier.

## Reference And Native Execution

[Reference/native tests](../../tools/nuisc/tests/native_application_bridge/return_stateful_fields.rs)
exercise 64 case/marker combinations, five events after each open and close,
under normal/reversed YIR inventories. Every published state is compared as
three raw words, avoiding NaN value equality and signed-zero normalization.
The native driver copies actual callback output into the next input through
the existing generic sequence probe, never expected golden state values.
Canaries and real selected answer-callee owner/bool/proof arguments are checked.

The new family contributes eight healthy native processes and 72 fatal ones:
3584 healthy accepted callbacks plus 216 accepted callbacks before failures.
There are nine independent argument traps, including the advance argument
before any print. Later selected failures happen after advance has completed
locally but before a new State can be published. Two earlier events leave a
different bool/count from open while retaining negative-zero marker bits.
Reference failures retain that complete last state, enter Faulted, refuse
later events and cannot report successful completion after close.

Fatal native probes establish exact flushed prefixes and absence of later
status/state/canary publication, not post-abort memory inspection. Ordinary
CLI fatal stdout can be buffered and is not treated as prefix evidence.

## Source-Free Scope

[The CLI workflow](../../tools/nuis/tests/native_session_workflow/effect_call_return_stateful_fields.rs)
checks sixteen images: four source variants, each fresh/cache/two source-free
restores. Four independent 20-event sessions cover all sixteen source cases
with round-trippable positive/negative zero, subnormal and finite markers.
Every published marker is parsed back to f64 and its bits compared. This is
decimal-input coverage, not raw NaN payload coverage through the CLI.

Each image runs those sessions before and after five selected event traps,
including advance. Only the three previously accepted states may be published;
no failed/next event or close state/completion is accepted. A 65-event script
still rejects before open under the unchanged 64-event bound.

This family contributes 3056 accepted CLI callbacks, 80 fatal events and sixteen
oversized-script rejections. Fresh/cache executable/native-object/envelope/
policy/lifecycle identities, unchanged execution bytes and both source-free
restores are checked independently for each source variant. Source, manifest
and project cache are absent before restoration; the input artifact is unchanged.

## Acceptance And Lineage

The serial macOS CPU cohort passes 501 distinct tests: 451 compiler, eighteen
native/reference/oracle, two CLI, three host and 27 tensor. Five tests are new:
one AOT, three oracle/reference/native and one CLI. The failed fixed-call-count
AOT attempt is corrected before acceptance; focused reruns are not additional
distinct tests. Native predecessors replay through the same sequence probe.
Only the stateful-bool CLI predecessor is freshly replayed; other CLI receipts
remain historical rather than being included in this cohort.

Run serially from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields return_in_fields return_nested_in_fields return_prefix_in_fields return_sibling_in_fields return_tail_in_fields return_child_tail_in_fields return_pair_tail_in_fields return_triple_tail_in_fields return_quad_tail_in_fields return_quint_tail_in_fields return_paired_in_fields return_exit_prefix return_typed_exit_prefix return_plain_prefix return_plain_tree return_plain_const return_stateful_bool return_stateful_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_paired_in_fields return_exit_prefix return_typed_exit_prefix return_plain_tree return_plain_const return_stateful_bool return_stateful_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_stateful_bool return_stateful_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

All 3205 source-drift checks /22332 patterns, UTF-8, changed file caps/relative
links, changed Rust formatting and diff checks pass. Four untouched historical
files retain their whole-workspace formatting failures; strict Clippy and
full-workspace tests were not run. No Linux/GPU/Windows or measured-performance
acceptance is implied.

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
The six complete preceding stateful-bool fields remain unabridged with fresh
SHA-256 hashes. Full command history is enclosed in a collision-checked quoted
no-op heredoc. Shell syntax and a harmless-front replay establish that nested
historical commands remain literal data rather than executing again.

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `21b3886974ab08f2bc5324a04176bd10089c536f11205b23f91c4dc9bb5a7651` |
| `next_step` | `cc09d413e40a56b7846d42a45a483b451393b5ea608e9263368cd6878586825c` |
| `blocker` | `6b19c5e6eb9e360d9c969df2a4f633a363ba0c5ed9261dcea23694ef1b976247` |
| `next_action` | `9886e12ed57deae028728f502117cfb3f1a045379720fcfa69a13d8289e0018b` |
| `validation_command` | `c819212082711bd1f2637f48117242bcaf35372a52de574bced6a476880cd97f` |
| `expected_artifact` | `992fb1dda83a34906b6fb3cd04671794c0e595c5f55915baa0f2b1c8e02d38be` |

The rebuilt ordinary CLI exposes six source-equal fields and six matching
complete predecessor hashes. Coverage, manifest/milestone coverage, hierarchy,
task-card lineage and drift are clean. Active/99 is local, not global completion.

## Next Boundary

That checkpoint is followed by the
[state-derived branch selection proof](nuis-native-state-derived-branch-selection-v1.md);
the paragraph below records this receipt's original next boundary.

Prove branch selection derived from previous state, not only state-dependent
returned values. Make identical event inputs select different work after prior
state changes, keep inactive argument proofs poisoned, and verify both branch
traces plus selected traps before publication across repeated events and
source-free delivery. Retain exact scalar types, read-only captures, current
guard/effect authority and original bounds. Resource transport, generalized
in-place writes, arbitrary deeper exits, global const verification, stable ABI,
formal safety, GPU/Linux/Windows, performance and self-hosting remain separate.
