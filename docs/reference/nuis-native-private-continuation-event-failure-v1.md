# Native Private Continuation Event Failure V1

Status: staged beta-0.16.* worktree proof. Git is authoritative; this is not a
version bump or a new source-eligibility claim.

This follows the [float-pair continuation proof](nuis-native-private-float-pair-continuations-v1.md).
The same source fixtures cover bool, i32, i64, f32 and f64 private children
beside one outward bool ancestor. No compiler/runtime production code is changed
by this follow-up.

## Failure Semantics

The scripted native process must complete open and a successful event before
a selected helper call's checked argument division traps during the next event,
before entering that callee. Its published
states must remain exactly those two accepted states, not a partially updated
third state. The failed event, later scripted event, close and completion marker
must not be published.

A real native trap terminates the process. This proves the publication prefix
and lack of later execution, not post-crash access to process memory or rollback
of irreversible effects. Already executed output is not undone. Buffered stdout
may be lost, so an empty CLI stdout cannot prove that selected work was skipped.

Reference-executor failures are different: the session becomes Faulted, retains
the last accepted event state, rejects later events and permits one explicit
cleanup with that state. Cleanup cannot convert the original failure into success.
The existing native-host recoverable-status and malformed-output tests remain a
separate host-policy proof, not evidence that native traps can be caught.

## Independent Cases

There are 48 selected helper families: twelve in the bool nested/sequential
fixture and nine each for i32, i64, f32 and f64. The test data selects a genuinely
active path for every helper. Inactive proof denominators remain zero.

The shared oracle predicts state, output order and current values independently
of the compiler carrier. The bool/i32/i64 replay functions have moved into small
test-only oracle modules and are reused by their original CLI regressions.
The IEEE float oracle is unchanged. Decimal float cases do not replace the
existing raw NaN-payload proof.

Open and the accepted event have different complete states. The failing event
uses the original open path with exactly its selected helper proof changed to
zero. This prevents a stale initial state from masquerading as the last event
state.

## Native Probe

Five native tests run both ordinary and reversed YIR inventories, producing
96 native event-time traps across ten linked executables. Each next callback
receives the actual preceding callback's output words, not an oracle-supplied
state. Exact output states, status words and surrounding canaries are checked.

The test-only print wrapper flushes observations. It does not replace source
callees, checked divisions, private helper computations, joins or llvm.trap.
The entire observed prefix must equal the independent oracle: two successful
callbacks followed only by work before the selected failing helper. No return
from that callback, caller continuation, close probe or later source work may
appear. Owned aggregate allocator/drop calls remain absent from these carriers.

Case selection is child-local environment input to the probe, not mutable
global test environment. The existing bounded process harness accepts a prepared
command and keeps its ten-second deadline. It rejects host kills and hangs;
only the actual SIGILL/SIGTRAP signals satisfy this trap proof.

Five reference tests separately repeat the 48 failures in both inventory orders,
checking last-event state retention, no event retry, one explicit cleanup and
failure-preserving completion.

## CLI Evidence

Five CLI workflows repeat all 48 selected failures on fresh build, cache hit
and three source-free restorations: 240 event-time traps. Every failed run begins
with accepted open/event callbacks and is followed by a healthy open/event/close
run with exact stdout and state. This defines 1200 accepted lifecycle callbacks,
including the two accepted callbacks before
each fatal event. Variants and callbacks are not additional test units.

Executable bytes and packaging policy are identical across all five publication
phases. Cache-hit payloads also retain the executable envelope, lifecycle
contract and native object bytes. Each image's own artifact container is
unchanged by successful or failing execution.

A fresh build's whole artifact container is not required to equal a cache hit's:
its embedded build manifest legitimately changes compile_cache_status from
miss to hit. Identity checks compare executable payload and contracts, not
diagnostic metadata. The selected standalone input artifact then remains
byte-identical across all three restorations.

Source, project manifest and project cache are deleted before restoration.
No interpreter/provider fallback is admitted, and failed processes cannot poison
the artifact or prevent a later healthy launch.

## Acceptance

The 2026-10-11 focused cohort passes **53 distinct tests**: ten new native/reference
units, thirteen CLI units (five new and eight original family regressions),
three existing native-host failure-policy units and 27 tensor units.
All 26 execution units and 27 tensor units have passed. Actual test-binary name
lists confirm counts of 10/13/3/27 with no duplicate units. The already accepted
701-test float-pair checkpoint is preserved as history, not silently rerun or
added to this scoped receipt. Focused retries add no distinct units.

The first CLI run passed the eight original units; its five new units rejected
an overstrict whole-container cache comparison. After correcting identity checks,
all five new units passed in separate f32 and four-family runs. Binary assertion
failures now produce bounded diagnostics instead of dumping artifact byte arrays.
Those retries add no units to the receipt. The full
workspace, strict Clippy, new GPU/Linux/Windows runs, measured performance,
formal memory safety and completed self-hosting are not claimed.

The rebuilt ordinary CLI publication audit confirms clean coverage, hierarchy,
task-card lineage and all 2467 drift checks / 17136 required patterns, with six
source-equal task fields and six complete hash-identical float-pair predecessors.
The selected cell stays active/99; helper-entry budget exhaustion is next.
The post-publication 27-unit tensor rerun adds no units to this 53-test receipt.

Selected serial recipes use CARGO_INCREMENTAL=0, CARGO_BUILD_JOBS=1 and -j1:

- nuisc native_application_bridge: private_event_failure.
- nuis native_session_workflow: private_event_failure plus the original nested
  sequential bool, i32-pair, i64-pair and float-pair logical selection filters.
- yir-runtime-host: native_event_failure_retains_last_state,
  malformed_native_output_and_failed_close, native_host_failure_still_requires.
- nuis binary: dev_tensor.

Drift registration is
tools/nuis/src/dev_tensor_drift_data_native_private_event_failure.rs.

## Remaining Boundary

The next task recorded by this checkpoint is addressed by the
[event-budget follow-up](nuis-native-private-continuation-event-budget-v1.md).
It checks one fixed long route per type after successful open/event, with literal
adjacent limits, unchanged pre-trap output and repeated event/close reset.
All-path budget exhaustion remains outside that bounded proof.

This follow-up does not widen scalar capture types, sealed writes, private child
exports, hidden ordinary logical leaves, mixed/partial exit authority, common
backend guards or Effect policy.
