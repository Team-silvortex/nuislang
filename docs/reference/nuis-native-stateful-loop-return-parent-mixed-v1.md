# Native Stateful Mixed-Parent-Exit Typed Loop Returns V1

This bounded CPU receipt advances the
[parent-break proof](nuis-native-stateful-loop-return-parent-break-v1.md).
The Git baseline remains beta-0.17.3, commit 81dc1669. There is no release bump
or stable-ABI declaration.

## Mixed Parent Exits And Real Returns

[The real Nuis source](../../tools/nuisc/tests/native_application_bridge/state_loop_return_parent_mixed.ns)
contains ordinary parent-local break and continue beside a selected typed child
return in the same function. Break precedes continue in source order, with
independent input selectors. Both regions print 25 before their local exit.

Thirty-two variants cover bool/f32/f64/flat mixed-record results, leading/trailing
parent and child steps and original/mirrored guards. This is a mixed composition
proof, not an inference from two separate exit receipts.
A real return, including false or signed zero, bypasses both parent-local exits
and the entire ordinary fallback. A break must reach that fallback; a continue
must skip only the current parent suffix and take exactly its source step.

[Structural controls](../../tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects_return_parent_mixed_tests.rs)
trace the actual parent helper and require its break control to differ from the
private function-return pending bit. The existing return-ownership proof refuses
to share that bit with ordinary breaks; generic mixed-exit normalization keeps
its running and breaking decisions separate.
No production opcode, backend/name dispatch, ABI, helper authority or coalescing
optimization is added. Independent typed payload/signal storage remains intact.

## Observable Steps And Fallback

A first parent continue followed by a selected child return has fixed trace:

```text
5 8 9 25 8 6 16 7 10 11 12 30 35
```

There is no ordinary parent suffix, print 26, checkpoint or fallback.
The fallback's finish-zero input is deliberately poisoned. The real return
callee observes parent index 1/2 for trailing/leading parents; record totals
are 111/122 for a trailing parent and 213/224 for a leading parent, for
trailing/leading children respectively. Scalar-result callbacks publish only
their selected field and retain the remaining prior-State fields.

A continue followed by an ordinary break with an empty reached child has trace:

```text
5 8 9 25 8 6 16 7 25 31 33 32 35
```

The real checkpoint observes parent index 1/2. Thus a trailing continue cannot
lose its one step and a leading continue cannot add a second step.
The break must not take another step or invent a function return.
If both local exit selectors match the first round, the source-order break
wins: trace `5 8 9 25 31 33 32 35`, observed checkpoint index 0/1.
The inactive poisoned child bound is not evaluated.

After print 31 a real checkpoint call prints 33; the unused division
`1 / finish` must still execute before print 32 and the typed fallback.
A finish-zero failure reaches neither print 32 nor callback publication.
Completed child-local continue effects precede both parent-local exit regions
and remain child-scoped. Empty/negative children and unused-proof finish 2
retain their source semantics.
Unit controls also reject source work after terminal exits, extra parent-index
writes, parameter writes, escaped child width and computed print authority.
Fixed witnesses supplement the independent source-level oracle, which uses
ordinary Rust break/continue/return rather than generated control flags.

## Reservations And Faults

The entered parent reserves its whole bound of 2 even for a first-round break.
Native limit 2 accepts an unselected poisoned child and a continue/empty-child/
break path; the source oracle rejects limit 1 before entering the parent.
Only reached child invocations evaluate and reserve their full bound from the
remaining callback budget. An early return or local exit gives no refund.

The budget subset uses eight result/parent-step representatives at limits 2/4/5:
816 healthy non-close callbacks and 192 reservation failures after 576 accepted
prefix callbacks. Every successful after-prefix candidate is replayed.
A zero-parent invocation still executes its fallback.

[Native/reference tests](../../tools/nuisc/tests/native_application_bridge/state_loop_return_parent_mixed.rs)
check normal/reversed YIR storage order. Each main route checks 8,192 healthy
non-close callbacks and 512 failing events after 1,536 accepted prefix callbacks.
Six raw float-word pairs include signed zeros, signed subnormals and quiet/
signaling NaN payloads. Observers enter the real extent, return and checkpoint
callees. Native output is copied into the next input, never replaced with
expected State. Event scalar input deliberately differs from the actual State.

Native successes check status, complete State words and output canaries.
Fatal probes compare flushed effect prefixes through SIGILL/SIGTRAP; this is
not post-abort memory inspection. Reference faults retain the last accepted
State, enter Faulted, reject later events/completion and allow explicit close.
Failures distinguish reached bound checks after a completed continue, root 0,
unused-proof root 2, child-body checks, real fallback checks, ordinary suffixes
and callback tails.

A bounded preceding parent-break replay checks four result representatives
through 96 healthy native callbacks and 32 fatal events after 96 prefix
callbacks, in both storage orders. This is not a full rerun of all preceding
native/reference matrices.
[AOT controls](../../tools/nuisc/src/aot_application_effect_call_state_loop_return_parent_mixed_tests.rs)
preserve exact State layout, each individual print grant, Effect ordering and
no owned-aggregate heap transport.

## Delivery And Acceptance

[CLI delivery](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_parent_mixed.rs)
uses four result representatives spanning both step placements and guard
polarities. Fresh, cache-hit and two source-free restorations produce
sixteen delivered images. Five healthy sessions precede each image's five
fatal-event checks; one healthy session replays afterward.
The profiles include continued-then-break and continued-then-return paths.
There are 384 healthy non-close callbacks, 96 closes and 80 fatal events after
240 accepted prefix callbacks. Float text is parsed back to exact signed-zero/
subnormal words; NaN payloads use the native word route.
Fatal CLI checks assert no failed-State publication or successful completion,
not exact buffered abort stdout.

Source, project manifest/cache and original build output are removed before
restoration. Binary/container bytes, host objects, envelope, lifecycle and
packaging mode remain unchanged.

Focused serial acceptance passes 256 distinct tests, nine new:
208 compiler tests, four native/reference tests including the bounded preceding
break replay, thirteen pure/counting/typed-carry regressions, one CLI delivery
test, three host fault tests and 27 tensor tests.
All 3,411 drift checks, changed-Rust formatting, UTF-8, relative links and
changed source/test/document line caps pass.

## Tensor Lineage

The coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
All six complete predecessor fields/hashes are retained, with old validation
commands quoted as inert history. The rebuilt ordinary CLI matches source
fields, complete predecessor hashes and the ready task card, with clean drift/
coverage/manifest/milestone/hierarchy/lineage status.

| Predecessor Field | Complete SHA-256 |
| --- | --- |
| `evidence` | `592e9a8ceadffe478ab6eed46bf51db37e03b7e3c860beaee188a92616420a2b` |
| `next_step` | `c8a6001f038e3dd0b98b4ba3e9548fbed08e16b6bdd9b37859d237ea12aea754` |
| `blocker` | `125acde6402e397975eb08d5d0b50e516e95f4e67fd4728275b4c163ded9c421` |
| `next_action` | `cd73aad96470d0ee9304361b64c616a6f8e80f11097d6b342e40a0bc9b071f84` |
| `validation_command` | `feb2c5c55f40a94f086af194207007ce396880e1a62e376037f11a0b1aee8186` |
| `expected_artifact` | `8810c5b685354b0f20ed791de760a0e687bc809ea0e7f74b98a14044d8be87f7` |

Next prove effect-prefix preservation before attempting to coalesce typed
effectful return snapshots: complete selected checks/calls, local exits and
pending publication must remain source-ordered with no replay.
This receipt does not enable that optimization.
General I/O/resources, asynchronous effects, broader entering-State writes,
global immutability, stable ABI, full heterogeneous closure and completed
self-hosting remain separate. This is not full-workspace/strict-Clippy, a fresh
Linux/GPU/Windows run, performance, formal or global-memory-safety acceptance.
