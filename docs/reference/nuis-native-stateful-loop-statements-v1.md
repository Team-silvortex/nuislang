# Native Stateful Direct Loop Statements V1

This bounded CPU receipt advances the
[iteration-call effects checkpoint](nuis-native-stateful-loop-effects-v1.md).
The Git baseline remains beta-0.17.1, commit 99ebfcb2; this is a worktree
follow-up, not a version bump or a stable-ABI declaration.

## Explicit Statement Authority

The isolated iteration-effect plan now passes an explicit
`LiteralPrints::Allow` capability through source loop admission, nested lexical
validation and scoped branch/iteration outlining. The pure route explicitly
passes `Reject`; the Buffer route never acquires literal-print authority.
Only `NirStmt::Print(NirExpr::Int(_))` is admitted. Computed, bool and floating
prints, arbitrary expression statements and return-in-loop effect closures
remain outside this profile. Source prints are neither erased nor classified
as pure, and the effect catalog never seeds pure discovery.

[Source admission](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/sequences.rs)
recognizes authorized printing as actual work even when no business carry
exists. [Scoped outlining](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/nested.rs)
uses the existing discarded neutral i64 helper result for that case: no fake
State slot or carry seed is introduced. Branches retain their original guards;
nested loops propagate the same explicit capability. Exact type/mutability,
no-forward-sibling-read, header invariance, lexical depth and completed-call
dependency checks remain unchanged.

[Unit controls](../../tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects_tests.rs)
cover leading/trailing and nested print-only admission, mutable carries,
source-fixture admission, unchanged pure exclusion before/after outlining,
nonliteral/foreign-statement rejection and direct Buffer/pure validator denial.
Those source-shape controls do not by themselves certify every nested/exit
form through native delivery. The retained effect plan continues to veto
return-in-loop effect roots and their dependent callers.

## Actual Execution

[The shared Nuis profile](../../tools/nuisc/tests/native_application_bridge/state_loop_statements_profile.rs)
adds a real `while echo < trips` containing only print 7 and its induction step,
followed by the state-computing loop. Print 10 is a direct loop statement after
the root proof succeeds. The selected source branch prints 19 before the
checked refine argument and 20 after the actual refine call succeeds.
Both loops remain dynamic; no test-side source unrolling or artificial source
helper is used. The existing advance/finish/answer prints remain 5/35/40.

The state-computing loop explicitly stages its self-update and bool predicate,
respecting the existing source-ordered carried-read restriction. Four source
variants mirror the selected arm and wrap actual current/answer arguments with
+0. The independent signed arithmetic oracle remains unchanged. Expected print
traces additionally insert the first loop's real prints, including before a
later root/work/tail/answer failure.

[Reference and native tests](../../tools/nuisc/tests/native_application_bridge/state_loop_statements.rs)
reuse 48 seed/gate/flag/trip cases, four raw marker words and two proof modes.
Negative/zero trips produce no iteration prints; one/three trips use current
values. Separate poisoned holes test unselected zero divisors. Normal/reversed
YIR inventories execute actual previous callback outputs, not expected State
injection. Observers record actual refine and answer arguments; all source
print markers originate in Nuis.

Eight main processes contribute 15360 healthy accepted callbacks.
Thirteen arithmetic failures across four sources/two inventory orders contribute
104 fatal processes and 312 earlier accepted callbacks. Selected failures at
indices zero/one/two preserve completed iteration prefixes, including print 19
before a failing refine argument and no print 20 afterward. Reference failure
retains the entire last State, rejects subsequent events and cannot complete
successfully after close. Native failures prove flushed exact prefixes and
absence of later publication, not post-abort memory inspection.

Raw positive/negative zero, minimum subnormal and signed quiet-NaN marker words
retain their exact State bits. Source effects are not rolled back; new State
publication still waits for successful callback return.

## Shared Budgets And Delivery

The main policy explicitly grants six loop-work units and 100 helper entries
per callback. Two three-trip loops consume all six units. A print-only loop
does not bypass accounting merely because its helper result is discarded.
Limits zero/two/three reject oversized first-loop work before print 7.
Limits three/four/five admit the entire first three-trip loop, then reject the
second reservation before print 10: the exact flushed prefix is 5,7,7,7.
Each loop would fit alone; the shared callback debit is what rejects the pair.
All negative controls provide valid root/work arguments so a masking division
trap cannot stand in for reservation rejection. Native children remain bounded
by the existing ten-second deadline.

The separate reservation cohort adds twelve healthy processes /60 accepted
callbacks and thirty fatal processes /30 earlier accepted callbacks. Zero-trip
sessions still work at limit zero; the budget resets for each callback.

[AOT controls](../../tools/nuisc/src/aot_application_effect_call_state_loop_statements_tests.rs)
check the exact State layout, retained runtime loop structure, no owned
aggregate allocation, omitted-single-grant rejection, ungranted-mode rejection
and rejection after removing Effect ordering. The compiler does not invent
backend permission from statement admission.

[The shared CLI workflow](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop.rs)
now takes an exact trace function and budget, rather than a two-family flag.
It replays pure calls, effectful calls and direct statements through the same
fresh/cache/two source-free restore checks. Each family has sixteen images,
640 healthy accepted callbacks, 336 prior accepted callbacks, 112 fatal events
and sixteen oversized-script rejections before open. The three families cover
48 images. CLI marker/trip/proof profiles are four finite witnesses, not a full
raw-marker cross-product or NaN input proof. Fatal buffered CLI stdout is not
prefix evidence; the flushed native probe supplies that evidence.

Executable, host-object, envelope, lifecycle and exact budget-bearing packaging
identities survive cache and restoration. Source, manifest, project cache and
original output are removed before both restores. Execution and restoration
leave artifact/container/executable bytes unchanged.

## Acceptance And Lineage

Acceptance is serial on the current macOS CPU host. The selected cohort is
668 distinct tests: 614 compiler, 21 native/reference/oracle, three CLI, three
host and 27 tensor. Nine tests are new; focused reruns and child processes are
not counted as additional tests. Existing carried-loop/call-effect, sequential,
join, skip and typed exit-prefix compatibility cohorts are replayed.

Run from the repository root, serially:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields return_in_fields return_nested_in_fields return_prefix_in_fields return_sibling_in_fields return_tail_in_fields return_child_tail_in_fields return_pair_tail_in_fields return_triple_tail_in_fields return_quad_tail_in_fields return_quint_tail_in_fields return_paired_in_fields return_exit_prefix return_typed_exit_prefix return_plain_prefix return_plain_tree return_plain_const return_stateful_bool return_stateful_fields return_state_routing state_join state_skip state_sequence state_loop iteration_effects control_loops scalar_helpers scalar_control --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_typed_exit_prefix state_join state_skip state_sequence state_loop:: state_loop_effects:: state_loop_statements:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- effect_calls::state_loop:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

All 3302 drift checks /22824 patterns, changed Rust formatting, UTF-8,
source/test/document caps, relative links and diff checks pass. Four untouched historical whole-workspace formatting
differences remain. Strict Clippy/full-workspace tests and fresh
Linux/GPU/Windows/performance acceptance are not claimed.

The coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
All six preceding complete values are retained, not just their recent fronts.
Command history is a collision-checked quoted no-op heredoc; syntax and
harmless current-front replay ensure that old commands remain inert data.

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `085fd1c4860e07accf4076f29e8cac92611df6e53629dcd271dda53412dc3c0c` |
| `next_step` | `5de8a3cdca5a1f357eab91931b0e4369f2e0836f93be06fc8dc8233c541c6b18` |
| `blocker` | `382c2daf006c1e4b8d53cabaa6dc7b8a5c410ddf951486307d438fcd34564933` |
| `next_action` | `1a8cb912169e052b2e7661e994f4497d54dfa8e686afd96ec263a4ca8d422ab5` |
| `validation_command` | `498b13ba038364476d3d0ddbf9c864ff875d99cb57fcf33420352dd5c542975e` |
| `expected_artifact` | `774cf117db5180628dca821c82c884b9c76e52d8454ca7ff9ddb4b69084a1bc4` |

The rebuilt ordinary CLI must match all six source fields and predecessor
hashes, with clean drift/coverage/manifest/milestone/hierarchy/lineage and the
same ready selected task card. Passing this bounded receipt is not global
completion.

## Next Boundary

The [effectful loop-exit follow-up](nuis-native-stateful-loop-exits-v1.md)
now supplies this composition proof through existing lowering. The counts
above remain the direct-statement checkpoint's historical acceptance.

Exercise real effectful `break`/`continue` sessions through this route: verify
skipped suffix prints, actual induction/carry recovery, per-callback cumulative
reservations, selected failures and source-free identity. Existing source-shape
normalizers are not a substitute for that composition proof.

General I/O, asynchronous resources, return-in-loop effect closures, broader
entering-State writes, stable ABI, formal/global memory safety, measured
performance, generalized GPU closure and completed self-hosting remain
separate.
