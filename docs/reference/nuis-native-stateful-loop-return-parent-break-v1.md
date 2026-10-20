# Native Stateful Parent-Break Typed Loop Returns V1

This bounded CPU receipt advances the
[parent-continue proof](nuis-native-stateful-loop-return-parent-continue-v1.md).
The Git baseline remains beta-0.17.3, commit 81dc1669. There is no release bump
or stable-ABI declaration.

## Parent Break And Real Returns

[The real Nuis source](../../tools/nuisc/tests/native_application_bridge/state_loop_return_parent_break.ns)
adds a parent-local break after the selected child and before the parent return
or suffix. Print 25 marks that break region. Breaking must leave the loop and
execute the function's ordinary typed fallback, not publish a pending return.

Thirty-two source variants cover bool/f32/f64/flat mixed-record results,
leading/trailing parent and child steps, and original/mirrored guards.
A real child return, including false or signed zero, skips the parent break,
later invocations and the entire fallback. An ordinary break skips its suffix
and remaining iterations but must not skip the fallback.
Both kinds of exit preserve completed source effects.

[Unit controls](../../tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects_return_parent_break_tests.rs)
trace the actual parent loop helper and assert that its canonical break bit is
not the function-return pending bit. They do not require the child's optional
return-signal sharing optimization.
The existing [return-ownership proof](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_signal.rs)
rejects sharing for ordinary local breaks
and falls back to independent loop control. No production opcode, backend/name
dispatch, ABI, helper authority or new optimization is needed.

## Observable Fallback And Step Timing

After print 31, a real Nuis checkpoint call observes the source parent index and
prints 33. The subsequent unused division `1 / finish` must execute before
print 32 and the typed fallback. A finish-zero trap cannot disappear just
because its integer result is unused or the final result type is bool/float.

A first-round break with an unselected poisoned child has source trace:

```text
5 8 9 25 31 33 32 35
```

The actual checkpoint callee observes index 0 for a trailing parent and 1 for a
leading parent. This tests live post-loop index recovery, not just a source
step-shape assertion. A finish-zero failure retains the prefix through print 33
but reaches neither print 32 nor callback publication.
The record fallback publishes prior total minus 5; scalar-result callbacks
retain the prior total plus 1 and publish only their selected typed field.

The opposite witness returns from the first child with finish set to zero:
its trace is `5 8 6 16 7 10 11 12 30 35`, with no fallback/checkpoint call.
A late selected return first completes an ordinary parent suffix. Its source
trace is `5 8 9 23 21 22 [26] 8 6 16 7 10 11 12 30 35`;
print 26 exists only for the trailing-parent tail region.
Observed return indices are 1/2 for trailing/leading parents. Record totals are
111/122 for a trailing parent and 213/224 for a leading parent, for trailing/
leading children respectively. These fixed witnesses supplement the independent
source-level oracle; the oracle does not read generated control signals.

A completed child-local continue stays within its child before the parent
break and fallback. Empty/negative children and unused-proof finish 2 also
execute the proper fallback. Unit controls reject source work after terminal
break/continue, extra parent-index writes, parameter writes, escaped child
width and computed print authority.

## Reservations And Faults

A first-round break still reserves the full entered parent bound of 2.
Native limit 2 accepts an unselected child with zero-divisor/overflow inputs;
the independent oracle rejects limit 1 before the body. The skipped child does
not evaluate its bound or reserve work, and no later child is entered.
A reached child reserves its full bound from the remaining callback budget,
even if its first body returns. There is no exit refund.

The native budget subset uses eight result/parent-step representatives with
limits 2/4/5: 624 healthy non-close callbacks and 192 reservation
failures after 576 accepted prefix callbacks.
Successful after-prefix candidates are all replayed, not discarded by the
failure filter. A zero-parent invocation still executes its ordinary fallback.

[Native/reference tests](../../tools/nuisc/tests/native_application_bridge/state_loop_return_parent_break.rs)
check ordinary/reversed YIR storage order and actual prior-State handoff.
Each main route checks 7,680 healthy non-close callbacks and 512 failing events
after 1,536 accepted prefix callbacks. Six raw floating-word pairs include
signed zeros, signed subnormals and quiet/signaling NaN payloads.
Observers enter the real extent, return and checkpoint callees.
Native output is copied into the next input; expected State is not injected.

Native successes check status, complete State words and output canaries.
Fatal native probes compare flushed prefixes through SIGILL/SIGTRAP; this is
not post-abort memory inspection.
Reference faults retain the prior State, enter Faulted, reject later events/
completion and allow explicit close. Reached bound errors, root 0, unused-proof
root 2, child-body checks, post-break fallback checks, ordinary parent suffixes
and callback tails are distinct selected failure stages.

A bounded preceding parent-continue replay checks four result representatives
through 96 healthy native callbacks and 32 fatal events after 96 prefix
callbacks, in both YIR storage orders. It is not a new full replay of every
preceding native/reference matrix.
[AOT controls](../../tools/nuisc/src/aot_application_effect_call_state_loop_return_parent_break_tests.rs)
retain exact State layout, individual literal-print grants, Effect ordering and
no owned-aggregate heap transport.

## Delivery And Acceptance

[CLI delivery](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_parent_break.rs)
uses four result-kind representatives spanning both parent/child step placements
and guard polarities. Fresh, cache-hit and two source-free restorations produce
sixteen delivered images.
Five healthy sessions precede each image's five fatal-event checks, including
the post-break fallback trap; one healthy session replays afterward.
This checks 384 healthy non-close callbacks, 96 closes and 80 fatal events after
240 accepted prefix callbacks. Signed-zero/subnormal text is parsed back to
exact float words; NaN-payload evidence uses the native word route.
Fatal CLI checks assert no failed-State publication or successful completion,
not exact buffered abort stdout.

Source, project manifest/cache and original build output are absent before
restoration. Binary/container bytes, host objects, envelope, lifecycle and
packaging mode stay unchanged.

Focused serial acceptance passes 252 distinct tests, nine new:
204 compiler tests, four native/reference tests including the bounded preceding
continue replay, thirteen pure/counting/typed-carry regressions, one CLI delivery
test, three host fault tests and 27 tensor tests.
All 3,402 drift checks, changed-Rust formatting, UTF-8, relative links and
changed source/test/document line caps pass.

## Tensor Lineage

The coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
All six complete predecessor fields/hashes are retained, and old validation
commands remain quoted inert history. The rebuilt ordinary CLI matches source
fields, complete hashes and the ready task card, with clean drift/coverage/
manifest/milestone/hierarchy/lineage status.

| Predecessor Field | Complete SHA-256 |
| --- | --- |
| `evidence` | `75cd1349661b18d960a61f0b9c7d1006452e63db61321380ec1c464c0483deaa` |
| `next_step` | `884539102fe48cd3dab78452b55483ad794121e6a7574ede134e24711950a565` |
| `blocker` | `517a525c1376cb1088abcffee6997fb7ae6d023d3d0b907cbfed75f3350b2a76` |
| `next_action` | `75d17545e0b10b1b29f58e2a09e80631954387cfbdf726d061fdc2f80f8aa3ae` |
| `validation_command` | `d0b79666a90bd4c741a3d1cf682107fd5189f574bdcb64663a01180a6b85caf1` |
| `expected_artifact` | `c9aaa5cdb26d4d94171be89e7bea88e56c95d35910dfc2aa1d407b7922a75ddb` |

The [mixed-exit follow-up](nuis-native-stateful-loop-return-parent-mixed-v1.md)
now checks that one-function composition, with exact source steps and observed
fallback indices. The acceptance counts above remain this earlier receipt's own.

The original next boundary was to combine parent-local break and continue beside these selected typed child
returns in one function under both parent step placements, keeping loop break,
continuation and function return independent. Separate receipts for break and
continue are not a proof of that mixed composition.
Independent typed return storage remains the baseline until an effect-prefix
proof permits coalescing. General I/O/resources, asynchronous effects, broader
entering-State writes, global immutability, stable ABI, full heterogeneous
closure and completed self-hosting remain separate. This is not full-workspace/
strict-Clippy, a new Linux/GPU/Windows run, performance, formal-verification or
global-memory-safety acceptance.
