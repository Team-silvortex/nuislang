# Native Private Continuation Event Budget V1

Status: staged beta-0.16.* worktree proof. Git remains authoritative.
No production compiler/runtime behavior or source eligibility is changed.

This follows the [event-failure proof](nuis-native-private-continuation-event-failure-v1.md)
using the same bool/i32/i64/f32/f64 child fixtures beside one outward bool ancestor.
It checks a bounded long-event route against adjacent helper-entry limits after
successful open and a distinct successful short event.

## Inclusive Charging

Each lifecycle callback owns a stack-local helper-entry counter. Every admitted
YIR function entry debits one, including the lifecycle root and generated private
arms/predicates. Zero remaining traps before that function body; arguments
already evaluated by the caller are not refunded. Flattened source functions
do not create phantom YIR entries: observe is incorporated into start here.
Loop reservations remain separate and unchanged.

The literal test paths, not a runtime threshold search, define these bounds:

| Private Child | Short Open | Short Event | Long Event | Adjacent Limits |
| --- | ---: | ---: | ---: | --- |
| bool | 6 | 7 | 38 | 37 and 38 |
| i32 or i64 | 7 | 8 | 25 | 24 and 25 |
| f32 or f64 | 7 | 8 | 24 | 23 and 24 |

The bool seed is true, integer seeds are zero and floating seeds are negative
zero. Open has false/false flags, the short event has false/true flags and the
long event has true/true flags. Bool reply follows the second flag.
Inactive proof denominators remain zero. Each callback state is independently
predicted by the existing source-semantic oracle.

The long event includes typed current-child joins, seals, nested selections and
an unused source call. At the smaller limit, answer_work has executed but the
final private publication continuation cannot enter. Its print(79) and callback
output stores must not execute. Already emitted effects are not rolled back.
The larger limit permits a second identical long event and close even though
the first long event consumed its whole allowance. This distinguishes local
reset from session-wide accumulation.

Private helper names are literal fixture-topology regression expectations,
not a public ABI. Their identities and interleaved prints are checked against
actual native output; the test does not discover an expected path from that
output. Source output order is also cross-checked against the independent oracle.

## Native Probe

Five tests link two limits under normal and reversed YIR inventories:
20 real native processes, ten fatal budget traps and ten successful five-callback
sequences. The complete interleaved entry identity, remaining budget, source
print, return status, state words and two canaries must match the literal oracle.

The driver feeds actual preceding output to the next event and close, never
oracle-manufactured state. Test-only observations flush after the original debit
and immediately before an existing helper-entry trap. Admission comparisons,
debit stores, function bodies, joins, output stores and real llvm.trap remain
unchanged. Only SIGILL/SIGTRAP satisfies fatal rejection; host kills do not.

Before the failing event, open and short-event states differ. The pre-trap
observation reads the actual callback output buffer and requires the entire
last accepted state and both canaries unchanged. Neither return, later event,
close nor their post-callback markers may appear after the trap.
This is a pre-trap memory observation, not post-crash access or recoverable cleanup.

These loop-free carriers retain stack counters and no owned aggregate allocator
or drop calls. The bounded native harness has a ten-second process deadline.

## CLI Evidence

Five CLI tests build both adjacent policies through fresh build, cache hit and
two source-free restorations: 40 image checks, 20 CLI traps and 260 accepted CLI callbacks.
Each image runs the open/short-event/long-event/long-event/close script, followed
by healthy short open/event/close reentry. At the smaller limit only the first
two script states publish; at the inclusive limit all five publish and completion
succeeds. Exact healthy stdout is checked; absent fatal stdout is not no-work evidence.

Each compiled policy initializes exactly three callback-local counters with its
own limit. Default existing test grants remain at 100 through the shared helper;
production source discovery, common guards and Effect policy are not widened.

Executable bytes, packaging mode, envelope, lifecycle contract and native objects
remain equal between fresh/cache builds of the same policy. Embedded cache
diagnostics may change, so whole fresh/cache containers are not compared.
Each image's container remains unchanged by execution. The standalone input
artifact remains unchanged by both restorations after source, manifest and
project cache are deleted. There is no provider/interpreter fallback.

## Acceptance

The 2026-10-11 focused serial acceptance passes 57 distinct scoped units:
ten new tests and 47 existing regressions/policy/tensor units.

The accepted union is 21 native tests (five new budget, ten prior checked-argument
failure and six existing core helper-entry tests), six CLI tests (five new budget
and one existing selected-call workflow), three existing native-host failure-policy
tests and 27 tensor tests. Probe variants, retries, callbacks and processes are
not extra test units. The preceding 53-test event-failure and 701-test float receipts
remain historical checkpoints, not additional current acceptance.

The native 21-unit union passes in 44.77 seconds; five new CLI workflows pass in
227.56 seconds and the unchanged-default CLI regression passes in 22.10 seconds.
All three existing host policy tests and all 27 tensor tests pass. Test-binary
name inventory checks define distinct units; variants and post-publication retries
do not enlarge the 57-test receipt.

Ordinary CLI publication verifies clean coverage, hierarchy, task-card lineage
and 2477 drift checks / 17233 required patterns. The selected cell remains
active/99 with six source-equal fields and six complete hash-identical event-failure
predecessors. A post-publication 27-unit tensor rerun confirms the final metadata
without adding units.

Use CARGO_INCREMENTAL=0, CARGO_BUILD_JOBS=1, --locked, -j1 and --test-threads=1:

- nuisc native_application_bridge: private_event_budget, private_event_failure,
  helper_entries_share_exact_budget, helper_entries_reject_before_body,
  helper_entries_skip_unselected_paths, helper_entries_use_unsigned_limits,
  helper_entries_count_loop_free_roots and helper_entries_include_scoped_iterations.
- nuis native_session_workflow: private_event_budget and
  native_effectful_selected_calls_build_cache_restore_and_execute_only_selected_arguments.
- yir-runtime-host: native_event_failure_retains_last_state,
  malformed_native_output_and_failed_close and native_host_failure_still_requires.
- nuis binary: dev_tensor, followed by rebuilt ordinary CLI live status audit.

## Remaining Boundary

These thresholds cover one fixed short/long route per type, not all source paths
or every internal budget cut point. No production compiler/runtime changes, new
GPU/Linux/Windows execution, measured speedup, formal memory safety or completed
self-hosting are claimed.

The next functional boundary is an exact i32 outward ancestor beside a private
bool child in terminal mixed continuations. It still needs original-forest
preflight, ordered source work and exact typed private transport without exporting
child bindings or admitting resources, partial exits or hidden ordinary leaves.

That boundary is now covered by the subsequent
[i32-ancestor follow-up](nuis-native-private-i32-ancestor-continuations-v1.md).
The 57-unit event-budget receipt above remains its historical checkpoint,
not the acceptance count of the broader functional extension.

