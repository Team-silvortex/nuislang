# Native Stateful Effectful Loop Exits V1

This receipt advances the
[direct loop-statement checkpoint](nuis-native-stateful-loop-statements-v1.md).
The Git baseline remains beta-0.17.1, commit 99ebfcb2. It is a bounded CPU
composition proof, not a new ABI, blanket I/O permission or release bump.

## Existing Lowering, Stronger Composition

The existing explicit literal-effect route already admits structured loop
exits. This follow-up adds executable evidence instead of introducing another
special-case lowering path or helper-name whitelist. Source admission and
outlining continue to use the same normalized flags, scoped function boundaries,
exact write/type/header authority and backend grant/order checks.

[The Nuis program](../../tools/nuisc/tests/native_application_bridge/state_loop_exits.ns)
has a state-derived break/continue choice after a real carry update. It prints
10 at iteration entry, 11 at the exit selector, 12 before break, or 13 before
continue. Otherwise it prints 19, evaluates checked refine arguments, performs
real work and prints 20. Print 31 follows the entire loop. Advance/finish/answer
effects remain 5/35/40.

Four [source variants](../../tools/nuisc/tests/native_application_bridge/state_loop_exits_cases.rs)
combine leading/trailing induction with original/mirrored selection. A trailing
continue has its exact step immediately before the terminal statement; the
normalizer removes only that duplicate and lets the driver advance once.
Leading continue skips the suffix after the already-completed step. Break
suppresses the driver backedge; observed leading indices use existing recovery.

The source calls `observe(current, index)` after the loop, and the actual index
contributes ten times its value to the returned computation. Native observers
also record these two actual arguments independently. Missing/double induction
cannot be hidden by checking only the final bool or the carried count.

[Unit controls](../../tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects_exits_tests.rs)
check pure exclusion before/after outlining, one canonical break control,
recovery only for observed leading indices and hand-computed distinct witnesses:
the same seed/three trips produce continue totals 81 versus 86 and break totals
35 versus 23 for trailing versus leading induction. Unstepped/wrong-step
continues, prints inserted between step/continue, work after terminal control,
computed prints and loop-header writes are rejected at parsing/admission.
Return-in-loop effect closures remain outside the isolated capability.

## Real Sessions And Failure Prefixes

The independent oracle models source `while`, `break` and `continue` directly,
not compiler-generated flags or transport. Its 96 signed seed/gate/flag/trip/cut
cases, four raw markers and two proof modes give 768 three-event sequences per
source/inventory order. Calls use actual previous callback output; the expected
State is never injected as the entering state. Repeated complete event inputs
still select control from the changing prior bool.

Negative/zero trips execute no loop effects or body proofs. Runtime one/three
trips cover first/later exits, continuing work after skipped iterations, and
normal loop completion. Separate poisoned work holes at the exit cut are safe
only because the selected break/continue skips the checked argument. Valid
proof arithmetic may differ from the all-valid invocation; the oracle does not
assume the two proof modes have identical State.

[Reference/native tests](../../tools/nuisc/tests/native_application_bridge/state_loop_exits.rs)
check exact real print traces, current refine arguments, final current/index
arguments, status/canaries and all State words in normal/reversed YIR inventory
order. Raw positive/negative zero, minimum subnormal and signed quiet-NaN marker
payloads are retained.

Eight main healthy native processes contribute 30720 healthy accepted callbacks.
Trailing variants have eleven selected arithmetic failures and leading variants
ten; across both source polarities/inventory orders this is 84 fatal processes
and 252 earlier accepted callbacks. Selected argument failures after completed
iterations or a continue preserve their exact print prefix and suppress later
work/common-tail/publication. The prior accepted State comes from three
successful zero-trip callbacks, not a supplied golden-state initializer.

Reference faults retain the complete last State, reject later events and
cannot finish successfully after close. Native fatal probes prove flushed
prefixes and absence of later publication, not post-abort memory inspection.
Completed source effects are not rolled back.

## Reservations, Grants And Delivery

The explicit main policy is three loop-work units and 100 helper entries per
callback. An early break is not permission to bypass the full declared bound:
even a first-iteration break candidate is rejected at limits zero/two if its
declared count exceeds that limit. Counts 65537 and i64::MAX reject after advance
but before print 10, with valid root/work arguments to prevent a masking trap.
Zero-trip controls remain valid at limit zero; one-trip controls work at limit
two. The separate budget cohort contributes sixteen healthy native processes,
120 accepted callbacks, 56 fatal processes and 56 prior accepted callbacks.
The existing ten-second native child deadline remains in force.

[AOT checks](../../tools/nuisc/src/aot_application_effect_call_state_loop_exits_tests.rs)
require the exact three-field State, retained runtime loop structure and no
owned aggregate allocation. Missing any single print grant, the default
ungranted mode or removed Effect ordering rejects. No pure/Buffer authority,
new source mutability or effectful return-in-loop authority is introduced.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_exits.rs)
checks four sources through fresh/cache/two source-free restored images:
sixteen images, 640 healthy accepted callbacks, 72 fatal events with 216 prior
accepted callbacks, and sixteen oversized-script rejections before open.
Each image reruns healthy sessions before and after fatal scripts. No failed,
next-event or close State is published after a fatal event. The four finite CLI
marker/trip/proof witnesses are not a full cross-product or raw NaN input proof.
The flushed native probe, not buffered fatal CLI stdout, supplies prefix proof.

Executable/host-object/envelope/lifecycle/exact packaging-mode identity is
checked across cache and both restores. Source, manifest, original output and
project cache are removed before restoration. Artifact/container/executable
bytes remain unchanged after execution and restoration.

## Acceptance And Lineage

This fresh serial macOS CPU cohort passes 207 distinct selected tests:
163 compiler, thirteen native/reference/oracle, one new CLI delivery, three host
and 27 tensor. Eight tests are new: three unit/oracle, one AOT, three native/
reference/budget and one CLI. Focused reruns/child processes are not extra tests.
The native cohort replays all three preceding loop families. The previous
668-test/48-image receipt remains historical; it is not counted as a new full
replay here.

Run serially from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- iteration_effects control_loops scalar_helpers scalar_control state_loop --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_loop:: state_loop_effects:: state_loop_statements:: state_loop_exits:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- effect_calls::state_loop_exits:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

All 3313 drift checks /22898 patterns, changed Rust formatting, UTF-8,
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
| `evidence` | `7de4af45209fb39855af4548ba19fb5d951c4d21d1f694848781e8c67db215d3` |
| `next_step` | `a364e65029eb978c34b222e127669229e10476cbde74ff442482949957b96de0` |
| `blocker` | `c9c8bf6bfa00aee6f0411d051819a985b8231901d7053f0607fd3156601a5a4b` |
| `next_action` | `c5b266104d3996e82064323455fe617cfdfe340fd55f74c41868648d39d2b335` |
| `validation_command` | `7afb79a6a8640b32cd456ff0e5928ca3dd6b2a2272016f0637c53fc1929dcb59` |
| `expected_artifact` | `783e968b3623a00cad3a3fd7164cd975e55caf1dc80515730c4932e6039b7e9c` |

The rebuilt ordinary CLI agrees with all six source values and complete
predecessor hashes. Drift/coverage/manifest/milestone/hierarchy/task-card lineage
remain clean with the same ready selected task card. Bounded acceptance does
not mark the coordinate complete.

## Next Boundary

Compose effectful child loops with their own break/continue under a continuing
parent: child exits must stay local, recovered child indices must feed actual
parent work, and all selected loops must debit the same callback budget.
Require real session/native/source-free evidence, not only admission tests.

General I/O, asynchronous resources, return-in-loop effect closures, broader
entering-State writes, stable ABI, formal/global memory safety, measured
performance, generalized GPU closures and completed self-hosting remain
separate.
