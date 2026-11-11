# Native State Branch Chain Exit Argument Print V1

Audit date: 2026-11-11. Checkout baseline: beta-0.18.7, `a270a99e`.
This uncommitted follow-up extends the
[selected-helper witness](nuis-native-state-branch-helper-exit-argument-print-v1.md).
It creates no version bump, commit, tag or release.

## Ordered Selected Calls

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/state_branch_chain_exit_argument_print.ns)
binds the first helper return to a branch-local `next` and passes it to a second
checked, effectful helper in the same selected branch. The first return carries
a bit-32 tag; the second helper observes that tagged input and decodes it. Final
State therefore matches the predecessor, but the intermediate is deliberately
different from both the original input and the final control value.

The original control selects the branch once. Arithmetic wrapping or a tag that
changes the intermediate's sign must not redirect the second call to the other
branch. Native source contrasts use bit 40 and the sign bit as alternative tags,
preserving final State while changing observed second-helper inputs and prints.

Two additional fault words follow the complete preceding 41-word input prefix.
Open/event/close input widths are 43/45/2, with two State output words. The
production lowering path, base child count two, 12-work ceiling and 192-entry
ceiling remain unchanged; no special rule was added for this fixture.

## Order, Authority And Failure

The chain runs after the float-update and selected-branch checks. Its order is
first entry prints, first check, first completion, second entry prints, second
check, second completion, record completion, exit work and callback publication.
A failed first helper never enters the second helper. A failed second helper
retains the first helper's completed print effects, but does not complete or
publish the pending callback State. Accepted reference State survives failed
events and faulted close; this does not roll back effects or make native process
traps recoverable.

[Source tests](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_branch_chain_exit_argument_print_tests.rs)
check both widths, branch-local scope, exact types, original readiness and
continue-step rules. All four leaves and their callers stay out of the pure
catalog. [Native/reference tests](../../tools/nuisc/tests/native_application_bridge/state_branch_chain_exit_argument_print.rs)
remove each of the twelve leaf print-site grants independently under both
declaration orders and widths. Caller, sibling and earlier-helper grants cannot
authorize a missing site. Static effect admission is distinct from runtime
suppression of unselected calls.

## Models And Matrices

The [model](../../tools/nuisc/tests/native_application_bridge/state_branch_chain_exit_argument_print_cases.rs)
uses an `execute_then` transition immediately after the first successful return,
before the record completes. The predecessor supplies an identity continuation;
the new continuation tags the arithmetic return, observes/checks the second
input and decodes it. The original branch identity travels separately from the
intermediate. The predecessor's six native/reference tests are rerun after this
extraction.

Sixteen raw sequences per width cover accepted-State backedges, event decoys,
wrapped branch changes, marker-valued controls, signed zero, NaN transport and
empty callbacks. Twenty-one native/reference fault rows retain all thirteen
predecessor faults and add four second-helper failures and four first-helper,
caller or float-check collisions. Exact prefixes pin which stage ran before
failure. Low-work and low-entry controls remain separate from source failures.

The [CLI driver](../../tools/nuis/tests/native_session_workflow/state_branch_chain_exit_argument_print.rs)
uses eight finite sequences and eleven fault rows: all eight new failures plus
representative caller, first-helper and tail failures. Healthy sequences run
before and after faults on fresh builds, cache hits and two independently
source-free restored artifacts in both widths, for 216 process runs. Source,
manifest and project cache are removed before restoration. State, stdout,
completion and binary/container/host-object/envelope/lifecycle/packaging identity
checks are retained. Exact trapping prefixes belong to direct native probes;
buffered CLI output may be a shorter prefix.

## Validation

Fresh selected validation on 2026-11-11:

| Selected Suite | Result |
| --- | --- |
| Chain source admission and refusals | 2 passed |
| Chain/helper native, reference, return, grant and budget controls | 12 passed |
| Fresh, cached and source-free CLI delivery | 1 passed; 216 process runs |
| Tensor selection, coverage, drift and complete predecessor lineage | 37 passed |
| Development and repository scripts | 40 passed |
| Repository line budgets and exception ratchet | 2 passed |
| Host absolute-path policy | 1 passed |

These are 95 selected test functions, including the Python tests. Repeated runs,
matrix rows and drift guards are not additional test functions. Native/reference
validation completed in 104.33 seconds, including the two alternate tag controls;
CLI delivery completed in 681.33 seconds, including intended failing runs.
This is selected host validation, not a full workspace or CI run.

The line-budget check initially found 1005 lines in the mainline tests. Moving
the historical checkpoint test unchanged into
[its own module](../../tools/nuis/src/dev_tensor_mainline_checkpoint_tests.rs)
leaves 253 selection-test lines and 753 checkpoint-test lines. No assertion or
test was removed; all 37 tensor tests passed again after the split.

The rebuilt ordinary CLI reports 4390 passing drift checks and zero failures.
All 4381 predecessor IDs retain their order, with nine new checks. All six
complete predecessor fields and hashes match source and runtime; coverage,
hierarchy and task lineage are clean. Historical commands pass syntax checks
and their quoted no-op suffix executes without replaying old jobs.

UTF-8 validation covers 6125 tracked files. Documentation links, formatting,
diff whitespace and architecture checks pass. The dependency graph remains at
29 packages, six domains and 74 normal/build edges; dev-only edges are excluded.

## Scope And Next Gate

The selected coordinate stays
`standard-library/ns-nova/persistent-application-session`, active/99.
Next let the first helper's actual return decide whether the second helper runs,
preserving local scope, independent effect grants, skipped faults, exact ordering
and complete State publication under the same budgets.

This is bounded host CPU evidence, not Linux/GPU parity, arbitrary helper depth,
resource or I/O/async capture, stable ABI, general memory safety or self-hosting.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- state_branch_chain_exit --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_branch_chain_exit state_branch_helper_exit --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- state_branch_chain_exit --test-threads=1 --nocapture
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

All six complete selected-helper worktree fields and their SHA-256 hashes are
retained. Historical validation commands remain quoted no-op here-document
data. Syntax-check the full command through stdin and execute only the inert
history suffix, never the historical command chains.

| Complete predecessor field | SHA-256 |
| --- | --- |
| `evidence` | `dc20d6f6c37d76e6c7c2b77a42b9d7e0139a9cf5f5b0547cc60978c16d16c787` |
| `next_step` | `4fc2c76166fdb30344926d56091d9e11c5cba22549fdcc51d4e0dd9613d9f15f` |
| `blocker` | `5013b4cbf8d938c4f57a06529879b2f0cfa661b14d971ccdf1cadf34748365be` |
| `next_action` | `800c0e1608c197256b944f2409c6047059d1224bb2208e04fcd52dae34675c16` |
| `validation_command` | `61f252aef12291edd4735fccedebfb4d457290b0ddeceaf09cce2569bd2da318` |
| `expected_artifact` | `de66ede314811a7fd48daf26f37b3df3eff6a13c4697ffb3c94715f8205f4e40` |
