# Native State Branch Control Exit Argument Print V1

Audit date: 2026-11-11. Checkout baseline: beta-0.18.7, `a270a99e`.
This is an uncommitted follow-up to the independently
[checked-control witness](nuis-native-state-checked-control-exit-argument-print-v1.md),
not a version bump or release.

## Selected Return Branches

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/state_branch_control_exit_argument_print.ns)
selects a checked return using the current persistent i64 control. Negative
controls check the negative guard and return wrapping `value - 3`; nonnegative
controls check the other guard and return wrapping `value + 2`. The unselected
guard may be zero without causing a trap. MIN/MAX wrapping changes which branch
the next child or callback selects.

The callback still publishes `State { total: f32/f64, control: i64 }` together.
One additional i64 guard follows the preceding 38 input words: open/event/close
widths are 39/41/2 with two State output words. Existing fixed-step fixtures,
transport limits and effect authority remain unchanged. No production lowering
rule was widened: the current selected-return lowering admits this composition.

The float check precedes control production. A selected branch failure prevents
the completed-record marker, exit arguments and callback publication. Successful
control production does not bypass later exit or tail failures. Native traps
terminate the process; accepted-State retention after a fault is checked in the
reference session, not claimed as a recoverable native pending-State snapshot.

## Models And Controls

The [model](../../tools/nuisc/tests/native_application_bridge/state_branch_control_exit_argument_print_cases.rs)
reuses the existing parent/child and grouped-exit model with an explicit control
transition at the float-check/record-completion boundary. The old model supplies
its original wrapping increment. The new model supplies selected guards and
branch arithmetic, so it does not reuse the fixed-increment schedule or assume
that exit decisions stay unchanged.

Sixteen raw sequences per width cover signed zero, NaN transport, full-width
controls, marker-shaped values, MIN/MAX transitions, zero-work callbacks and
event input decoys. Eight finite sequences form the delivery subset. Six failure
rows after an accepted two-callback prefix cover both selected branches, first
and later parents, a colliding float check and a final control-tail failure.
The accepted prefix ends at i64 MAX; later events must not use their decoy control.

[Source tests](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_branch_control_exit_argument_print_tests.rs)
retain readiness, exact i64 argument types, branch-local scope and continue-step
requirements. [Native/reference tests](../../tools/nuisc/tests/native_application_bridge/state_branch_control_exit_argument_print.rs)
observe actual producer inputs under both declaration orders, compare complete
State and exact print/fault prefixes, reject a wrong-type second guard without
faulting the reference session and preserve accepted State on failed events.

Helper entry accounting remains real. The initial three-child fixture exceeded
the unchanged 192-entry ceiling on a longer event. This profile uses base count
two instead, retaining later-parent and wrapping-transition coverage without
changing the predecessor fixtures or runtime accounting. The work ceiling stays
12. Explicit 64-entry and one-work rejection controls remain; the latter rejects
the parent reservation before any parent body observation. This is not an
optimization claim or evidence that the longer profile fits the same budget.

## Artifact Delivery

The [CLI driver](../../tools/nuis/tests/native_session_workflow/state_branch_control_exit_argument_print.rs)
runs eight healthy sequences before and after six failures for each width, on
fresh builds, cache hits and two independently restored artifacts: 176 process
runs in total. Source, manifest and project cache are deleted before restoration.
It compares both State words, stdout, completion and binary/container/host-object/
envelope/lifecycle/packaging identities. Buffered CLI failure output is only a
prefix check; exact trapping output belongs to the direct native probes.

## Validation

Source tests: six passed, including the four checked/fixed-control predecessor
tests. Native/reference/model/budget tests: four passed, plus fourteen predecessor
tests passed (121.07 seconds for that combined run). The delivery test passed in
385.84 seconds with all 176 process runs.

The remaining selected checks pass: 35 tensor tests, 40 script tests, two file
budget tests and one host-path-policy test. Together these are 103 distinct
selected tests; repeated runs and matrix rows are not additional test functions.
Formatting, patch whitespace, 6110 UTF-8 files and 4876 documentation links pass.
Architecture validation remains 29 packages, six domains and 74 normal/build
edges. This is not a full workspace or CI run.

The rebuilt ordinary CLI reports 4372 passing drift checks with no failures.
All 4364 preceding IDs remain in order, with eight new checks. The six source and
runtime fields agree, retain their complete predecessor values and hashes, and
coverage, hierarchy and task lineage remain clean. The fixed-step model's
existing guard now checks its explicit increment closure and transition call;
its ID and regression coverage are retained. Historical commands pass syntax
validation, with only the quoted inert suffix executed.

## Scope And Next Gate

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Next put independently checked helper calls inside selected control-return
branches, with explicit per-site effects and failure-order controls. Retain
unselected-branch suppression, complete two-field publication and source-free
delivery without widening quotas.

This is bounded host CPU evidence, not Linux/GPU parity, arbitrary resource or
I/O/async capture, stable ABI, general memory-safety proof or completed self-hosting.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- state_branch_control_exit state_checked_control_exit_argument_print state_control_exit_argument_print --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_branch_control_exit state_checked_control_exit state_control_exit --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- state_branch_control_exit --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test file_line_limit --locked -j1 -- --test-threads=1
python3 -m unittest discover -s scripts/tests
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-host-absolute-paths.sh
python3 scripts/check-text-encoding.py
bash scripts/check-doc-links.sh
cargo fmt --all -- --check
git diff --check
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --locked -j1
target/debug/nuis dev-tensor --json
```

## Predecessor Lineage

All six complete checked-control worktree fields are retained with SHA-256 hashes
and a persistent regression test. This predecessor includes the 2026-11-11
verification note, not just the committed fixed-control fields. Historical
validation commands remain quoted no-op here-document data. Syntax-check the
full command through stdin; execute only the inert history suffix, never replay
the historical command chain.

| Field | Complete Checked-Control Predecessor SHA-256 |
| --- | --- |
| `evidence` | `648a1fa4ede8d76f311b883314aa9b6594b9046abc11f63dd189180f1b7a7698` |
| `next_step` | `c6ab187b593b83ac7899bd09e2c746e1822ec862ef0a48b23aca8d89c6e9816c` |
| `blocker` | `2ba55c8f4d98195b81d03735e9a160f6a99e068759eae1a68234075359bf30ea` |
| `next_action` | `46c0e9f4533b980138849a846f0c600784ca2f864611916168fe6ffd7fea5a94` |
| `validation_command` | `bda5c4a25cf1f86e7d293b63c8705f28bf85deb572cacd98decf59d12cb561dc` |
| `expected_artifact` | `31df8cc484b576e896a4a3cd67800b10f1f158822929c54f9bbf317d845d290e` |
