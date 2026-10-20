# Native Return Storage Prefix Reassignment V1

This bounded CPU follow-up extends
[candidate attempts](nuis-native-return-effect-storage-candidates-v1.md).
Git identity remains beta-0.17.3 at 81dc1669; no release is made here.

## Fresh Definition And Ordered Writes

The [storage certificate](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage.rs)
now distinguishes a fresh first definition from subsequent same-name writes by
its independently reconstructed prefix scope. The first definition must be a
non-constant record with the exact result type, absent from the parameter and
preceding-binding scope. Subsequent writes must retain that established type;
constants, changed inferred/declared types and missing initialization fail closed.
Only the existing straight-line binding/literal-print prefix is scanned.

NIR does not retain the AST distinction between a declaration and an assignment,
or the AST `let mut` flag. This certificate does not reconstruct that provenance.
[Frontend assignment validation](../../tools/nuisc/src/frontend/validation_assignments.rs)
still rejects assignment to immutable, unknown or parameter bindings; lowering
only optimizes independently admitted source. Repeated same-name NIR `Let`
statements can represent ordered record rebindings in the same prefix scope,
not a new grant to write constants or parameters.

Original independent admission runs first. Original and shared bodies keep
their own ordered-read/type checks. The structural certificate preserves every
initialization, reassignment, print, call/check and scoped return in its original
slot. Neither a later write nor a return may publish before its expression
completes. Eligible storage still has to be a real source loop carry.
The declaration-order eight-attempt budget and exact independent fallback remain
unchanged; reassignment does not create duplicate candidate attempts.

## Executable Controls

[Reassignment units](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_reassignment_tests.rs)
check eight step/guard variants and three names, source/baseline immutability,
fresh pending identity and retention of all original typed writes.
Moved, deleted and duplicated writes/prints remain type-valid in the mutation
controls but fail the structural certificate. Constants, parameter storage,
missing initialization, changed types, barriers and immutable-source assignment
remain rejected. A valid source with an invalid shared inner read set retains
exact fallback; an original-invalid self-read cannot gain shared authority.
The failed-first/later-successful candidate case also composes with reassignment.

The [source adapter](../../tools/nuisc/tests/native_application_bridge/state_loop_return_shared_cases.rs)
adds two real same-record writes around print 703, between prints 701 and 702.
Integer +3/-3 and two float sign flips restore the established oracle payload
for the existing bounded inputs while exercising reads of each preceding value.
Four explicit fixture modes replace growing boolean combinations; they are test
policy, not new production dispatch or a language/backend enumeration.

[Execution comparisons](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_execution_tests.rs)
lower optimized and explicitly independent NIR for all eight variants. They
compare actual entry results, ordered effects and selected failures in normal
and reversed YIR storage, retaining a four-word maximum scoped carry reduction.
This is not physical-memory or speedup evidence.

[Native/reference probes](../../tools/nuisc/tests/native_application_bridge/state_loop_return_shared.rs)
extend actual State, raw float-word, local-exit, fault-retention and full
reached-reservation matrices with the reassigned source. A dedicated second-write
failure occurs after the first write and print 703 but before print 702:
the native failing callback observes exactly `[5, 701, 703]`, without normal
callback publication. Reference sessions retain their last accepted State,
enter Faulted, reject later work and refuse successful completion after close.
Native abort observations do not inspect memory after the process traps.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_shared.rs)
adds one reassigned representative alongside the three preceding modes. Each
uses fresh/cache/two source-free images, healthy runs around selected failures,
unchanged artifact identity and no failed State/completion publication. Original
source, project manifest/cache and original output are removed before restoration.
Exact abort prefixes belong to the native probes, not buffered CLI stdout.

## Remaining Frontier

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Direct-expression prefix writes now have executable evidence. Next validate
record-returning effectful helper calls used as prefix reassignment values:
once-only ordered arguments, completed aggregate handoff, selected callee failure
and original-State retention. Typing may already admit these calls; establish
real lowering/native/source-free witnesses before declaring them unsupported or
adding any new call grant. Preserve all existing barriers and fallback rules.

General resource/I/O/async capture, wider changing backedges, stable ABI,
cross-platform/device parity, full heterogeneous closure and completed
self-hosting remain separate. This receipt does not accept those wider gates.

## Scoped Acceptance

This follow-up passes 317 distinct focused tests, 11 new: 242 compiler units,
15 native/reference/reservation/prefix-failure/preceding-break compatibility
tests, four fresh/cache/source-free CLI tests, three host fault tests,
24 repository-maintenance tests, two line-budget tests and 27 tensor tests.
The initial seven-test probe contains six new tests and one preceding declaration
control, all included in the final 242-unit result rather than added again.
Prior receipts retain their historical acceptance counts.

The native suite passes on this M2 in 288.06 seconds, preserving the preceding
plain, printed-prefix and alternative-candidate routes after the fixture-mode
refactor. New reassignment matrices retain exact raw State words, ordinary exits,
selected failures and full reached-bound reservations under normal/reversed YIR
storage. Two step representatives independently verify second-write failure:
native output stops at `[5, 701, 703]`, while reference sessions retain their last
accepted State, enter Faulted and reject further work/completion. Native abort
observations are not post-abort memory inspection.

Four CLI representatives pass in 314.45 seconds, covering 16 delivery images:
fresh/cache/two source-free images per representative. This is not an
eight-variant source-free matrix. Each source program's own binary/container,
host objects, envelope, lifecycle and packaging identity remain unchanged across
cache/materialization; distinct programs need not have equal binaries.
Healthy sequences run before and after selected fatal cases, without failed
State/completion publication. These durations describe regression scope, not
runtime speedup or physical-memory measurements.

All 3,459 drift checks, locked metadata for 29 packages, workspace Rust
formatting, UTF-8, relative documentation links and source/test/document budgets
pass. The ordinary CLI was rebuilt and matches all six current tensor fields
and complete predecessor hashes. Coverage, hierarchy and task-card lineage are
clean. The selected coordinate stays active/99 with a ready task card for
record-returning effectful helper-call prefix-write witnesses. Existing typing
is not classified as unsupported without those execution diagnostics.
Complete historical values are retained, and shell syntax/inert-history probes
confirm that preceding validation commands remain data, not repeated execution.
Older drift guards now check fresh-definition/same-type-write rules and explicit
fixture modes rather than obsolete repeated-binding or boolean-selector text.

| Predecessor Field | Complete SHA-256 |
| --- | --- |
| `evidence` | `e4a9935ddcd4542ab7990006d04cd657b32bb1ad67ac78bf0602bf19c880de49` |
| `next_step` | `beba1706734fd2b36124abb6a43a8c4ce18fc32787a5f76f49361b683722849a` |
| `blocker` | `a602e862d3b877672d1cb86437a07435e75d52fb55e6e2137e3c646c6694f011` |
| `next_action` | `576f1bce1c505eaf3df78282408236dd6742cac87481a5941ed37efcb93ce5c7` |
| `validation_command` | `961341f58bddba73fd8619ab31323f0fe1a0fc107dc518a92daffa75c037cb1f` |
| `expected_artifact` | `6a8539c7df16c338266ec880716b510f35c06fbe0da94173d3d331ea351bc062` |

No Git commit, push, tag or version bump is performed. Existing uncommitted work
and prior regression registrations are preserved. No new Linux, Windows, GPU,
full-workspace semantic, physical-memory/performance or completed self-hosting
acceptance is claimed.
