# Native Stateful Parent-Continue Typed Loop Returns V1

This bounded CPU receipt advances the
[trailing-parent typed-return proof](nuis-native-stateful-loop-return-parent-steps-v1.md).
The Git baseline remains beta-0.17.3, commit 81dc1669. No release bump or stable
ABI is declared.

## Parent Continue And Selected Returns

[The real Nuis source](../../tools/nuisc/tests/native_application_bridge/state_loop_return_parent_continue.ns)
adds a parent-local continue after the conditional child and before the parent
return/suffix. Print 25 marks completed continuing work; current total gains
the current parent index. A trailing parent performs its canonical source step
immediately before continue. A leading parent already stepped at body entry and
does not increment again. Child returns bypass that entire continuing region.

Thirty-two source variants cover bool/f32/f64/flat mixed-record returns, both
parent step placements, both child step placements and original/mirrored guards.
The source-level oracle models ordinary continue and immediate return, not the
generated running/pending bits. Its previous defaults stay unchanged, and both
preceding leading-parent and trailing-parent reference families are replayed.

The immediate child return skips print 25, the parent return and the parent tail.
The late-entry witness first takes an inactive child, completes print 25 and
continues. Its next iteration reaches the child return before any poisoned
parent suffix. Its exact source trace is:

```text
5 8 9 25 8 6 16 7 10 11 12 30 35
```

The return callee observes parent index 1 for trailing parents and 2 for leading
parents. Record callback totals are 111/122 with a trailing parent and 213/224
with a leading parent, for trailing/leading children respectively.
These hand-fixed witnesses supplement the independent oracle.

A separate witness completes a child-local continue and the rest of that child
before a parent-local continue; the child exit cannot accidentally advance or
exit its parent. Empty/negative children and fully completed children retain
their own control scope. Source work after terminal continue, missing/wrong
pre-continue steps, stray induction/parameter writes, escaped child-local width
and ungranted computed prints remain rejected.

## Completed Prefix And Demand Reservations

The entire reached parent bound is reserved even when a child immediately
returns. Each selected child invocation reserves its full reached bound from
the remaining callback budget, with no exit refund.
Limit 4 rejects the late witness after the completed parent continue and the
second-iteration extent/print 7; limit 5 accepts. Reservation failure cannot
erase that completed print-25 prefix.

An unselected single parent round with zero-divisor or division-overflow child
inputs returns through continue/fallback without evaluating the child bound,
parent return or poisoned suffix. Its trace is `5 8 9 25 31 35`.
Native limit 1 accepts these paths; the independent oracle also rejects limit 0
before the body. A zero-parent loop consumes no body/child work.

The native budget subset uses eight result/parent-step representatives at
limits 1/4/5. It checks 528 healthy non-close callbacks and
216 reservation failures after 648 accepted prefix callbacks.
Every successful candidate after a real three-callback prefix is replayed as
a healthy native sequence; it is not silently dropped by the failure filter.

## Execution And Delivery

[Native/reference tests](../../tools/nuisc/tests/native_application_bridge/state_loop_return_parent_continue.rs)
exercise ordinary and reversed YIR storage order, actual prior-State handoff and
six raw floating-word pairs: signed zeros, signed subnormals and NaN payloads.
Each main execution route checks 10,496 healthy non-close callbacks plus 448
failing events after 1,344 accepted prefix callbacks.
Observers enter the actual extent/return callees, not replacement business code.
Event input deliberately differs from prior State.

Native probes copy actual output into the next input and check status, all State
words and output canaries. Fatal probes compare flushed effect prefixes through
SIGILL/SIGTRAP; this is not post-abort memory inspection.
Reference faults preserve the prior published State, enter Faulted, reject later
events/completion and allow explicit close.
Late reached bounds, root 0, unused-proof root 2, body checks, parent suffixes
and callback tails are distinct selected failure paths.

[AOT controls](../../tools/nuisc/src/aot_application_effect_call_state_loop_return_parent_continue_tests.rs)
check exact State layout, every individual literal-print grant and Effect order,
without owned-aggregate heap transport.
[Unit controls](../../tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects_return_parent_continue_tests.rs)
check all 32 variants, fixed traces/totals and negative authority controls.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_parent_continue.rs)
uses four result-kind representatives spanning both parent/child step placements
and guard polarities. Each goes through fresh, cache-hit and two source-free
restorations, for sixteen delivered images.
Six healthy sessions precede each image's failures and one replays afterward:
448 healthy non-close callbacks, 112 closes and 64 fatal events after 192
accepted prefix callbacks. Signed-zero/subnormal decimal output is parsed back
to exact float words; NaN-payload evidence is the native word route.
Fatal CLI checks assert no failed-State publication or successful completion;
stdio-buffered abort output is not claimed as an exact effect-prefix receipt.

Source, project manifest/cache and original build output are absent before
restoration. Binary/container bytes, host objects, envelope, lifecycle and
packaging mode remain unchanged.

The established generic branch/loop return normalization already implements
this composition. No production opcode, backend handler, source-name dispatch,
ABI or pure-helper authority was added.

## Acceptance And Tensor

Focused serial acceptance passes 249 distinct tests, eight new:
200 compiler tests, five native/reference tests (three new and two preceding
reference replays), thirteen pure/counting/typed-carry regressions, one CLI
delivery test, three host fault tests and 27 tensor tests.
All 3,393 drift checks, changed-Rust formatting, UTF-8, relative links and
changed source/test/document line caps pass.

The coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
All six complete predecessor fields/hashes are preserved. Historical validation
commands are quoted inert data. The rebuilt ordinary CLI matches source fields,
complete predecessor hashes and the ready task card, with clean drift/coverage/
manifest/milestone/hierarchy/lineage status.

| Predecessor Field | Complete SHA-256 |
| --- | --- |
| `evidence` | `67290d261a594324605a174a3a09395072cfd2f17c97a62dd94fb43c562c56d3` |
| `next_step` | `9e07b2e3c24ecc9831eb6cfc7183044b1939dcebda47523000adaae02a5544b3` |
| `blocker` | `3701a80d9cb779237bfe1c4d4f45d01378e959b177aff4b03cc9cbd33dd7a269` |
| `next_action` | `7794ab29b87602d5cb53263d5050bcc0571ca0f124c41296fbe1173efc2e114c` |
| `validation_command` | `72990498fe0b0051011d7b24445582c519258f91eea760d4178b179e3f23a437` |
| `expected_artifact` | `3f5aed3cfbf4944535d9c72b300944292f33312da19980c5d6facf6767d778a8` |

The [parent-break follow-up](nuis-native-stateful-loop-return-parent-break-v1.md)
now distinguishes ordinary break/fallback from real false/zero child returns,
with live post-loop indices, selected fallback traps and sixteen delivery images.
Mixed parent-local break/continue beside these selected returns remains separate.
Independent return payload/signal storage remains the baseline until an
effect-prefix proof permits coalescing.
General I/O/resources, asynchronous effects, broader entering-State writes,
global immutability, stable ABI, full heterogeneous closure and completed
self-hosting remain separate. This is not full-workspace/strict-Clippy, new
Linux/GPU/Windows, performance, formal-verification or global-memory-safety
acceptance.
