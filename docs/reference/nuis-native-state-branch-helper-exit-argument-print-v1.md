# Native State Branch Helper Exit Argument Print V1

Audit date: 2026-11-11. Checkout baseline: beta-0.18.7, `a270a99e`.
This uncommitted follow-up extends the
[selected-return witness](nuis-native-state-branch-control-exit-argument-print-v1.md).
It creates no version bump, commit, tag or release.

## Selected Effectful Helpers

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/state_branch_helper_exit_argument_print.ns)
keeps the current-control branch and its independent guard, then calls either
`negative_update` or `nonnegative_update`. Each leaf prints an entry marker and
its full i64 input, checks a separate fault input, prints a completion marker,
and returns the wrapping decrement or increment. Only the selected leaf runs.
Unselected caller and leaf guards may be zero without trapping or printing.

Two helper guards append to the preceding 39 input words. Open/event/close
widths are 41/43/2, with two output State words. The earlier fixed-step,
checked-control and selected-return profiles remain independently registered.
The source is handled by the existing effectful lowering path; no production
admission or runtime budget is widened.

The order is float-update check, selected branch check, leaf entry prints,
leaf check, leaf completion print, record completion, exit processing and final
callback publication. An earlier failure skips everything after it. A later
failure does not undo already-observed print effects: complete accepted State
retention is not a transaction that rolls back external effects.

## Separate Admission And Effect Grants

[Source checks](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_branch_helper_exit_argument_print_tests.rs)
keep the leaves, caller and work function out of the pure catalog while admitting
them in the iteration-effect catalog. Existing field readiness and continue-step
restrictions remain. Wrong leaf arguments are checked at the frontend; malformed
return bodies must not enter the outliner's exact-type catalog.

The [native checks](../../tools/nuisc/tests/native_application_bridge/state_branch_helper_exit_argument_print.rs)
remove each of the three print-site grants from each leaf in turn, keeping all
other grants. Every such removal rejects emission under both declaration orders
and both float widths. A caller or sibling grant cannot authorize that missing
site. These are static admission requirements for potentially selected sites,
distinct from runtime suppression of an unselected branch.

A separate source contrast flips bit 32 in each leaf input and decodes it inside
the leaf before arithmetic. Actual input observations and source prints change,
but returned State stays the same. MIN/MAX, wide and marker-valued controls and
reached leaf failures check that the returned value, not the encoded input, is
used without truncation.

## Model And Failure Boundaries

The [model](../../tools/nuisc/tests/native_application_bridge/state_branch_helper_exit_argument_print_cases.rs)
reuses the previous branch selection/guard boundary through `execute_selected`.
The predecessor still supplies its original arithmetic transition; the new model
supplies leaf entry, prints, fault and completion before returning the update.
The predecessor regression suite is rerun after that extraction.

There are sixteen raw sequences and eight finite delivery sequences per width,
including wrapped branch changes, signed zero, NaN transport, empty callbacks and
event-input decoys. Thirteen fault rows retain the six predecessor failures and
add four reached leaf failures at first/later parents plus three caller/leaf
collisions. Accepted prefixes end at i64 MAX so subsequent events exercise both
branches using real persistent State, not the supplied decoy control.

Native probes compare actual callee inputs and exact print prefixes. Reference
sessions reject a wrong-type helper guard without leaving Open, and retain both
accepted State words through failed events and faulted close. Native traps still
terminate the process; no pending-State recovery is claimed.

The predecessor's base count two, 12-work ceiling and 192-entry ceiling stay
unchanged. Explicit 64-entry and one-work rejection controls remain. These are
bounded workloads, not an optimization claim or unlimited helper depth.

## Artifact Delivery

The [CLI driver](../../tools/nuis/tests/native_session_workflow/state_branch_helper_exit_argument_print.rs)
runs eight healthy sequences before and after thirteen failures in fresh,
cached and two source-free restored forms, in both widths: 232 process runs.
Source, manifest and project cache are removed before restoration. Complete
State, source stdout, completion and artifact identities are checked. Direct
native probes own exact trapping prefixes; buffered CLI output may be shorter.

## Validation

Fresh selected validation on 2026-11-11:

| Selected Suite | Result |
| --- | --- |
| Helper/branch source admission and refusals | 4 passed |
| Helper/branch native, reference, return, grant and budget controls | 10 passed |
| Helper artifact delivery | 1 passed |
| Tensor and lineage regressions | 36 passed |
| Script unit tests | 40 passed |
| Repository file budgets | 2 passed |
| Host absolute-path policy | 1 passed |

These are 94 distinct selected tests, without counting matrix rows or repeated
runs again. The ten-test native/reference run completed in 38.00 seconds; the
232-run artifact test completed in 657.87 seconds. The high-bit return contrast
and every leaf grant refusal passed. Selected host acceptance is not a full
workspace or CI run.

The historical mainline regression previously required an old task description
to remain the current field prefix. It now checks retention of that history
and the actual selected coordinate, without forcing stale prose onto a newer
task. All full predecessor SHA-256 regressions remain enabled.

Encoding, documentation links, formatting and patch whitespace checks pass.
The encoding and link checks cover 6117 UTF-8 files and 4885 local links.
Architecture validation remains 29 packages, six domains and 74 normal/build
edges; source and test file limits are unchanged.

The rebuilt ordinary CLI reports 4381 passing drift checks and no failures.
All 4372 preceding IDs remain in order, with nine new checks. Six source/runtime
fields match and preserve complete predecessor values and hashes. Coverage,
hierarchy and task lineage are clean; the selected coordinate remains active/99.
Historical commands pass syntax validation with only their inert suffix executed.

## Scope And Next Gate

The selected coordinate stays
`standard-library/ns-nova/persistent-application-session`, active/99.
Next compose ordered helper calls within a selected branch, consuming the first
returned value in the next call and preserving independent effects, failure
priority and complete callback publication under unchanged quotas.

This is bounded host CPU evidence, not Linux/GPU parity, arbitrary resource or
I/O/async capture, a stable ABI, a global memory-safety proof or completed self-hosting.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- state_branch_helper_exit state_branch_control_exit --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_branch_helper_exit state_branch_control_exit --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- state_branch_helper_exit --test-threads=1 --nocapture
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

The six complete branch-control worktree fields and their SHA-256 hashes are
retained, including source-free verification evidence. Historical validation
commands remain quoted no-op here-document data. Syntax-check the full command
through stdin and execute only the inert history suffix, never the old commands.

| Field | Complete Branch-Control Predecessor SHA-256 |
| --- | --- |
| `evidence` | `8f1274da6499f40f92dd2f95e245c35ce3451bc874b0727d4b11c347aa9ab5f3` |
| `next_step` | `b49e466d815714bf50f8679bf88322f3dad879dbb5fee1d81ee1b0a0f01de330` |
| `blocker` | `a616abd648c3dd15061c3aa2eca0e31ac8cce2176abaac7aa25ee265dba3df86` |
| `next_action` | `619b201e391a54b1dfbbf66372f538e557c7d31b07889d01dddb74936063d838` |
| `validation_command` | `f4ce33c928ecb4e8a1f4cd6448d53b828c7d66f535b2950cee75c474a2b55c2f` |
| `expected_artifact` | `45ec6238e69386ce4192323b54beb02364bf1fcf9957a644f870624fddfcbe01` |
