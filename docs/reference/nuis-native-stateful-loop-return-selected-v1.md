# Native Stateful Selected Typed Loop Returns V1

This bounded CPU composition receipt advances the
[typed loop-return proof](nuis-native-stateful-loop-return-values-v1.md).
Git remains authoritative at beta-0.17.2, commit 00ab72d3; no release bump or
stable-ABI claim is made.

## Demand-Only Child Entry

[The real Nuis program](../../tools/nuisc/tests/native_application_bridge/state_loop_return_selected.ns)
derives child selection from the callback's current total and actual prior bool.
A selected arm prints 6, calls the real effectful `extent(trips, divisor)`
helper, prints 7 and only then reaches the child's reservation.
The callee prints 16 before dividing. Neither that call nor its arithmetic or
child reservation may run in the unselected arm.

An independent [source-level oracle](../../tools/nuisc/tests/native_application_bridge/state_loop_return_selected_cases.rs)
models ordinary source branches, immediate returns and shared full-bound
reservations, not generated guards or pending-return signals.
Source generation reuses the prior proof's exact bool/f32/f64/mixed-record
signatures and independently mirrors the child-entry branch.
All sixteen combinations of result kind, child step and branch polarity execute.
The parent remains leading; this is not trailing-parent acceptance.

Inactive zero-divisor/oversized and signed division-overflow bounds succeed,
even with poisoned return operands, under a one-unit parent budget in the oracle.
The native budget matrix separately executes inactive controls with limits 4/5.
Selected versions fail after print 16 and before print 7.
Reached child bounds reserve their complete positive extent even when the first
child step immediately returns. An early return never refunds that reservation.

## Selected Returns And Prior Progress

The child updates current integer/bool/f32/f64 values before its selected return.
The existing typed payload and independent pending signal escape both child and
parent loops without executing their suffixes. False and zero payloads remain
real exits, not fallthrough seeds.
A parent return can instead follow a skipped, empty, negative or completed child.

The late-entry witness starts at total 3, completes an unselected parent round,
then enters the child in round 2 and returns. Its exact source print sequence is:

```text
5 8 9 23 21 22 8 6 16 7 10 11 12 30 35
```

Record results are 213 for trailing child steps and 224 for leading steps.
Limits 4/5 reject/accept the full reached reservations respectively; the failing
prefix retains the first completed parent round and stops after the second
round's bound helper and print 7, before any child-body print.
The original immediate-child record witness still yields 109/120.

Later events intentionally supply different complete scalar argument values;
work consumes the actually published four-word State instead.
Root 0 fails before the real return callee, root 2 fails after its print 30,
and child-body, parent-suffix and callback-tail failures retain their own prefixes.
An unused scalar proof inside the callee must still trap.

## Execution And Delivery

[Unit checks](../../tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects_return_selected_tests.rs)
retain isolation from the pure catalog and reject branch-local width escape,
bound/index/parameter writes, computed prints, wrong return kinds and statements
after terminal continue. [AOT checks](../../tools/nuisc/src/aot_application_effect_call_state_loop_return_selected_tests.rs)
retain the exact State layout, per-site literal-print grants and Effect ordering.

[Reference/native tests](../../tools/nuisc/tests/native_application_bridge/state_loop_return_selected.rs)
execute both ordinary and reversed YIR storage order.
The main matrix has 8,064 healthy non-close callbacks, plus 224 failing events
after 672 accepted prefix callbacks, separately on each execution route.
Six raw float-word pairs include signed zero, signed subnormals and NaN payloads.
Observers at the real bound and return callee entries check actual arguments.
Native probes copy the actual output words into the next input, and check success
status, output canaries and complete State words. Fatal native checks compare
flushed prefixes up to SIGILL/SIGTRAP; this is not post-abort memory inspection.
Reference faults retain the last State, reject subsequent events/completion and
permit explicit close without publishing a failed result.

The separate native budget matrix checks 320 healthy non-close callbacks and
104 reservation failures after 312 accepted prefix callbacks.
It covers zero parents, inactive poisoned/overflowing bounds, huge reached
parents/children, first-child returns and later selected children.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_selected.rs)
executes eight diagonal source variants, each through fresh, cache-hit and two
source-free restorations: thirty-two delivered images.
Across these images it checks 1,280 healthy non-close callbacks and 128 fatal
events after 384 accepted prefix callbacks. Healthy sessions run before and
after failures, with exact output and raw signed-zero/subnormal parse-back.
NaN payload acceptance comes from word probes, not decimal CLI formatting.
Source, manifest, project cache and original build output are deleted before
restoration. Binary/container bytes, host objects, lifecycle, envelope and
packaging mode remain unchanged.

No production opcode, backend-specific handler, source-name dispatch or new ABI
is required: this composition uses the existing shared branch/loop return signal
and exact ordered word-operation admission.

## Acceptance And Tensor

Focused serial acceptance passes 242 distinct tests, including eight new tests:
192 compiler tests, six native/reference tests (three new and three replayed
typed-return tests), thirteen pure/counting/typed-carry regressions, one CLI
delivery test, three host fault tests and 27 tensor tests.
Changed Rust formatting, UTF-8, relative links and source/test/document line caps
pass. All 3,375 drift checks pass with no duplicate IDs.

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
All six complete predecessor fields and hashes are retained, with previous
validation commands quoted as inert here-document data.
The rebuilt ordinary CLI must match source fields and complete history, retain a
ready selected task card and report clean drift/coverage/manifest/milestone/
hierarchy/lineage status.

| Predecessor Field | Complete SHA-256 |
| --- | --- |
| `evidence` | `6f74b0f579736823d473503a5d8d65435fd9c86e4552861e8aef3e6b17fa18df` |
| `next_step` | `7f22575cf106aa27d758cf012064d4b01b613f6632c833150fb770aef1745988` |
| `blocker` | `ceff3467f9a42b470facb2d590c114f37848de22617d04358a2008074a7baa57` |
| `next_action` | `408661e3db59d0d022d528148dd32ed29cab5958f1882372faf3b3c0525ec374` |
| `validation_command` | `59251682c4f314022aa6ceff5b2e28e000b11bc36e27fed230b92652d7216b05` |
| `expected_artifact` | `c7875fdfa16c550315c45d0aab762375a5662613e8485e78c5e428b2ee07f545` |

The [trailing-parent follow-up](nuis-native-stateful-loop-return-parent-steps-v1.md)
now checks these selected typed child returns with exact parent indices, completed
and skipped tail-step regions, shared reservations and sixteen delivery images.
The [parent-continue follow-up](nuis-native-stateful-loop-return-parent-continue-v1.md)
now checks both parent step placements, completed continues and later reached
reservations. The [parent-break follow-up](nuis-native-stateful-loop-return-parent-break-v1.md)
now checks independent local break and typed fallback. The
[mixed-exit follow-up](nuis-native-stateful-loop-return-parent-mixed-v1.md)
now composes break/continue beside these returns in one function. Effect-prefix
proof before effectful snapshot coalescing remains separate.
General I/O/resources, asynchronous capture, broader entering-State writes,
global immutability, full heterogeneous closure and completed self-hosting remain
separate. No new Linux/GPU/Windows run, full-workspace/strict-Clippy acceptance,
performance improvement or formal/global-memory-safety proof is claimed here.
