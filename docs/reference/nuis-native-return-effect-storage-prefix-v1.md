# Native Return Storage Initialization Prefix V1

This bounded CPU follow-up extends the
[record-storage certificate](nuis-native-return-effect-storage-v1.md), not the
source admission profile. Git identity remains beta-0.17.3 at 81dc1669.
The later [bounded candidate-attempt follow-up](nuis-native-return-effect-storage-candidates-v1.md)
advances the first-candidate boundary below; this receipt retains its historical
scope and acceptance counts.

## Ordered Initialization Discovery

[Candidate discovery](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_storage.rs)
now has an explicit effectful entry. It may scan across `Print(Int)` statements
interleaved with initial bindings, without moving or evaluating those statements.
The pure entry retains its rejecting print policy. Dynamic prints, branches,
loops, returns and other non-binding statements remain discovery barriers.

The [independent storage certificate](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage.rs)
separately recognizes the same narrow print syntax. A literal print changes
neither the typing scope nor initialization dominance: a candidate must still
be an exact-result-type, fresh mutable record initialized before the first
control-flow barrier, and actually written as a source loop carry. Constants,
parameters, branch-local, repeated and mistyped declarations cannot substitute.
The certificate does not call the producer's candidate search.

Original independent admission runs first. Both original and shared bodies
retain their own ordered-read/type checks. The exact structural proof preserves
each print and binding in its original slot, including fallible initialization
between prints. A selected return finishes evaluating all fields before payload
and pending publication. Unproved sharing retains the independent plan.
There is no new ABI, backend special case, call/purity grant, resource/async
capture authority or source-name convention.

## Executable Controls

[Prefix unit regressions](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_prefix_tests.rs)
check eight step/guard variants under three record names, original source
immutability, pure-policy separation and independent pending storage.
Mutation controls move, remove or duplicate prints, move initialization into a
branch, insert barriers, repeat/mistype declarations and test original-invalid
self-reads. A valid source whose new inner write fails the shared read-set check
keeps its exact independent fallback.

[Reference comparisons](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_execution_tests.rs)
lower optimized and explicitly independent NIR, check a four-transport-word
maximum scoped YIR carry reduction for all eight variants, and compare actual
entry results and ordered print traces in normal/reversed YIR storage.
Logical carry reduction is not physical-memory or speedup evidence.

[Native/reference probes](../../tools/nuisc/tests/native_application_bridge/state_loop_return_shared.rs)
reuse the independent source oracle, adding prints 701/702 inside `work`, after
the caller's print 5. They check exact prefixes, raw float words, actual prior
State, ordinary exits, failure retention and full reached-bound reservations.
A separate fallible-initializer probe accepts a real State first, then fails
between the two initialization prints: only `[5, 701]` executes in that callback,
not print 702, a result or a normal callback status. Native abort observations
do not inspect memory after the process traps.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_shared.rs)
adds one printed-prefix representative to the preceding representative. Each
uses fresh/cache/two source-free images, healthy runs around selected failures,
unchanged packaging/binary/container identity and no failed State/completion
publication. Original source, project manifest/cache and original output are
removed before restoration. Native probes, not buffered CLI abort stdout, own
the exact fatal-prefix evidence.

## Remaining Frontier

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Discovery still picks the first syntactically eligible carry. Next investigate
bounded deterministic attempts of alternative candidates when that first
candidate fails its own shared-body certificate. Each attempt must start from
the same independently admitted baseline, retain fresh pending identity and
publish no partial plan. Do not cross the initialization barriers to find more
candidates or infer shared authority from original admission alone.

General resource/I/O/asynchronous capture, wider changing backedges, stable ABI,
complete heterogeneous closure and completed self-hosting remain separate.
This scoped host-CPU work does not accept Linux/Windows/GPU parity or
full-workspace semantics.

## Scoped Acceptance

This follow-up passes 296 distinct focused tests, 11 new: 230 compiler units,
eight native/reference/reservation/initializer-failure/preceding-break
compatibility tests, two fresh/cache/source-free CLI tests, three host fault
tests, 24 repository-maintenance tests, two line-budget tests and 27 tensor
tests. The initial six-test prefix probe and strengthened valid-barrier replay
are included in the 230-unit result, not added again. Prior receipts retain
their historical counts rather than contributing another cumulative total.

The native suite passes on this M2 in 143.51 seconds. Its two fallible-initializer
step representatives and normal/reversed YIR storage observe exactly `[5, 701]`
in the failing callback, before later prints or normal callback publication.
The full eight-variant prefix matrix retains actual State, raw float words,
local exits and reached reservations, alongside the unchanged no-prefix route.
Native traps are exact effect-prefix observations, not post-abort memory checks.

The two CLI representatives pass in 154.91 seconds, covering eight delivery
images in total. Each representative owns fresh/cache/two source-free images;
neither is an eight-variant source-free matrix. Binary/container identity,
host objects, envelope, lifecycle and packaging remain unchanged across its
cache/materialization route. Healthy sequences run before and after both
selected fatal cases, with no failed State or completion publication.
These durations describe regression scope, not a speedup measurement.

All 3,443 drift checks, locked metadata for 29 packages, workspace Rust
formatting, UTF-8, relative documentation links and source/test/document budgets
pass. The ordinary CLI was rebuilt; all six selected tensor fields match the
source and retain their complete predecessor values/hashes. Coverage, hierarchy
and task-card lineage are clean. The coordinate remains active/99, with a ready
task card for bounded alternative-candidate attempts, not completed application
closure. Shell syntax and an inert-history probe verify that historical commands
remain data rather than being executed again.

| Predecessor Field | Complete SHA-256 |
| --- | --- |
| `evidence` | `741b7c788bc3c9ce307eb8f81595da6abba31a44fcfcd90c0d9a6bc2104adf09` |
| `next_step` | `ca80bf65b6957c27397ed22264b42d4178c1bde2ea7e0998226d6b86178b6a24` |
| `blocker` | `3236f49b7e77b8689cac0ef72d190a82333cc76321d06db99119735aa801b608` |
| `next_action` | `855fa036c5057bc506471241e34835975c0dc1e76b5ca60ddf0786055a7c98de` |
| `validation_command` | `b28905df45486565ca78966fe9d635dc955e13c67dd0a02aa7e9e5a40781764c` |
| `expected_artifact` | `6e786c315ad282115eda8a971e86c81bc6a702fc6835e19f52c305ee0d629b99` |

No Git commit, push, tag or version bump is performed. Existing uncommitted work
and prior regression registrations are preserved. No new Linux, Windows, GPU,
full-workspace semantic, physical-memory/performance or completed self-hosting
acceptance is claimed.
