# Native Stateful Parent-Local Loop Exits V1

This bounded CPU receipt advances the
[selected-child invocation checkpoint](nuis-native-stateful-loop-selection-v1.md).
Its Git baseline is beta-0.17.2, commit 00ab72d3. It does not bump the version
or declare a stable ABI.

## Child Work Before Parent Control

[The Nuis source](../../tools/nuisc/tests/native_application_bridge/state_loop_parents.ns)
keeps a leading-step parent and a conditionally selected child. The child still
owns its local break/continue and may use leading or trailing steps.
After either selected or retained child work, `print(23)` precedes
`checkpoint(current, child_index, parent_index)`. This real function computes
`current + 3 * child_index + 7 * parent_index`; its output decides whether
the parent takes its own break/continue.

Parent `print(25)` marks break and `print(26)` marks continue. Both suppress the
parent's remaining `print(21)`/checked-combine/`print(22)` suffix. Break also skips
later parent iterations; continue does not. The final real observer consumes
current value and recovered parent index after the parent loop.

Four [source variants and the independent oracle](../../tools/nuisc/tests/native_application_bridge/state_loop_parents_cases.rs)
combine original/mirrored guards with leading/trailing child steps.
The parent remains leading in this receipt. Its break/continue choice uses
the event gate and actual prior-State bool; the exit predicate consumes
already completed child work, not a fixed host schedule.

The existing guarded-branch/scoped-loop path accepts this composition without
new production source dispatch, helper-name whitelists or broadened effect
authority. There are two independent break-control registrations. An observed
leading parent gets index recovery independently of the child's leading-index
recovery before its value is passed to the checkpoint. Trailing child steps
need no leading-index copy. Pure discovery remains isolated.

## Values, Effects And Failures

[Unit/oracle controls](../../tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects_parents_tests.rs)
check independent child/parent scope, exact types and counts, protected parent/
header writes, source-terminal suffix rejection and branch-local width scope.
The 100-case signed seed/gate/flag/trip/round grid includes negative/zero parents
and zero/one/three-trip children. Four raw markers and two proof modes drive
actual repeated events.

Manual break witnesses produce `(total,current)=(69,56)` for trailing child steps
and `(47,34)` for leading child steps, at `parent_index=1`.
A three-parent continue witness produces `(215,182)` at `parent_index=3`.
The latter completes selected child work, continues the parent, executes an
unselected middle child and continues again before reaching a later child.

The parent continue does not discard completed child work. A zero divisor in
the skipped parent suffix proves that suppression is real, not a masked
result after speculative evaluation. Poison choices are made after preceding
valid suffix arithmetic: proof modes may change later selection and State,
and are not assumed equivalent.

[Reference/native execution](../../tools/nuisc/tests/native_application_bridge/state_loop_parents.rs)
uses actual preceding callback output. Native observers read real refine,
checkpoint, combine, final observer and answer arguments. Separately checking
these values prevents compensating current/index errors from hiding behind
the final total. Normal/reversed inventories, exact source prints, status,
canaries and raw State words are checked. Positive/negative zero, subnormal
and quiet-NaN marker bits survive; event seed/flag/marker inputs cannot replace
the retained State.

Eight main healthy native processes contribute 32000 healthy accepted callbacks.
Fifteen selected failures per trailing source/order and eleven per leading
source/order contribute 104 fatal processes/312 earlier accepted callbacks.
Selected failures cover checked bounds/body/hole proofs, reached parent suffixes,
advance, finish and answer arguments, rather than traps on unselected paths.

Reference failure retains the entire last published State, enters Faulted and
rejects later events/completion. Flushed native prefixes prove completed
effects and absence of later publication, not post-abort memory inspection.
Completed source effects are not rolled back. The existing ten-second native
child-process deadline remains in force; SIGKILL is not accepted as success.

## Reservations And Delivery

The main fixture requests exactly nine loop-work units and 100 helper entries.
This accommodates the explicit three-parent/two-three-trip-child continue
control; it is not a new compiler-wide grant. Each callback starts a fresh
shared reservation budget. A declared parent bound is reserved before entry,
and a child is reserved only when actually reached.

The separate limits 2/5/6/9 cohort checks:
- At limit 2, parent bounds 3/65537/i64::MAX reject after `print(5)`, before parent
  work. Two-parent selected child bounds 1/3/65537/i64::MAX reject after prints 5,8,6,7.
- At limit 5, completed three-trip child work and a parent break succeed without
  reserving a later child. Bounds 4/65537/i64::MAX reject before the first child.
- At limit 6, two parent continues and an unselected middle child complete.
  The third parent enters, then its selected child rejects after `print(7)`,
  before its first body statement. No skipped parent suffix is evaluated.
- At limit 9, the corresponding three-parent continue sequence succeeds exactly.
  All policies also retain skipped oversized/zero-divisor controls, negative
  child-width body suppression and small positive controls.

This budget cohort contributes 32 healthy processes/800 accepted callbacks and
88 fatal processes/88 prior accepted callbacks. The earlier parent/first-child
budget rejections use valid arithmetic, so another trap cannot mask preflight
failure. Selected suffix poison is intentional in the parent-control witnesses.

[AOT controls](../../tools/nuisc/src/aot_application_effect_call_state_loop_parents_tests.rs)
retain exact State layout `State{result:bool;total:i64;marker:f64}`, runtime
loops and no owned aggregate allocation. Default ungranted mode, any missing
single print grant, or removal of transitive Effect order reject.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_parents.rs)
checks four source variants through fresh/cache/two source-free restorations:
16 images, 1280 healthy accepted callbacks, 112 fatal events/336 earlier accepted
callbacks and 16 oversized-script rejections before open. Eight healthy scenarios
run before and after fatal cohorts: four finite marker/trip/proof profiles,
two skipped oversized-bound profiles and actual multi-event parent break/
continue controls. These finite CLI witnesses are not a full cross-product or
raw-NaN input proof. Buffered fatal CLI stdout is not prefix evidence.

Executable, host objects, envelope, lifecycle and exact budget-bearing mode
match across cache and both restorations. Source, manifest, project cache and
original output are removed before restoration. Artifact, container and
executable bytes remain unchanged by restoration/execution.

## Acceptance And Next Boundary

Fresh serial macOS CPU acceptance passes 211 distinct selected tests:
177 compiler, three native/reference/budget, one CLI, three host and 27 tensor.
Nine are new: four unit/oracle, one AOT, three native/reference/budget and one CLI.
Older native/CLI/full-workspace cohorts remain historical unless explicitly
rerun; focused reruns and child processes do not inflate test counts.

Run serially from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_loop_parents:: --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- iteration_effects control_loops scalar_helpers scalar_control state_loop --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- effect_calls::state_loop_parents:: --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

All 3345 drift checks/23098 patterns, changed Rust formatting, UTF-8, changed
source/test/document caps, relative links and diff checks pass.
Four untouched historical whole-workspace formatting differences remain.
Full-workspace/strict Clippy, fresh Linux/GPU/Windows, formal/global memory
safety and measured performance are not claimed.

The coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
All six complete preceding values remain unabridged and hash-checked.
Command history is a collision-checked quoted no-op heredoc; shell syntax
and harmless current-front replay prove that old commands stay inert data.

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `eb19666c4efdcb8926a1a25b4a5dcaae51fa32bd69354cdf543c0bd20de78921` |
| `next_step` | `fe104beeb84a945429a52ed9455d58f3469dcfc44f867068876b8240c9a5168f` |
| `blocker` | `549ac57073b5f767e81081286df4a695c5fe183ca8b9f6c2e430a90161da8d65` |
| `next_action` | `e9be8ad1586bd95ae364eca059e68b5997c86b464c04a75b6a931d539eb20e57` |
| `validation_command` | `a9b52c575813953bbf437286a8633c6781a227082ead7b00c51dbfbba515da1f` |
| `expected_artifact` | `ef346742ddf6dad55b0262956bb3862030ad855ac994f9548cd5466fc445166c` |

The rebuilt ordinary CLI matches all six source fields and complete predecessor
hashes, with clean drift/coverage/manifest/milestone/hierarchy/task-card lineage
and the same ready selected task card. This bounded receipt does not complete
the coordinate.

This checkpoint's next boundary was the explicit effectful return-in-loop
exclusion, including transitive callers. The subsequent
[effectful loop-return proof](nuis-native-stateful-loop-returns-v1.md)
now advances it with explicit profile validation, independent payload/signal
transport and real native/source-free evidence, not merely removal of a guard.

Trailing-parent composition, generalized iteration I/O/resources, asynchronous
effects, broader entering-State writes, stable ABI, general GPU closures and
completed compiler self-hosting remain separate.
