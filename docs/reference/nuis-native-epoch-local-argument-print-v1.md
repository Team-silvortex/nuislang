# Native Epoch-Local Argument Print V1

Audit date: 2026-11-04. Checkout baseline: `dc87ec43` (`fixed license`,
also retaining the preceding implementation). Latest version-label commit:
beta-0.18.4, `ceaa2caa`.
This worktree connects the [borrowed initializer plan](nuis-native-epoch-local-plan-v1.md)
to source admission. That earlier analysis-only receipt retains its original
scope, executed totals and predecessor hashes. No release is created here.

## Root-Owned Admission

The [root iteration validator](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs)
first tries ordinary initialization unchanged. Only an otherwise-unready
initializer, root depth zero and explicit scalar-print mode may request a plan
for its complete validated iteration-effects block and fresh entry epoch.
Branches and child iterations do not gain this new admission.

The existing plan requires one fresh plain i64 definition, literal markers only
before it, and one adjacent pure i64 observer with an exact f32/f64 argument.
The word has one lazy comparison-scoped use; forwarding/arithmetic retains its
restriction. Later aliases, prints, writes, returns, repeated occurrences,
cross-kind helpers and non-lazy use remain rejected. Other already-ready local
forms keep their earlier ordinary rules; this is not a ban on ordinary branch
locals or a general pre-update read grant.

The [source gate](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch_local_plan.rs)
checks identity of the actual block, epoch, scope, layouts and helper catalog,
plus definition/consumer indices and the shared bounds. A temporary type-only
scope validates the observer through the existing print-shape, purity and type
gate. Neither the new word nor its original record is published into ordinary
`Locals` scope/availability/write authority. The matched observer is consumed
with its definition as one certificate, not rechecked under ambient readiness.

The eager initializer additionally passes the ordinary non-logical expression
shape gate. It cannot use the plan's comparison context to introduce hidden
short-circuit initializer trees outside the function-wide print-root budget.
Original global logical-root, work/depth/capture and native execution limits
are retained; analysis eligibility alone is not source admission.

## Existing Normalization, Once In Place

The [Nuis fixture](../../tools/nuisc/tests/native_application_bridge/epoch_local_argument_print.ns)
initializes its word from the updating record's original i32 field, signed
widening, full-width XOR and a checked pure helper. Its next observer consumes
that word inside a guarded comparison. The record changes only afterward.

No new opcode, runtime ABI, source substitution or special backend emitter is
introduced. The existing scoped iteration/effect normalizer receives both
original statements in their original order. The initializer is not cloned into
the observer or hoisted into its lazy RHS. A false later gate may skip the use,
but cannot suppress initialization or an initializer fault. Zero-trip loops
still skip body work and execute their reached tail normally.

Each iteration observes its own entry field. Callback events use accepted State
rather than a decoy initial-value parameter. The witness also faults inside a
later record update and at the final return, checking last accepted State and
faulted close independently of the word's comparison result.

## Execution And Delivery

The [oracle](../../tools/nuisc/tests/native_application_bridge/epoch_local_argument_print_cases.rs)
reuses the preceding independent bit-by-bit XOR model and computes the new eager
definition/guard/update schedule separately. Sixteen raw sequences per float
width cover signed/zero/subnormal/infinite/quiet-NaN State, high-bit masks,
changing i32 entry fields and zero-valued words. Twelve CLI sequences per width
use finite parameters. Nine fault rows distinguish initializer, reached lazy
use, record update and tail failures; a zero word does not skip a reached guard.

Five [native/reference controls](../../tools/nuisc/tests/native_application_bridge/epoch_local_argument_print.rs)
check normal/reversed YIR, real checked64 entries and full-width arguments,
flushed fault prefixes, raw State words, canaries, counts and faulted close.
The witness keeps loop work 3 and helper entries 64. A separate limit-8 native
probe must reject before callback success/State publication. Actual compile/emit
refusals retain alias/duplicate/non-lazy/nonroot boundaries, not merely catalog
membership tests.

The [CLI matrix](../../tools/nuis/tests/native_session_workflow/epoch_local_argument_print.rs)
uses uninstrumented production artifacts: fresh/cache execution, followed by two
independent restores after removing source, project manifest and project cache.
Binary/container and host-object/lifecycle/envelope/packaging identities remain
checked, with healthy runs repeated after faults. Direct probes prove the exact
flushed trap prefix; the CLI only checks any output its trapping child flushed.

## Validation

The final source state passed 304 distinct selected tests/checks:

| Selected Set | Passed |
| --- | ---: |
| Compiler temporaries/epoch/iteration-print/effect library regressions | 216 |
| Epoch-local, ready-local, i64 XOR and i64 arithmetic native controls | 17 |
| f32/f64 fresh/cache/two source-free delivery matrix | 1 |
| Python CI-script regressions | 40 |
| Repository file budgets | 2 |
| Host-absolute-path policy | 1 |
| Development tensor tests | 27 |

Nine controls are new relative to the preceding plan receipt: two source tests,
one owner-context identity test, five native/reference controls and one CLI
matrix. The seven preceding plan tests remain; the source-admission assertion
is intentionally advanced without granting ordinary local availability. Earlier
receipts, raw matrix rows and repeated runs are not added to the 304 total.

Structural validation also passed: 5,997 UTF-8 text files, 4,677 local Markdown
links, offline locked metadata, formatting and whitespace checks. The workspace
architecture remains 29 packages, six domains and 74 normal/build edges. The
proof index stays at 300 lines; the iteration-check registry is 778 lines. The
rebuilt ordinary CLI report matches all six source fields and complete
predecessor hashes; all 4,164 drift checks passed. Coverage, hierarchy and task
lineage are clean, and the selected coordinate remains active/99.

This is bounded native host CPU execution, not GPU/provider parity, stable ABI,
arbitrary resource/I/O/async capture, whole-engine completion or self-hosting.

## Next Gate

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. Next is fresh child-owned epoch pairs: distinguish an actual child
entry from an ordinary branch reusing its parent's epoch, require a ready entry
source and reject ancestor-grant inheritance before new child admission. Wider
prefixes, multiple definitions/uses and arbitrary escapes retain separate gates.

The subsequent [child-owned receipt](nuis-native-child-epoch-local-argument-print-v1.md)
advances that child-entry gate separately; this dated root receipt retains its
original acceptance scope and counts.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- temporaries:: print_epoch:: iteration_prints:: iteration_effects:: --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- epoch_local_argument_print:: local_scalar_argument_print:: i64_xor_argument_print_epoch:: i64_arithmetic_argument_print_epoch:: --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- epoch_local_argument_print --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete preceding tensor fields remain lossless alongside their
SHA-256 identities. Historical commands are quoted no-op here-document data,
checked through stdin without replaying old work.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `059f0efc97c8de519e29f2caa94b02382772abcaac69ed8301cd143911890a7f` |
| `next_step` | `3599d9a5194780eedcdbeb7ce55f3e0609113f71504e46a6deb867c87234c322` |
| `blocker` | `04ba02f922bf355d5578de8676b40962d853c85662fa2937ecc19d39a751022f` |
| `next_action` | `36e2a5539c148fa31ad44497c98910dfc1d88b6ecea447da6b3007f6b1d6d9ce` |
| `validation_command` | `a7d6034bc4d843f3743a37a32c9ce8d54351cd135405f522f23016213997f77b` |
| `expected_artifact` | `fc48adb511961875f6f281d2979da4754ce53ca24a146af31a35f337ca586189` |
