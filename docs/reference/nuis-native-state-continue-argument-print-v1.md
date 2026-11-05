# Native State Continue Argument Print V1

Audit date: 2026-11-04. Checkout baseline and latest committed patch:
beta-0.18.5, `bbb00651`. This extends the
[early-break receipt](nuis-native-state-break-argument-print-v1.md).
No release, version bump or commit is created here.

## State-Bounded Continue

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/state_continue_argument_print.ns)
adds a selected continue path after each child's observed State update.
That path prints marker 78, performs the identical unit step and continues.
The ordinary suffix prints marker 79, evaluates a checked helper and prints
its zero result before the ordinary step. Each parent observes the final child
index. A continue skips that suffix, not the next iteration or its eager checks.

All cuts retain full child traversal: a positive callback executes 0/3/0/4
children and accepts negative State, while its successor uses that accepted
State for 0/4/0/4 despite a positive initial-value decoy. First, middle, last,
absent and out-of-range cuts change suffix call counts, not completed updates.
The zero-parent and zero-child controls leave their unused poisoned guards inert.

## Existing Lowering, Not An Exemption

This checkpoint composes existing source normalization, generated-prologue
provenance and fresh epoch ownership; it adds neither syntax nor a production
admission exception. The
[source controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/parent_reentry_argument_print_tests.rs)
cover both float widths, reversed declarations, literal/dynamic parent bounds
and mirrored continue branches. Missing, duplicate, mismatched and moved steps
remain rejected, including an intervening effect between the required step and
continue. Generated-name lookalikes, branch-moved initializer pairs, mutable
child bounds, unready ancestors and borrowed-value aliases remain rejected.

## Independent Models And Faults

Each f32/f64 kind has 27 raw sequences, eight finite sequences and 20 fault rows.
The [profile](../../tools/nuisc/tests/native_application_bridge/state_continue_argument_print_cases.rs)
reuses the independent IEEE sign/magnitude schedule, toggling sign only for
completed updates. It retains NaN, zero, infinity and subnormal distinctions.
The callback adds a suffix guard after the existing cut, making fourteen input
words without changing earlier twelve/thirteen-word profiles.

The [shared effect model](../../tools/nuisc/tests/native_application_bridge/parent_bound_argument_print_cases.rs)
advances its index once on a continue, records skipped suffixes independently
and checks a reached suffix guard only after the State update. Helper-call
expectations add only non-continued suffixes. The
[oracle controls](../../tools/nuisc/tests/native_application_bridge/state_continue_argument_print.rs)
pin complete work, final indices, State, suffix counts and faults. Removing a
selected continue makes its deliberately poisoned suffix fail. A poisoned next
iteration still fails after a successful continue; it is not mistaken for break.

Failures include reached producers, eager initializers, lazy arguments, State
updates, suffixes and final publication. Reference sessions retain the last
accepted callback State through rejection and faulted close. Direct native
probes require exact flushed prefixes and no successful result publication;
they do not inspect failed output-buffer memory or promise rollback of output.

## Reservations And Delivery

A positive callback succeeds with eleven work units and rejects at ten before
the final parent's body; the negative schedule retains twelve/eleven. Checked
producer failure at that boundary precedes reservation failure. Eight/64 helper
entry rejection controls remain below the unchanged healthy limit of 192.
Neither continue nor skipped suffixes refund reserved loop work.

The [CLI matrix](../../tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs)
checks fresh builds, confirmed cache hits and two restores after deleting source,
manifest and project cache. Three-callback finite sequences and healthy runs
surrounding faults compare State, output and binary/container/host-object/
envelope/lifecycle/packaging identity. CLI trap output may be a buffered prefix;
the direct native harness checks exact flushed output.

## Registry Maintenance

The native iteration check registry was near its 800-line limit. Its recent
argument-print module declarations and exported check slices now live in a
[focused registration module](../../tools/nuis/src/dev_tensor_drift_data_native_argument_print_registration.rs).
The parent retains the original ordered list, with the continue group appended
after break. The existing registration regression now checks these groups'
contiguity, order and uniqueness as well. No historical guard IDs are removed.

## Validation

All 167 distinct selected tests/checks passed:

- 50 compiler library controls.
- 45 native/reference/oracle controls (260.54 seconds).
- Two CLI delivery matrices (643.03 seconds), covering state break and continue.
- 40 Python controls.
- Two repository file-budget checks and one host-path policy check.
- 27 tensor unit tests, including registration order and uniqueness.

The continue profile adds seven tests: two source controls, four native/model
controls and one CLI delivery test. Matrix rows, repeated runs and historical
receipt results are not added to the 167 count. Both delivery matrices complete
fresh/cache/two-source-free-restoration paths for f32 and f64.

Static checks pass for 6,038 UTF-8 files, 4,758 local documentation links and
workspace architecture (29 packages, six domains, 74 normal/build edges).
Locked offline workspace metadata, formatting and whitespace checks pass, as
does the ordinary CLI build. The runtime tensor has 4,243 passing guards,
preserving all 4,233 predecessor IDs and their order. Its six source/runtime
fields and full predecessor hashes agree; coverage, hierarchy and task lineage
remain clean. Historical validation commands are syntax-valid inert data.

## Scope And Next Gate

The coordinate stays `standard-library/ns-nova/persistent-application-session`,
active/99. These are bounded host CPU witnesses, not arbitrary record/resource/
I/O/async capture, stable ABI, Linux/GPU parity or a complete native engine.

Next combine selected break and continue paths in one state-bounded child loop.
Prove their distinct step rules, shared suffix skipping, completed State,
observed indices and reservation accounting without lending ownership to branches.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- state_continue_argument_print state_break_argument_print state_bound_argument_print fallible_bound_argument_print parent_bound_argument_print child_progress_argument_print parent_reentry_argument_print epoch_local_ child_print_epoch local_scalar_argument_print loop_bool_projections float_compare_argument_print_epoch iteration_effects_exits --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- parent_reentry_argument_print:: epoch_local_argument_print:: float_compare_argument_print_epoch:: state_loop_exits:: sibling_bounds_native_ready_updates_fit_160_but_entry_limit_100_still_rejects --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- parent_reentry_argument_print::state_continue parent_reentry_argument_print::state_break --test-threads=1 --nocapture
python3 -m unittest discover -s scripts/tests
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test file_line_limit --locked -j1 -- --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-host-absolute-paths.sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1
python3 scripts/check-text-encoding.py
bash scripts/check-doc-links.sh
python3 scripts/check-workspace-architecture.py
cargo metadata --locked --offline --no-deps --format-version 1
cargo fmt --all -- --check
git diff --check
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --locked -j1
target/debug/nuis dev-tensor --json
```

## Predecessor Lineage

All six complete state-break fields remain with the hashes below. Historical
commands are quoted no-op here-document data, checked through stdin for syntax
and inertness without replaying their contents.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `4cbaa9bd31bc4f38f1ce67194d631b4d0c814166162a0731ae550455fbef8cf4` |
| `next_step` | `f21df1bf7c395d48b6b0de58150b8beac5eec799e83192351656d46cc157f74a` |
| `blocker` | `2d7534356a32030a55b4c87f7636edb72df81def9947d0c96bb0e337df379fe3` |
| `next_action` | `8c9d0c3430b3c95666e00a51a058d1bdd57adce91810059ce32fbc6c68922dea` |
| `validation_command` | `b943fd26e011f96b78cd103a9f7b57190039861f57592c79e405bb4976bb2f2f` |
| `expected_artifact` | `00842dbb3798dbc4c994cf9732e3ab5dc93e5797e16e8c08e4c0ffc726705729` |
