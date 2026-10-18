# Native Stateful Selected-Child Invocation V1

Historical return-in-loop exclusions below describe this checkpoint. The later
[effectful loop-return receipt](nuis-native-stateful-loop-returns-v1.md)
advances that boundary through explicit literal-print typed transport and
native/source-free proof, not general I/O or pure-function reclassification.

This bounded CPU receipt advances the
[child-local loop-exit checkpoint](nuis-native-stateful-loop-children-v1.md).
Its Git baseline is beta-0.17.2, commit 00ab72d3. It does not bump the version
or declare a stable ABI. Recovery acceptance below was replayed on 2026-10-18.

## Source And Contract

[The Nuis source](../../tools/nuisc/tests/native_application_bridge/state_loop_selected.ns)
keeps a continuing parent loop. Each parent first updates its current value,
then derives child selection from that value and the actual prior-State bool.
Only the selected arm prints 6, evaluates `width = trips / root`, prints 7
and enters the child loop. The inactive arm prints 9 and updates the current
value without evaluating the child bound or body proofs.

The selected body retains child-local break/continue, leading/trailing induction,
cut-skipped checked work and a fallible body proof `1 / (root + 1)`.
After either arm, the parent's actual `combine` arguments consume the current
value, recovered child index, parent index and suffix proof. Both parent rounds
continue unless selected work fails. Selection is recomputed each round;
it is not a fixed schedule or a mask applied after speculative work.

Four [source variants and the source-control oracle](../../tools/nuisc/tests/native_application_bridge/state_loop_selected_cases.rs)
place the selected child in either arm and use leading/trailing child steps.
The oracle models source loops, local exits and full-bound reservation directly,
not generated flags or projected field transport.

The existing guarded-branch/scoped-loop pipeline accepts this composition.
No production source-name dispatch, helper-name whitelist or effect admission
exception is added. Unit/AOT tests keep pure discovery, parent/header authority,
lexical width scope, exact literal-effect grants, Effect ordering and the
three-field State unchanged. Computed print and parent/header writes reject.

## Execution And Reservations

[The unit/oracle tests](../../tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects_selected_tests.rs)
check 100 signed seed/gate/flag/trip/round cases, actual state-derived selection
and independent failure counts. Explicit skipped witnesses yield `total=339`,
`current=316` and child indices 0/0. Selected `cut=2` witnesses yield
`(total,current)=(770,747)` for trailing steps and `(510,487)` for leading steps.
Actual callee arguments are checked separately; compensating current/index errors cannot
hide behind the total.

Valid arithmetic before a selected failure may itself change later selection.
The trailing `hole=2` witness finishes its first parent suffix, selects the later
child and traps after `print(19)`. Poisoned and all-valid arithmetic are therefore
not assumed to produce identical control or State. An initial test-only count
of thirteen failures missed this case; the corrected fourteen-case assertion
also checks the later selected prefix explicitly.

[Reference/native sessions](../../tools/nuisc/tests/native_application_bridge/state_loop_selected.rs)
pass the real previous callback output into the next callback. Native observers
record refine, combine, final current/parent index and answer arguments. Raw
State words retain positive/negative zero, subnormal and quiet-NaN marker bits;
status, output canaries, exact source prints and normal/reversed inventory
order are also checked.

Eight main native healthy processes cover 32000 healthy accepted callbacks.
Four sources, two inventory orders, four raw markers and two proof modes run
800 open/three-event/close sequences per source/order. Fourteen selected
arithmetic failures per source/order produce 112 fatal processes with 336
earlier accepted callbacks. These include checked bound failure before `print(7)`,
selected body failure after `print(7)`/before `print(10)`, cut-skipped and later selected
hole proofs, parent suffix, advance, finish and answer proofs.

Reference faults retain the last published State, enter Faulted and reject
later events/completion. Flushed native prefixes prove completed effects and
absence of later publication, not post-abort memory inspection. Completed
source effects are not rolled back. The ten-second native process deadline
remains in force.

The main policy grants eight loop-work units and 100 helper entries.
The separate limits 2/5 cohort proves:
- An unselected child consumes no reservation: two continuing parent rounds
  accept even with `i64::MAX` child trips, including a zero bound divisor.
- A selected negative width skips its invalid body proof; zero-width and
  one-trip children supply small positive controls.
- At limit 2, selected positive bounds 1/3/65537/i64::MAX reject after prints 5,8,6,7,
  before any child statement.
- At limit 5, the first selected three-trip child and complete parent suffix
  execute; the second selected child rejects after `print(7)`/before `print(10)`.
  First-child continue and early break are separate controls. Break does not
  refund the full three-trip reservation.

This budget cohort adds 16 healthy processes/320 accepted callbacks and
48 fatal processes/48 prior accepted callbacks. Budget failures use valid
arithmetic, so a different trap cannot mask reservation failure. Skipped
oversize controls run repeated callbacks using their actual preceding State.

[AOT checks](../../tools/nuisc/src/aot_application_effect_call_state_loop_selected_tests.rs)
require every exact print grant and transitive Effect order. The default
ungranted mode rejects. Runtime loops and the exact State layout remain,
without owned aggregate allocation.

## Delivery And Acceptance

[CLI delivery](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_selected.rs)
runs four source variants through fresh/cache/two source-free restorations:
16 images, 960 healthy accepted callbacks, 144 fatal events/432 prior accepted
callbacks and 16 oversized-script rejections before open. Healthy sessions run
before and after fatal cohorts. Alongside four finite marker/trip/proof
profiles, every image runs skipped `i64::MAX` bound and zero-divisor controls.
Finite CLI markers are not raw-NaN input acceptance or a complete cross-product.
Buffered fatal CLI stdout is not prefix evidence; the flushed native probe
supplies that evidence.

Executable, host objects, envelope, lifecycle and exact budget-bearing mode
match across cache and restores. Source, manifest, project cache and original
output are removed before restoring. Artifact, container and executable remain
byte-identical after restoration/execution.

The recovery cohort passes 206 distinct selected macOS CPU tests:
172 compiler, three native/reference/budget, one CLI, three host and 27 tensor.
Nine are new: four unit/oracle, one AOT, three native/reference/budget and one CLI.
The interrupted wider native cohort has no confirmed final result and is not
counted. Earlier 214-test, 207-test and 668-test receipts remain historical,
not newly counted full replays. Focused reruns and child processes do not
inflate test counts.

Run serially from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_loop_selected:: --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- iteration_effects control_loops scalar_helpers scalar_control state_loop --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- effect_calls::state_loop_selected:: --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

All 3334 drift checks /23032 patterns, changed Rust formatting, UTF-8, changed
source/test/document caps, relative links and diff checks pass. Historical
whole-workspace formatting differences remain outside this change.
Full-workspace/strict Clippy, fresh Linux/GPU/Windows, measured performance
and formal/global memory safety are not claimed.

The coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
All six complete preceding values are preserved with hashes; command history
is a collision-checked quoted no-op heredoc. Shell syntax and harmless
current-front replay prove that old commands remain inert data.

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `cb727a8f3fdc68d7b72f5f8bef9137d46d948f584f08f772306314ef8f5ab3bb` |
| `next_step` | `cf23db8e866f149fbbddd6a28eb2b47f1e104934a27201264fae2d9b8a4a9e58` |
| `blocker` | `33e2ad3345c9458671714e0ab34b59c22be4a6c469343c99233ad996410c7dcf` |
| `next_action` | `95b100e4e41335e7fdd13d1eaea365b418c7200d4c2b0cb45a22d526f292436d` |
| `validation_command` | `cfe5c96127058383cc8b3c0cbb8a5834763de15c5ee46099b54da08eed41f3ec` |
| `expected_artifact` | `77485f4e685a76d006843338ff2c2cc493ae513b02af754eac18daa644ed7609` |

The rebuilt ordinary CLI matches all six source fields and complete predecessor
hashes, with clean drift/coverage/manifest/milestone/hierarchy/task-card lineage
and the same ready selected task card. Bounded acceptance does not complete
the coordinate.

## Next Boundary

The subsequent [parent-local loop-exit receipt](nuis-native-stateful-loop-parents-v1.md)
now advances the historical parent-exit boundary below for a leading parent,
using real child checkpoint inputs, recovered parent indices and source-free
sessions. Its next functional admission boundary is effectful source return
from a loop; trailing-parent composition remains separately unproved.

Exercise parent-local break/continue after completed selected child work.
Retain real child-produced current/index values, suppress only the selected
parent suffix and later reservations, and verify failure-before-publication
plus fresh/cache/source-free identity through actual sessions.

General I/O/resources, asynchronous/return-in-loop effect closures, broader
entering-State writes, stable ABI, GPU closure generalization and completed
self-hosting remain separate.
