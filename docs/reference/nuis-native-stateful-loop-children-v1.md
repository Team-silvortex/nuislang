# Native Stateful Child-Local Loop Exits V1

Historical return-in-loop exclusions below describe this checkpoint. The later
[effectful loop-return receipt](nuis-native-stateful-loop-returns-v1.md)
advances that boundary through explicit literal-print typed transport and
native/source-free proof, not general I/O or pure-function reclassification.

This bounded CPU receipt advances the
[effectful loop-exit checkpoint](nuis-native-stateful-loop-exits-v1.md).
Its Git baseline is beta-0.17.2, commit 00ab72d3. This work does not bump the
version or declare a stable ABI.

## Nested Source, Existing Lowering

[The Nuis program](../../tools/nuisc/tests/native_application_bridge/state_loop_children.ns)
runs a continuing leading-step parent loop. Every parent iteration initializes
a child index and runs a child counted loop. The state-derived gate is flipped
by actual parent-index parity, so two parent rounds exercise one child break
and one child continue in either order.

Print 8 marks each parent entry. Child prints 10/11/12/13/19/20 keep the earlier
exit/checked-work semantics. After the child, the parent prints 21, calls
`combine(current, child_index, parent_index, proof)`, then prints 22.
After the entire parent, print 31 precedes a final current/parent-index observer.
The child index contributes ten times its value and the parent index one
hundred times its value to real parent work; they are not test-only counters.

Four [source variants and the independent oracle](../../tools/nuisc/tests/native_application_bridge/state_loop_children_cases.rs)
combine leading/trailing child induction and original/mirrored selection.
The parent remains continuing in this receipt. Child exits use the existing
scoped normalization, shared loop lowering, type/mutation/header checks and
explicit literal-effect authority. No production helper-name whitelist,
special source dispatch or new lowering branch is introduced.

[Unit controls](../../tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects_children_tests.rs)
require exactly one child break-control registration, observed child recovery
only for leading child steps, unchanged pure exclusion and rejection of child
writes to parent induction or either header. Manual two-parent/three-child-trip
witnesses give totals 605/809 for the trailing child and 423/762 for the leading
child under the two opening gates. Actual arguments are observed separately,
so a wrong child index cannot hide behind a compensating current value.

The oracle models nested source control and bound reservation directly, not
generated flags or field transport. Its 100 cases consist of a 96-case signed
seed/gate/flag/child-trip/parent-round grid plus explicit negative-parent and
later-cut witnesses. Four raw marker words and two proof modes yield 800
three-event sequences per source/inventory order.

## Execution And Failure Isolation

[Reference/native execution](../../tools/nuisc/tests/native_application_bridge/state_loop_children.rs)
uses actual previous callback output, not expected-State injection. Native
observers record real refine arguments, all four combine arguments, final
current/parent index and answer inputs. Exact print traces, status, canaries and
raw State words are checked in normal/reversed YIR inventory order.

Two parent-entry and parent-suffix effects must occur even when the first child
breaks. Child continue must skip only its own remaining work. Alternating child
control also checks fresh exit flags on the next child invocation. Recovered
child indices feed actual parent work before its backedge.

Zero/negative parent rounds skip child bounds and all parent/child work.
Zero/negative child trips still allow the parent's suffix and next iteration.
Separate poisoned root/hole/suffix proofs certify skipped work. Their valid
arithmetic and resulting States may differ from all-valid proofs; equivalence
between proof modes is not assumed. Raw positive/negative zero, minimum
subnormal and signed quiet-NaN marker words retain exact State bits.

Eight main native healthy processes contribute 32000 healthy accepted callbacks.
Each source/inventory order has sixteen selected arithmetic failures:
advance/root/tail/answer, selected child holes and first/second parent-suffix
proofs. This is 128 fatal processes and 384 earlier accepted callbacks.
A later child/suffix trap preserves earlier completed parent and child effects,
but cannot publish a failed/next/close State or complete the session successfully.

Reference faults retain the entire last State and reject subsequent events.
Flushed native fatal prefixes prove completed effects and no later publication,
not post-abort memory inspection. Source effects are not rolled back.
The existing ten-second native child-process deadline remains in force.

## Shared Reservations And Delivery

The main policy grants eight loop-work units and 100 helper entries:
two parent trips plus two three-trip child reservations. Child break does not
refund a declared reservation, and a later child does not receive a fresh
callback budget.

The budget cohort exercises limits zero/two/seven with valid body proofs.
At zero, an oversized parent rejects after source prefix 5.
At two, the parent reserves both trips, enters its first body (print 8), then
rejects the first positive child reservation with prefix 5,8.
At seven, the first child and complete parent suffix execute; the second parent
prints 8, then rejects its child reservation before print 10. Both first-child
break/continue choices are checked, including an early break whose full bound
still consumes three units.

Positive controls accept an unselected i64::MAX child bound under a zero-trip
parent, zero child work under two parent trips, and one-trip children under the
larger budget. Bounds 65537/i64::MAX reject without a masking arithmetic trap.
The separate budget cohort contributes 24 healthy processes /240 accepted
callbacks and eighty fatal processes /eighty prior accepted callbacks.

[AOT controls](../../tools/nuisc/src/aot_application_effect_call_state_loop_children_tests.rs)
retain the exact three-field State and runtime loop structure without owned
aggregate allocation. Omitting any single print grant, the default ungranted
mode or removal of Effect ordering rejects. Pure/Buffer authority and
return-in-loop effect restrictions remain unchanged.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_children.rs)
checks four sources through fresh/cache/two source-free restored images:
sixteen images, 640 healthy accepted callbacks, 160 fatal events with 480 prior
accepted callbacks, and sixteen oversized-script rejections before open.
The finite CLI marker/trip/proof witnesses are not a full cross-product or raw
NaN input proof. Fatal buffered CLI stdout is not prefix evidence; the flushed
native probe supplies that evidence.

Executable, host objects, envelope, lifecycle and exact budget-bearing packaging
mode remain identical across cache and both restores. Source, manifest,
original output and project cache are removed before restoring. Artifact,
container and executable bytes remain unchanged by restoration/execution.

## Acceptance And Lineage

This fresh serial macOS CPU cohort passes 214 distinct selected tests:
167 compiler, sixteen native/reference/oracle, one new CLI delivery, three host
and 27 tensor. Eight tests are new: three unit/oracle, one AOT, three native/
reference/budget and one CLI. Focused reruns and child processes are not extra
tests. The native cohort replays the four preceding loop families. Earlier
207-test and 668-test receipts remain historical, not newly recounted full
replays.

Run serially from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- iteration_effects control_loops scalar_helpers scalar_control state_loop --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_loop:: state_loop_effects:: state_loop_statements:: state_loop_exits:: state_loop_children:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- effect_calls::state_loop_children:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

All 3324 drift checks /22970 patterns, changed Rust formatting, UTF-8,
source/test/document caps, relative links and diff checks pass. Four untouched
historical whole-workspace formatting differences remain. Strict Clippy,
full-workspace, fresh Linux/GPU/Windows and performance acceptance are not
claimed.

The coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
All six complete preceding values remain unabridged. Command history is a
collision-checked quoted no-op heredoc; shell syntax and harmless current-front
replay verify that old commands remain inert data.

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `70b252648dc702e07a35a0d13dc7a66bc8294d96082a159025db96b2c8b6920d` |
| `next_step` | `b1180f72947784ce48b92964dad3f1f78eb0e571db428f754114c14cbfb6b6e8` |
| `blocker` | `50dc7863c51f19afac23ffc169d8c825ba943936a0a05fca846ee7b08e55a959` |
| `next_action` | `432a80c96d9a211b1ee9a90385706a952dd9cb31df190467ef6fe86528a1e972` |
| `validation_command` | `8404b55f9b75320af581a39b09cd95f1657b6f093c6db0bb7333a30045c74af8` |
| `expected_artifact` | `fc96c16b29debf7c85db46ea34669c5177af63c902ad25a26a8b9f7d8ab62d5a` |

The rebuilt ordinary CLI matches all six source fields and complete predecessor
hashes, with clean drift/coverage/manifest/milestone/hierarchy/task-card lineage
and the same ready selected task card. Bounded acceptance does not mark the
coordinate complete.

## Next Boundary

The subsequent [selected-child invocation receipt](nuis-native-stateful-loop-selection-v1.md)
now advances the historical conditional-invocation boundary below through real
sessions and sixteen delivery images. Its next boundary is parent-local exits
after completed selected child work.

Exercise conditionally selected child-loop invocation inside a parent:
an unselected child must not reserve work or evaluate its bounds/body proofs.
Verify state-derived selection, parent continuation, selected failure prefixes
and source-free identity through actual sessions rather than shape tests alone.

General I/O, asynchronous resources, parent exits after child work,
return-in-loop effect closures, broader entering-State writes, stable ABI,
formal/global memory safety, measured performance, generalized GPU closures
and completed self-hosting remain separate.
