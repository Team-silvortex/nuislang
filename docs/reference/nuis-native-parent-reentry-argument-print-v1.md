# Native Parent Reentry Argument Print V1

Audit date: 2026-11-04. Checkout baseline: `dc87ec43`; latest version-label
commit: beta-0.18.4, `ceaa2caa`. This worktree follows the accepted
[child-owned route](nuis-native-child-epoch-local-argument-print-v1.md).
No release, version bump or commit is created by this receipt.

## Repeated Parent Entry

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print.ns)
adds a runtime parent-count parameter to the child-owned fixture. Each parent
performs its identity update, constructs a fresh child record from the current
total and original left field, then runs the dynamic child loop. Child updates
use the right field; the next parent must reseed left rather than reuse right.
Markers 76/77 distinguish parent entry/completion from child work and final tail.

Initializer, lazy-use and update guards subtract the current parent index.
Their zero checks therefore fail in later parent entries, after earlier child
work succeeded. A false lazy gate cannot suppress the eager initializer, and
a zero initializer result cannot suppress a reached lazy helper. Tail failure
occurs after all parents but before callback State publication.

This adds execution coverage for the existing owner/normalization route,
not a new production lowering permission, opcode, ABI or emitter.
[Source controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/parent_reentry_argument_print_tests.rs)
retain unready outer-record, escaping-alias and branch-owned-pair refusals,
for both float widths and declaration orders. No parent certificate is lent
to ordinary branch recursion.

## Independent Model And Execution

The [model](../../tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print_cases.rs)
composes the earlier independent bit-by-bit child word oracle, starting a fresh
copy on every parent entry. It strips only the child model's synthetic callback
and final-tail markers, and retains real child/helper output. The new parent
markers and single actual tail are modeled separately. Numeric pins distinguish
left/right/left field order independently of emitted code.

There are 22 raw sequences, six finite sequences and six fault rows per f32/f64
kind. Zero parent trips and zero child trips poison otherwise-faulting guards
to prove that skipped work is not evaluated. Subsequent callbacks use accepted
State rather than their decoy initial argument. Later faults cover initializer,
lazy helper, update and final tail, with an additional zero-word lazy fault.

The [native/reference probes](../../tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print.rs)
check raw State, output order, canaries, reversed YIR, faulted-close behavior
and rejection of events after a fault. Reference helper counts are
`parents * trips * (1 + selected)` for checked64 and
`parents * (trips + 1) + 1` for finish. Failure retains the last accepted
callback State even though earlier output in the failing callback is observable.

The fixture uses 12 shared loop-work units: three parents each with up to three
children. The compiler's global limits are unchanged. The first native run
exhausted the inherited 64-entry fixture budget: entries count all reached YIR
functions, including outlined branches, not just source-level helper calls.
Success uses an explicit 192-entry fixture bound. Separate limit-8 and limit-64
probes retain the protective rejection before successful callback publication;
the longer witness is not shortened and the producer's accounting is unchanged.

The [CLI matrix](../../tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs)
runs fresh compilation, a cache hit and two independent source-free restores.
Source, manifest and project cache are deleted before restore. Binary/container,
host-object, envelope, lifecycle and packaging identities are checked. Healthy
runs surround faults on each image. Native probes flush exact fault prefixes;
CLI fault stdout is checked only to the extent the trapping child flushed it.

## Validation

The final selected suite passed 109 distinct tests/checks:

| Selected Set | Passed |
| --- | ---: |
| Parent/root/child epoch-local and ready-local compiler regressions | 25 |
| Parent/root/child native and reference controls | 13 |
| Parent f32/f64 fresh/cache/two source-free delivery matrix | 1 |
| Python CI-script regressions | 40 |
| Repository file budgets | 2 |
| Host-absolute-path policy | 1 |
| Development tensor tests | 27 |

Seven controls are new: two source tests, four native/reference/oracle controls
and one CLI matrix. Matrix rows, the initial budget failure, repeated runs and
historical receipts are not added to this total. No production lowering code
changed in this successor, so the focused neighboring regressions were rerun
rather than claiming a new run of the preceding 220-test broad library suite.

The CLI matrix completed in 163.85 seconds. Structural checks passed for 6,011
UTF-8 text files, 4,698 local Markdown links, locked offline metadata, formatting
and whitespace. Workspace architecture remains 29 packages, six domains and
74 normal/build edges. The proof index stays at 300 lines; the drift registry
is 784 lines, below its 800-line source budget.

The rebuilt ordinary CLI passes all 4,182 drift checks, retaining every one of
the previous 4,174 IDs and adding eight parent-reentry checks. All six source
fields match the runtime report and retain their complete predecessor values
and SHA-256 identities. Coverage, hierarchy and task lineage are clean; the
selected coordinate remains active/99. Historical shell data passes syntax
and inertness checks through stdin without replaying earlier validation work.

## Scope And Next Gate

This is bounded native host CPU evidence on the current machine, not Linux/GPU
provider parity, arbitrary resource/I/O/async capture, complete native-engine
closure, stable ABI, whole-program memory safety or compiler self-hosting.

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. The next gate is a fault in a later child of a later parent, after
both earlier parents and earlier siblings completed. This witness intentionally
uses parent-dependent guards, so its failing parents fault at their first child.
The [later-child receipt](nuis-native-child-progress-argument-print-v1.md) records
that subsequent boundary separately. This receipt retains its original scope
and counts. Wider prefixes, repeated local consumers and ancestor-grant
inheritance retain their own independent gates.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- parent_reentry_argument_print epoch_local_ child_print_epoch local_scalar_argument_print --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- parent_reentry_argument_print:: epoch_local_argument_print:: --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- parent_reentry_argument_print --test-threads=1 --nocapture
```

## Predecessor Lineage

The six complete child-owned fields remain lossless, with the following hashes.
The preceding validation command is quoted no-op here-document data, checked
through stdin for syntax and inertness without executing historical commands.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `5bb3893c140de6b6e78a185ca4ce815738ebdf94232b9a26023e9f4e59d61878` |
| `next_step` | `da9ddeccf42c6e1d9ef5527f7537c4bde3053f4bb227379f154f1c23f2c2694c` |
| `blocker` | `3a86266b40428103356e7da986f58da8c47da931b1b2b6f6a69794fab5d21db5` |
| `next_action` | `35212b4f4fc6d63333baf64827a7e90d89a82f59c665bd546cbcbb3ff3f13d48` |
| `validation_command` | `b44e3c4481097bd7c237cbbecf407b211082781b9e166d68df95fc149cc90004` |
| `expected_artifact` | `883715279554cc21a204d677e46903286ba07a4f2317ad02d49e1a14866c9303` |
