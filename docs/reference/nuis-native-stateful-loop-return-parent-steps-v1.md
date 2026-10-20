# Native Stateful Trailing-Parent Typed Loop Returns V1

This bounded CPU receipt advances the
[selected typed-return proof](nuis-native-stateful-loop-return-selected-v1.md).
The Git baseline is beta-0.17.3, commit 81dc1669. There is no release bump or
stable-ABI declaration.

## Trailing Parent Steps

[The real Nuis source](../../tools/nuisc/tests/native_application_bridge/state_loop_return_parent_steps.ns)
moves the parent induction step to the end of its original body.
Literal print 26 marks entry to that final step region; it remains before the
canonical step, not a replacement for it.
The selected child retains its real effectful extent call, full reached-bound
reservation and exact bool/f32/f64/mixed-record return computation.

Sixteen native/reference source variants cover all result kinds, child leading/
trailing steps and original/mirrored guards. The parent is trailing in this new
matrix. Leading-parent acceptance is the preceding receipt and is replayed.
The shared source generator and independent source-level oracle keep that older
default unchanged; the new oracle mode steps the parent only after its suffix.

## Completed And Skipped Step Regions

The immediate-child witness begins at total 3.
The return callee must observe parent index 0, not a speculative leading index.
The record callback publishes 7 for a trailing child step and 18 for a leading
child step. False and signed-zero returns remain actual exits.

The late-entry witness begins at total 4 and uses divisor 2 for the parent's
post-child check. Its first parent iteration is unselected, completes its suffix
and reaches print 26. The next iteration enters the child and returns with
observed parent index 1, skipping the second parent suffix and step region.
The exact source print sequence is:

```text
5 8 9 23 21 22 26 8 6 16 7 10 11 12 30 35
```

Record results are 111 and 122 for trailing and leading child steps respectively.
A budget of 4 fails after the reached second-iteration extent/print 7, retaining
the first completed parent suffix and print 26. Budget 5 accepts.
The complete entered parent bound remains reserved even for a first-child return;
an early exit does not refund it.

Two unselected parent rounds with poisoned zero-divisor/overflow child bounds
complete both tail-step regions without evaluating the extent helper.
The independent oracle accepts a parent-only budget of 2 and rejects budget 1
before print 8; the native controls below use limits 4/5.
A skipped zero-parent loop does not run either body or tail step.
Empty/negative children and parent-local returns keep index 0 where appropriate.

Unit negative controls reject missing or duplicate canonical parent steps,
branch-local width escape, parameter/index writes, computed prints, mismatched
return types and source work after terminal continue.
The pure helper catalog still cannot authorize the effectful source.

## Execution And Delivery

[Native/reference tests](../../tools/nuisc/tests/native_application_bridge/state_loop_return_parent_steps.rs)
run ordinary and reversed YIR storage order, actual prior-State handoff and six
raw floating-word pairs including signed zero, signed subnormals and NaN payloads.
The new main matrix checks 8,064 healthy non-close callbacks and 224 failing
events after 672 accepted prefix callbacks on each execution route.
Observers enter the real extent and return callees; they do not replace Nuis
business computation.

The native budget subset covers four result-kind representatives with limits
4/5, checking 160 healthy non-close callbacks and 52 reservation failures after
156 accepted prefix callbacks.
Native probes copy actual output to the next input and check status, complete
raw State words and output canaries. Fatal probes compare flushed prefixes
through SIGILL/SIGTRAP; this is not post-abort memory inspection.
Reference faults retain the last published State, reject later events and
completion, and allow explicit close without a failed State publication.
Root 0, unused-proof root 2, child body, parent suffix and callback tail remain
distinct selected failure stages.

[AOT controls](../../tools/nuisc/src/aot_application_effect_call_state_loop_return_parent_steps_tests.rs)
check all four return kinds for exact State layout, individual print grants and
transitive Effect ordering. No owned-aggregate heap transport is introduced.
[Unit controls](../../tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects_return_parent_steps_tests.rs)
cover all sixteen source variants.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_parent_steps.rs)
uses one representative per result kind: plain/trailing children for bool/f64
and mirrored/leading children for f32/record.
Each goes through fresh, cache-hit and two source-free restorations, for
sixteen delivered images. The other guard/child-step combinations are native/
reference evidence, not additional CLI images.
Five healthy sessions precede each image's failure checks; one healthy session
then replays after failures. This checks 384 healthy non-close callbacks and
64 fatal events after 192 accepted prefix callbacks.
Both signed-zero/subnormal word pairs get exact CLI float parse-back.
NaN payload evidence comes from native words, not decimal output.
The source, manifest, project cache and original build output are removed before
restoration. Binary/container bytes, host objects, lifecycle, envelope and
packaging mode remain unchanged.

The existing generic branch/loop return signal mechanism already preserves this
composition. No new production opcode, backend handler, source-name dispatch,
ABI or pure-helper authority is needed.

## Acceptance And Tensor

Focused serial acceptance passes 244 distinct tests, including eight new tests:
196 compiler tests, four native/reference tests (three new and one leading-parent
reference replay), thirteen pure/counting/typed-carry regressions, one CLI delivery
test, three host fault tests and 27 tensor tests.
Changed Rust formatting, UTF-8, relative links and source/test/document line caps
pass. All 3,384 drift checks pass without duplicate IDs.

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
All six complete predecessor fields/hashes are retained and old validation
commands remain quoted inert history. The rebuilt ordinary CLI matches source
fields, complete hashes and the ready task card, with clean drift/coverage/
manifest/milestone/hierarchy/lineage status.

| Predecessor Field | Complete SHA-256 |
| --- | --- |
| `evidence` | `07d5ab54eb4779f1c60e0cced146c19fe59b7acf80427fc2d101af3a47b60687` |
| `next_step` | `61740e0682020358694e2dd72ba9b627049edab5edde202b12d8493c31583a53` |
| `blocker` | `d05a7e86a3bf8482ccea0866de0ef60dd73f1c49c4e59c3cf614acd96b684a6b` |
| `next_action` | `f7cc2edfd1bb77f56c5502509573cb48b5dd5152c9f8284e5bc4a82f08e8ba30` |
| `validation_command` | `4007fbe82af62ae7351df58389866a8933bb297baa627d8b2b78b96edd8daf2d` |
| `expected_artifact` | `f5c0e54b72e75b7b5247e238b916eea873d7989fdd1b6c39b833d878a3dfdbef` |

The [parent-continue follow-up](nuis-native-stateful-loop-return-parent-continue-v1.md)
now composes these selected typed returns with both parent step placements,
completed continuing regions and later reached reservations. Independent return storage remains the baseline;
effectful snapshot/coalescing requires its own prefix-preservation proof.
General I/O/resources, asynchronous effects, broader entering-State writes,
global immutability, stable ABI, full heterogeneous closure and completed
self-hosting remain separate. This receipt is not full-workspace/strict-Clippy,
new Linux/GPU/Windows, performance, formal-verification or global-memory-safety
acceptance.
