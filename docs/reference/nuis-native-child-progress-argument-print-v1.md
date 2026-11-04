# Native Child Progress Argument Print V1

Audit date: 2026-11-04. Checkout baseline: `dc87ec43`; latest version-label
commit: beta-0.18.4, `ceaa2caa`. This worktree follows the
[parent-reentry receipt](nuis-native-parent-reentry-argument-print-v1.md).
No release, version bump or commit is created here.

## Later Children In Later Parents

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/child_progress_argument_print.ns)
uses `outer * trips + child` to vary initializer, lazy-use and update guards.
The child induction variable is already ready at its own loop entry; the
original record field retains its existing epoch certificate. No ordinary
readiness, ancestor authority or branch permission is added.

With three children per parent, guard values four and five reach zero in the
second and third child of the second parent. Guard seven reaches the second
child of the third parent. Completed earlier siblings remain observable;
the fault must not publish a partially updated callback State.

There are nine fault rows per kind: each of initializer/lazy-use/update after
four or five completed children, an eager initializer fault with a false lazy
gate after seven, a reached lazy fault on a zero right-field word after seven,
and the tail after all nine children. Healthy controls include zero parent or
child work with poisoned guards and a false lazy gate that skips its late fault.

This successor needs no production lowering change. It exercises the existing
ready-induction plus original-field composition, not a new opcode, ABI or
backend-specific shortcut. Both profiles retain the same eleven input words.

## Independent Model And Shared Drivers

The [model](../../tools/nuisc/tests/native_application_bridge/child_progress_argument_print_cases.rs)
counts completed siblings sequentially instead of reproducing the source's
parent-times-trip-count formula. Each step reuses the independent bit-by-bit
child word oracle with a single iteration; only synthetic callback/tail markers
are removed. A fresh parent resets the field to left, while later siblings use
right. Numeric pins verify left/right/left with descending guard values.

The [profile table](../../tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print_cases.rs)
selects source and case factories. Its clock selector is test-model data, never
a hidden callback argument or runtime ABI field. The earlier parent-only model
is retained. Each profile has 22 raw sequences and six finite sequences per
f32/f64 kind; their fault sets remain separate.

The [shared native/reference driver](../../tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print.rs)
checks raw State, exact flushed fault prefixes, canaries, normal/reversed YIR,
helper counts, rejection of further events after fault and faulted close.
Marker 75 counts completed children; marker 77 counts completed parents. Update
failure ends at marker 74, before that child's completion marker. The zero-word
lazy fault ends at the reached checked helper with guard zero.

The [shared CLI driver](../../tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs)
runs both profiles through fresh/cache images and two independent restores
after deleting source, manifest and project cache. Healthy executions surround
faults, and binary/container, host-object, envelope, lifecycle and packaging
identities are retained. Trapping CLI stdout remains a possibly buffered prefix;
the direct native probe checks the exact flushed sequence.

Both profiles use the same 12 loop-work and 192 helper-entry budgets. Independent
8/64-entry controls retain failure before publication. No global producer limit
is widened. Output effects are not rolled back; callback State publication is
the guarded boundary.

## Validation

The final selected suite passed 116 distinct tests/checks:

| Selected Set | Passed |
| --- | ---: |
| Child-progress, parent/root/child epoch and ready-local compiler regressions | 27 |
| Child-progress, parent/root/child native and reference controls | 17 |
| Parent and child-progress f32/f64 fresh/cache/two source-free matrices | 2 |
| Python CI-script regressions | 40 |
| Repository file budgets | 2 |
| Host-absolute-path policy | 1 |
| Development tensor tests | 27 |

Seven controls are new: two source tests, four native/reference/oracle controls
and one CLI matrix. Both profiles rerun after sharing the drivers. Matrix rows,
repeated runs and earlier receipts are not added to this total. No broader
library suite or Linux/GPU run is claimed.

The two CLI matrices completed in 342.66 seconds. Structural checks passed for
6,015 UTF-8 text files, 4,708 local Markdown links, locked offline metadata,
formatting and whitespace. Architecture remains 29 packages, six domains and
74 normal/build edges. The proof index stays at 300 lines and the drift registry
at 787, within the existing limits.

The rebuilt ordinary CLI passes all 4,190 drift checks, retaining all 4,182
previous IDs and adding eight child-progress checks. All six source fields
match the runtime report and retain their complete predecessor values and
SHA-256 hashes. Coverage, hierarchy and task lineage remain clean; selection
stays active/99. Historical command data passes syntax and inertness checks
through stdin without executing its contents.

## Scope And Next Gate

This is bounded native host CPU evidence on the current machine, not Linux/GPU
parity, arbitrary resource/I/O/async capture, stable ABI, whole-program memory
safety or a complete native engine.

The coordinate stays `standard-library/ns-nova/persistent-application-session`,
active/99. Next is parent-dependent child trip counts: a parent with zero child
work followed by later parents that resume work, with correct fresh entry,
work reservation and failure/publication behavior. Here each callback still
uses one fixed child trip count across all of its parents. The
[subsequent parent-bound receipt](nuis-native-parent-bound-argument-print-v1.md)
records the changing-count matrix separately; historical scope and totals here
remain unchanged.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- child_progress_argument_print parent_reentry_argument_print epoch_local_ child_print_epoch local_scalar_argument_print --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- parent_reentry_argument_print:: epoch_local_argument_print:: --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- parent_reentry_argument_print:: --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete parent-reentry fields remain lossless with these identities.
The previous validation command is quoted no-op here-document data. Syntax and
inertness are checked through stdin, never by replaying historical commands.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `73b79389397a75bd394e57813e688da88b71ef21141040d5221bcdfd45ea42b7` |
| `next_step` | `8c25896e892a79bd755f7d73e3cbaf9a6045e0d235608920a798645fe8b94edf` |
| `blocker` | `1316ac259a751091e7b21ca94da6a5bce8f62a2476e2681add2de4645798df34` |
| `next_action` | `fc873b14e4040c25cd329461c5bad6e2d3e946e2967c551d14ce2f65806367f4` |
| `validation_command` | `49833a4c66e883511e1b21aa10ad7eb699d405778b98080006f851c706ce8684` |
| `expected_artifact` | `01bed8612e29d65f384c247964fc13b0e3fc515add9eb097ff848e77bd10e795` |
