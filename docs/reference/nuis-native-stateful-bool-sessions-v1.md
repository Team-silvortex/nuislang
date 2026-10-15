# Native Stateful Bool Sessions V1

This CPU checkpoint follows the
[sealed-binding preflight repair](nuis-native-sealed-binding-preflight-v1.md).
It demonstrates real previous-State consumption by composing existing
read-only field access, ordinary bool const staging and complete bool-return
trees. It does not add a new compiler rule, runtime ABI or field-write authority.

## Source Computation

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_stateful_bool.ns)
opens from the input flag, but step passes state.result into observe instead
of the event flag. All four return routes depend on that previous result:
the yes/first route combines it with a transformed i64 owner; yes/other flips
it through a sealed initializer; no/first flips it with observable unused work;
no/other retains it. The result is placed in a new State value, not written
in place through an entering aggregate or sealed binding.

Seven ordinary bool constants remain in ancestor/leaf scopes. Original,
mirrored and logically wrapped original/mirrored sources are four distinct
variants. Inactive argument-proof operands are zero. Existing exact type,
scope, seal, selected-only effect, guard and source-budget proofs remain in
charge of admission.

[AOT checks](../../tools/nuisc/src/aot_application_effect_call_return_stateful_bool_tests.rs)
require State{result:bool}, nonempty helper coverage, guard-first transitive
Effect ordering, explicit print policy and no owned aggregate allocator/drop.
Removing effect order or print permission rejects under existing validation.

## State Witnesses

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_stateful_bool_cases.rs)
describes the source result as a bool parity operation, not by interpreting
lowered code. Sixteen seed/gate/child/initial-flag cases each open once and run
five events. The first event flips the state. The next event must therefore
distinguish the latest state from open/one-event-stale state.

Two adjacent target events have exactly identical inputs, and a later target
is repeated after a forced flip. For every case, at least one such repeated
input changes result. Replacing previous state with the event flag, constant
false or constant true also changes a result in every sequence. Both owner
seeds additionally distinguish the real +1 computation from the stale owner.
These counterfactuals are arithmetic witnesses, not mutated-compiler runs.
The reference/native and CLI executions independently check actual results.

[Reference/native tests](../../tools/nuisc/tests/native_application_bridge/return_stateful_bool.rs)
run normal/reversed YIR inventories. Reference failures occur after two
accepted events, whose last state differs from open. A selected division trap
retains that last state, enters Faulted, refuses later events and cannot report
successful completion even after close.

[The shared word probe](../../tools/nuisc/tests/native_application_bridge/word_session_probe.rs)
now accepts multi-event Sequence and SequenceFailure inputs. Its old pair
adapter remains for predecessor fixtures. Before every event/close it copies
the actual previous callback output into the input state words; expected
states are used only for output comparison, never injected into a callback.
Each accepted callback checks status, exact state and both output canaries.
Instrumentation observes real answer_work owner/bool/proof arguments and
flushes prints without replacing source computation. It does not probe every
callee or claim a general memory-safety proof.

The new native family runs eight healthy processes and 64 fatal processes:
896 healthy accepted callbacks plus 192 accepted callbacks preceding failures.
All eight independently selected argument boundaries retain exact failure
prefixes and never reach a post-failure status/state/canary publication.

## Source-Free Delivery

[The CLI workflow](../../tools/nuis/tests/native_session_workflow/effect_call_return_stateful_bool.rs)
checks sixteen images: four source variants, each fresh/cache/two source-free
restores. Two independent healthy sessions each open, execute 40 events and
close, covering 80 events while respecting the existing 64-event script bound.
Each image also checks that 65 events reject before open; no runtime bound is
raised to accommodate the fixture. The original single 80-event test was
correctly rejected and is not counted as acceptance. The independent oracle
tracks the expected prior state, checks repeated identical
inputs with different results, and compares every published state and print.

Each image runs both healthy sessions before and after four selected event
traps. Faulting sessions open false, accept two events ending true, then trap;
only those three accepted states may be published, with no completion or later
event/close state. Fatal CLI stdout may be buffered: exact failure-prefix
evidence comes from the separately flushed native probe.

This family contributes 2880 accepted CLI callbacks and 64 fatal CLI events,
plus sixteen oversized-script rejections before any callback.
Fresh/cache executable, native objects, envelope, policy and lifecycle remain
identical within each source variant. Execution leaves payload/container bytes
unchanged. Source, manifest and project cache are removed before both restores,
which leave the input artifact unchanged. Distinct variants need not produce
byte-identical binaries.

## Acceptance And Lineage

The serial macOS CPU cohort passes 497 distinct tests: 450 compiler, fifteen
native/reference/oracle, two CLI, three host and 27 tensor. Five tests are new:
one AOT, three oracle/reference/native and one CLI. The initial missing
test-only collection import and oversized CLI fixture are corrected before
accepted reruns; the 64-event runtime bound is unchanged. Focused reruns
are not extra distinct tests. Twelve native predecessors replay through the
pair adapter. Only the ordinary-const CLI predecessor is freshly replayed;
other historical CLI receipts are not included in this cohort. The earlier
combined CLI attempt passed the ordinary-const control before rejecting the
oversized new fixture; the corrected bounded fixture then passed its focused
replay. Accepted distinct tests are counted once across these runs.

Run serially from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields return_in_fields return_nested_in_fields return_prefix_in_fields return_sibling_in_fields return_tail_in_fields return_child_tail_in_fields return_pair_tail_in_fields return_triple_tail_in_fields return_quad_tail_in_fields return_quint_tail_in_fields return_paired_in_fields return_exit_prefix return_typed_exit_prefix return_plain_prefix return_plain_tree return_plain_const return_stateful_bool --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_paired_in_fields return_exit_prefix return_typed_exit_prefix return_plain_tree return_plain_const return_stateful_bool --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_plain_const return_stateful_bool --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

All 3192 source-drift checks /22255 patterns, UTF-8, changed file caps/relative
links, changed Rust formatting and diff checks pass. Four untouched historical
files retain their whole-workspace formatting failures; strict Clippy and
full-workspace tests were not run. No Linux/GPU/Windows or measured-performance
acceptance is implied.

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
The six complete preceding sealed-binding fields remain unabridged with fresh
SHA-256 hashes. The full validation history is enclosed in a collision-checked
quoted no-op heredoc. Shell syntax and a harmless-front replay verify that the
historical commands remain literal data, not commands executed again.

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `c92d9d5c0cdd2472fe32f0082d29aeb079284bf72ce4dee2c8f0a3beafeee72c` |
| `next_step` | `8c5b74565977d81d46649057f3107f1a269425be353073f2dfde4d093a400a60` |
| `blocker` | `2f866242677b78b363bf7181a2769feca623ad2975f529d10ca5772afc288803` |
| `next_action` | `16af85ab3cba91323cea4e361d8eb7073e5581a2ebc87e5eb2741534f4928cd2` |
| `validation_command` | `eb823444d20f0d951501826613835dd7636e19cfdc6e9f6879a2bcfd648b85b9` |
| `expected_artifact` | `976d378e51310892b89f3bedab37d13c13eb09ffe381d0727ea3e415227c0eeb` |

The rebuilt ordinary CLI exposes six source-equal fields and six matching
complete predecessor hashes; coverage, manifest/milestone coverage, hierarchy,
task-card lineage and drift are clean. This local proof remains active/99,
not a global completion or generalized heterogeneous-state claim.

## Next Boundary

Extend this bounded read-only session computation to a small typed multi-field
State. Make a later field depend on a newly computed earlier value, preserve
untouched fields bit-for-bit and compare an independent latest/stale-state
oracle across repeated events and selected failures. Do not widen inherited
root/field write authority or conflate reconstructing an owned State with
in-place mutation. Resource transport, arbitrary deeper exits, global const
verification, stable ABI, formal safety, GPU/Linux/Windows, performance and
completed self-hosting remain separate.

The [typed state reconstruction receipt](nuis-native-stateful-field-reconstruction-v1.md)
follows this historical next-boundary task with mixed-type state computation
and exact untouched floating words, without expanding entering-write authority.
