# Native Effectful Return Storage Sharing V1

This bounded CPU follow-up advances the
[independent effect-prefix certificate](nuis-native-return-effect-prefix-v1.md).
It enables a separately certified record-storage reuse, not arbitrary effectful
snapshot optimization. Git identity remains beta-0.17.3, commit 81dc1669.
The later [ordered initialization-prefix follow-up](nuis-native-return-effect-storage-prefix-v1.md)
extends the original contiguous-binding boundary below; this receipt retains
its own historical scope and acceptance counts.

## Original Read Authority

[Return preparation](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/returns.rs)
still admits source using independent typed payload and pending storage.
Only after its structural certificate and iteration-body validator pass does
the optional shared-storage attempt run. A failed attempt returns the intact
independent plan, without publishing partial helpers or changing source.

[The separate storage certificate](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage.rs)
rechecks that baseline. It independently verifies an exact-result-type mutable
local, initialized in the initial contiguous binding prefix and already written
as an actual loop carry. Parameters, constants, missing, late and non-carried
locals cannot substitute for return storage. The pending flag remains fresh,
independent and identical to the baseline flag.

The shared tree must preserve every original statement, selected call/check,
ordered argument, branch arm, loop step and ordinary local exit. The only
differences permitted are omission of the separate zero payload initializer and
publication of a completed return value into the eligible source record.
Pending is set only afterward, followed by exact scoped exit propagation.

Both independent and shared bodies receive their own ordered-read/type checks.
Reusing storage must not grant a return expression an assignment's self-read.
One mutation is admitted by the shared body's self-read rule but rejected by
the original independent body; source admission remains rejected. Conversely,
adding an inner write to a formerly read-only record can invalidate a sibling
read. That valid source retains its independent-storage fallback.

The checker reuses the bounded tree walk and exact structural matcher, not the
producer's candidate decision. It does not widen purity, call, resource, async,
GLM, ABI or native grant authority. The older independent certificate still
rejects shared plans; the new certificate does not weaken that contract.

## Actual Carry Reduction

[The generated Nuis fixture](../../tools/nuisc/tests/native_application_bridge/state_loop_return_shared_cases.rs)
carries a real mutable `State { result: bool, total: i64, gain: f32, marker: f64 }`
instead of four scalar locals, retaining the established independent source
oracle and observable work. Boolean fields are explicitly read before scalar
comparison, respecting existing predicate admission rather than broadening it.
Eight variants cover both parent/child step placements and mirrored guards.

[Execution controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_execution_tests.rs)
lower both optimized and explicitly independent NIR. Every variant's maximum
scoped YIR carry width drops by four transport words, not just one named local.
This is not a physical-memory or speedup measurement: final LLVM allocation
and register optimization may differ. A separate two-i64 record test checks
field swapping and a fallible final field, with a two-word carry reduction.

Independent/optimized reference runs compare actual entry results and print
traces under normal/reversed YIR storage, including selected traps. Direct
field evaluation must finish before any reused old field is overwritten.
No source-name or backend-specific case is introduced into production logic.

## Execution And Delivery Scope

[Native/reference regressions](../../tools/nuisc/tests/native_application_bridge/state_loop_return_shared.rs)
exercise the eight variants, raw float-word pairs, local exits, actual prior
State handoff and failure after accepted prefixes. Reference faults retain the
last valid State, enter Faulted and reject further events/completion.
Native probes observe flushed failure prefixes, status/output canaries and no
owned-aggregate heap transport; they do not inspect memory after native abort.
Bounded representatives at limits 4/5 reject full reached reservations even
when the source would return early. Skipped poisoned bounds remain unevaluated.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_shared.rs)
checks one representative through fresh, cache and two source-free restorations.
Original source, project manifest/cache and original output are removed before
restoration. Binary/container, host objects, envelope, lifecycle and packaging
identity remain unchanged. Failed callbacks publish no new State/completion;
native probes, not buffered CLI abort stdout, own the exact-prefix evidence.

## Remaining Frontier

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Candidate discovery still stops at the first non-binding statement. Next prove
an initialization/dominance scan across an ordered literal-print prefix without
moving effects, accepting branch-local/parameter storage or granting invalid
reads. Independent storage remains the fallback for every unproved case.
General resource/I/O/asynchronous capture, broader entering-State writes,
stable ABI, full heterogeneous closure and completed self-hosting remain
separate. No GPU/Linux/Windows, whole-workspace semantic or performance
acceptance follows from this scoped CPU optimization.

## Scoped Acceptance

This follow-up passes 285 distinct focused tests, 12 new: 224 compiler units,
four native/reference/reservation/preceding-break compatibility tests, one
fresh/cache/source-free CLI test, three host fault tests, 24 repository-maintenance
tests, two line-budget tests and 27 tensor tests. The initial seven-test probe
is included in the final 224-unit result, not added again. The eighth new unit
is the two-field swap/fallible-publication control. Prior feature and maintenance
receipts keep their own historical totals.

The eight source variants retain the independent source oracle and explicitly
independent lowering baseline. All eight reduce maximum scoped YIR carry width
by four transport words; the two-i64 control reduces it by two. Native probes
check raw float words, accepted State handoffs, selected failure prefixes,
ordinary local exits and full reached-bound reservations. These are bounded
CPU correctness and representation results, not a performance comparison.

CLI acceptance checks one representative across four images: fresh, cache and
two source-free restorations. Its final run passes in 82.02 seconds on this M2.
Each image checks healthy sequences before and after both selected fatal-event
cases, with unchanged binary/container identity and no failed State/completion
publication. Exact abort-prefix observations belong to the native probes, not
the buffered CLI output. This does not multiply one delivery representative
into an eight-variant source-free matrix.

All 3,435 drift checks, locked metadata for 29 packages, workspace Rust
formatting, UTF-8, documentation links and source/test/document budgets pass.
The ordinary CLI was rebuilt and matches all six current tensor fields and
their complete predecessor hashes. Coverage, hierarchy and task-card lineage
remain clean. The selected coordinate stays active/99, with a ready task card
for the ordered literal-print initialization-prefix proof described above.
Complete predecessor values remain retained; shell syntax and an inert-history
probe confirm that the historical validation commands are not executed.

| Predecessor Field | Complete SHA-256 |
| --- | --- |
| `evidence` | `37b5bee9d2ec08992e24218d240349cd6b11d4339ea6c22008e3de84bd13549a` |
| `next_step` | `346db6e1ba32011d444eff5543e30eeb77c1587a1ca5f3c069935dcc06ce113d` |
| `blocker` | `811991fd6328a0bc4818b6849f138a658d2da0b06a8f29fa990ff16d33875a52` |
| `next_action` | `6cd42b2001ed18300a3dddb611ae802bbac787594bab1dd1801cf8faa595c0a5` |
| `validation_command` | `274c58812e88cb50e135249c48bdb29d194e250b0f0206a45ea6cca62d64a8f7` |
| `expected_artifact` | `14bb81e2528f1eff41dac8136101135383189b4ff6fdbac533520805f84d3d49` |

No Git commit, push, tag or version bump is performed. Existing uncommitted
mainline changes and prior regressions are preserved. No new Linux, Windows,
GPU, full-workspace semantic, physical-memory/performance or completed
self-hosting acceptance is claimed.
