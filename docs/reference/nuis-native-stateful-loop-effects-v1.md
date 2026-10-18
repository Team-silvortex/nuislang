# Native Stateful Literal-Print Iteration Effects V1

Historical return-in-loop exclusions below describe this checkpoint. The later
[effectful loop-return receipt](nuis-native-stateful-loop-returns-v1.md)
advances that boundary through explicit literal-print typed transport and
native/source-free proof, not general I/O or pure-function reclassification.

This bounded CPU receipt follows the
[pure carried-loop proof](nuis-native-stateful-bounded-loops-v1.md).
The starting Git baseline remains beta-0.17.1, commit 99ebfcb2.
The new production route admits a narrowly specified iteration-call capability,
not general I/O and not a relaxation of pure-value discovery.

## Separate Shape And Effect Authority

[The isolated plan](../../tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects.rs)
owns a private signature table and selects only non-pure functions containing
counted loops. It reuses completed dependency, exact type, scope, carry, induction
and guarded-control validation. Its separate profile permits only direct NIR
i64 literal print statements in synchronous scalar/value callees. The real source
statements are retained; no shadow source erases effects or labels them pure.

Pure value/capture/selection collectors still explicitly use the pure profile.
The effect table never seeds them. It is used only for real call retention and
selected loop/control outlining. Already-normalized dependencies are not
reprocessed against stale signatures. Functions with return-in-loop effects and
their dependent callers are excluded from this route. A reverse dependency walk
propagates that veto without recursive graph traversal.

[Profile tests](../../tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects_tests.rs)
check unchanged source during discovery, pure exclusion before and after
outlining, real print retention, rejected computed/non-i64 prints, cycles,
unknown callees, invalid call kinds/arity and direct loop-body effects.
Deliberately unvalidated NIR dependencies cannot gain provisional authority.
Existing expression, lexical, capture, return and graph budgets are unchanged.

This is shape/effect classification, not backend permission. The existing
explicit literal-print policy still validates every selected YIR site, its
direct i64 constant, exact closure and dependency order. No new backend policy,
ABI, helper-name whitelist, allocation path or entering-State write authority
is introduced.

## Real Selected Source Effects

[The shared Nuis source profile](../../tools/nuisc/tests/native_application_bridge/state_loop_effects_profile.rs)
extends the previous four original/mirrored plain/+0-wrapped sources with
source prints 10/19/20. The condition callee prints 10 after its proof argument
succeeds. A selected helper branch prints 19 before evaluating its checked work
argument; the actual work callee then prints 20. The existing advance/finish/
answer prints 5/35/40 remain. None of these six source effects is synthesized
by the native observer.

The prior independent arithmetic oracle is reused unchanged: 48 signed
seed/gate/flag/trip cases, four raw markers and two proof modes give 384
three-event sequences per source/inventory order. Dynamic negative/zero trips
perform no iteration effects; one and three trips consume current values.
All-valid complete event arguments repeat. Separate state-aware poisoned
holes at indices zero, one and two must not evaluate a skipped zero divisor.
Their proof masks and valid arithmetic may differ from all-valid inputs.

[Reference/native execution](../../tools/nuisc/tests/native_application_bridge/state_loop_effects.rs)
checks actual prior callback output handoff, not expected State injection.
Native observers record only actual refine count/proof/index and final answer
arguments. Print markers now come from Nuis source. Normal/reversed YIR
inventories preserve status, canaries, result words and exact once-only traces.
Positive/negative zero, minimum subnormal and a signed quiet-NaN marker payload
retain exact State bits.

Eight main native healthy processes contribute 15360 healthy accepted callbacks.
Thirteen arithmetic traps per source/inventory order contribute 104 fatal
processes and 312 earlier accepted callbacks. Selected hole failures at indices
zero, one and two occur after print 19 but before the work callee/print 20;
earlier completed iterations remain visible. Condition, advance, common-tail
and answer failures retain their precise earlier prefixes. Source effects are
not rolled back; publication of a new State waits for successful callback return.

The prior accepted prefix remains State(12,true), State(16,false), State(21,false),
with the opening negative-zero marker. Reference faults retain that complete
last State, reject later events and cannot complete successfully after close.
Native fatal runs establish flushed prefixes and no later publication, not
post-abort memory inspection.

## Explicit Budgets And Delivery

The main policy remains three loop reservations and 100 helper entries per
callback. Separate native limits zero/two/three admit zero trips and reject
over-budget counts, 65537 and i64::MAX after advance but before any iteration
print. Negative inputs deliberately contain valid root/work arguments, so an
ordinary argument trap cannot masquerade as budget rejection. These add six
healthy processes /30 accepted callbacks and 24 fatal processes /24 earlier
accepted callbacks. Reservation reset, independent non-wrapping induction,
the existing ten-second native child deadline and the 64-event script bound
are unchanged.

[AOT checks](../../tools/nuisc/src/aot_application_effect_call_state_loop_effects_tests.rs)
require exact State layout, preserved runtime loop structure and no owned
aggregate allocation. Omitting any single print grant rejects, as does the
default ungranted mode or removing Effect ordering. The previous compile-time
print rejection control now checks the backend permission boundary; the new
pure-profile tests separately retain its original purity invariant.

[The shared delivery workflow](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop.rs)
runs both the existing pure family and the new real-effect family through
the same artifact lifecycle. Each family checks sixteen images:
four sources, each fresh/cache/two source-free restores. Each image runs four
healthy three-event sessions before and after seven selected hole/tail traps.
It checks exact source stdout, three prior published states before a fatal
event, no failed/next/close state and no successful completion afterward.

Per family this is 640 healthy accepted CLI callbacks, 336 prior accepted
callbacks, 112 fatal events and sixteen oversized-script rejections before
open. The two families together cover 32 images. Four finite CLI marker/trip/
proof profiles are not a full raw-marker cross-product; printed markers are
parsed back and compared by bits. CLI does not certify NaN payload input or
fatal partial stdout; the flushed native probe supplies prefix evidence.

Cache checks compare executable, host objects, envelope, lifecycle and exact
budget-bearing packaging mode. Source, manifest, project cache and original
output are removed before restoring twice. Sources remain absent; restores
and executions leave artifact/container/executable bytes unchanged.

## Acceptance And Lineage

The serial selected macOS CPU cohort passes 659 distinct tests: 609 compiler,
eighteen native/reference/oracle, two CLI, three host and 27 tensor. Eight tests
are new: three profile, one AOT, three native/reference/budget and one CLI.
Focused reruns and child processes are not counted as extra tests. Compiler
coverage includes the existing control-loop, helper-purity and scalar-control
cohorts. Native compatibility replays sequential updates, joins, skipped-arm
retention, pure carried loops and the typed exit-prefix paired-row adapter.
Other older CLI receipts remain historical.

Run serially from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields return_in_fields return_nested_in_fields return_prefix_in_fields return_sibling_in_fields return_tail_in_fields return_child_tail_in_fields return_pair_tail_in_fields return_triple_tail_in_fields return_quad_tail_in_fields return_quint_tail_in_fields return_paired_in_fields return_exit_prefix return_typed_exit_prefix return_plain_prefix return_plain_tree return_plain_const return_stateful_bool return_stateful_fields return_state_routing state_join state_skip state_sequence state_loop iteration_effects control_loops scalar_helpers scalar_control --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_typed_exit_prefix state_join state_skip state_sequence state_loop:: state_loop_effects:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- effect_calls::state_loop:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

All 3288 drift checks /22759 patterns, changed Rust formatting, UTF-8,
changed caps/relative links and diff checks pass. Four untouched historical
whole-workspace formatting differences remain. Strict Clippy/full-workspace
tests and fresh Linux/GPU/Windows/performance acceptance are not claimed.

The coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
All six complete preceding values remain unabridged with the following hashes.
Command history is a collision-checked quoted no-op heredoc; syntax and harmless
current-front replay verify that prior commands remain nonexecuting data.

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `7d6ba52ddfd3f9219668b9af8cbdf22636d1f93e5e5ca15d05a6b5f7adcb897a` |
| `next_step` | `baa5754bacf37b88cd2b9e33ea9ac200afa6f0f50bec3ba953a9fb7be18296de` |
| `blocker` | `0faf9c445d13c5e70800455a0d1862e308519ee5a259e9ad0fe7a9050e433455` |
| `next_action` | `739ff2fb08ea725a3e4ba74322232a46bb69c52e46c2d585d1c064959575347f` |
| `validation_command` | `81d8d7506db8b34c35c89ba7624b8cc2061f1b3189376538553450b3696357e2` |
| `expected_artifact` | `c4e3191048b8a53992015d9b31acbd67c16e9f71217055f1e7672cae0bf5b252` |

The rebuilt ordinary CLI agrees with all six source fields and predecessor
hashes. Coverage, manifest/milestone coverage, hierarchy, task-card lineage,
the selected ready task card and drift remain clean.

## Next Boundary

The [direct loop-statement follow-up](nuis-native-stateful-loop-statements-v1.md)
now advances this historical boundary with explicit authority, print-only
execution and cumulative-budget evidence. Its receipt records the current
cohort; the counts above remain this checkpoint's historical acceptance.

Admit direct literal statements in the loop body through an explicit effect-aware
iteration validator, including print-only and selected statement paths, rather
than forcing an artificial source helper or treating prints as pure. Preserve
zero-trip non-evaluation, exact grant/order checks, callback-local work limits,
current-value carries, failure-before-publication and source-free identity.
General I/O, asynchronous resources, return-in-loop effect closures, generalized
entering-State writes, global immutability, stable ABI, formal safety, measured
performance, generalized GPU closure and completed self-hosting remain separate.
