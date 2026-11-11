# Native State Return Prefix Exit Argument Print V1

Audit date: 2026-11-11. Checkout baseline: beta-0.18.7, `a270a99e`.
This worktree follow-up extends the
[result-join witness](nuis-native-state-return-join-exit-argument-print-v1.md).
It creates no version bump, commit, tag or release.

## Selected Arm Prefixes

The [Nuis source](../../tools/nuisc/tests/native_application_bridge/state_return_prefix_exit_argument_print.ns)
now binds the selected helper return to an arm-local `staged` value, computes
an independently tagged `prefixed` value, and checks an arm-specific fault
parameter before producing the joined result. The skipped-call arm performs its
own local computation and check. Only the final value leaves either arm.
Post-join computation, prints and checks still precede complete State publication.

The [iteration admission adapter](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_iteration.rs)
reuses the [shared prefix proof](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_prefix.rs)
for paired typed destinations, sequential local typing, freshness, observed
suffix use and the 32-statement arm limit. Predicate and both arms share the
4096-node and depth-64 expression limits. The completed effect-call catalog is
used only in the separate iteration profile, not promoted to the pure catalog.

This shape admits straight `let`/`const` initializers and a final matching typed
declaration. Calls may carry separately admitted effects. Standalone prints,
nested control/early exits, local or outer rebinding, forward/self reads,
unready captures and branch-private exports are not admitted by this shape.
Initializer roots retain the non-logical iteration expression restriction;
ordinary logical conditions keep their existing shared guard rules.

The [scalar control builder](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_control.rs)
preserves each whole prefix inside its guarded value-returning arm, replacing
only the final destination with a return. The owner's continuation consumes one
ready result. The [hygiene pass](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_control_hygiene.rs)
uses independent local maps for each prefix and exports only the joined identity,
including collisions between an outer branch and its moved suffix.

[General regressions](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_iteration_prefix_tests.rs)
exercise five exact scalar result kinds, declaration reordering, scope and
type refusals, the 32/33 arm limit, 63/64 expression depth, two individually
bounded arms that exceed the shared budget together, and linear growth over
64 joins. Five-kind coverage is admission/normalized-NIR evidence; the full
native fixture joins i64 values in f32/f64 State sessions.

## Selected Checks And Failure Order

Open/event/close inputs are 46/48/2 words. The predecessor's 44 input words
are retained, followed by independent called-arm and skipped-arm prefix faults.
State remains two words, with unchanged base child count two, 12-work ceiling
and 192-helper-entry ceiling.

The first return still decides whether the second helper runs. Its actual
selected result is transformed with a bit-34 prefix tag, observed by post-join
printing, then decoded for State. Native source contrasts replace that tag
with bit 41 and the sign bit, so substituting an earlier value or truncating
the prefix computation changes observable output.

Two healthy sequence controls poison the unused arm's prefix check independently.
The zero-return control also poisons the skipped helper. These must succeed,
not merely match a model in which both arm checks happen to pass. Prefix faults
at offsets 16, 17, 48 and 49 retain already completed helper effects but suppress
post-join prints and publication. Earlier caller/float/helper faults keep their
own exact first-failure prefixes; later post-join faults remain independently
covered. Completed output is not rolled back.

The [model](../../tools/nuisc/tests/native_application_bridge/state_return_prefix_exit_argument_print_cases.rs)
reuses predecessor input rows but defines its own ordered transition from the
first helper's actual return. It does not derive expectations from compiler IR
or rewrite an already-produced trace. The raw matrix contains 23 healthy and
42 fault sequences per width, including the predecessor's 33 faults unchanged
in role and nine new prefix-fault/collision sequences.

[Native/reference checks](../../tools/nuisc/tests/native_application_bridge/state_return_prefix_exit_argument_print.rs)
also withdraw each of twelve leaf and six post-join print grants separately
under both declaration orders. Reference rejected events and faulted close keep
both accepted State words. Native traps terminate the process; this does not
claim recoverable native rollback.

The [CLI driver](../../tools/nuis/tests/native_session_workflow/state_return_prefix_exit_argument_print.rs)
selects four finite healthy sequences (including both poisoned-unused-prefix
controls) and ten fault sequences. Healthy runs precede and follow faults on
fresh builds, cache hits and two independent source-free restorations in both
widths: 144 process runs. This is a selected delivery matrix, not every raw row.
Source, manifest and project cache are removed before restoration. State,
stdout, completion and binary/container/host-object/envelope/lifecycle/packaging
identity remain checked. Direct native probes own exact trapping prefixes;
CLI output may be buffered at a trap.

## Validation

Fresh selected validation on 2026-11-11:

| Selected Suite | Result |
| --- | --- |
| Source admission, scalar joins/control and iteration effects | 102 passed |
| Prefix/join native, reference, return, grant and budget controls | 15 passed |
| Fresh, cached and source-free CLI delivery | 1 passed; 144 process runs |
| Tensor selection, coverage, drift and complete predecessor lineage | 40 passed |
| Development and repository scripts | 40 passed |
| Repository line budgets and exception ratchet | 2 passed |
| Host absolute-path policy | 1 passed |

These are 201 selected test functions, including Python tests. Repeated runs,
matrix rows and drift guards are not additional test functions. Native/reference
validation completed in 89.33 seconds; CLI delivery completed in 386.66 seconds,
including intended failures. These are audit timings, not performance claims.
This is selected host validation, not full workspace or CI acceptance.

The rebuilt ordinary CLI reports 4429 passing drift checks and zero failures.
All 4416 predecessor IDs retain order, with thirteen new feature guards.
All six complete predecessor fields and hashes match source and runtime.
Coverage, hierarchy and task lineage are clean. Historical commands pass syntax
checks; their quoted no-op suffix executes without replaying old jobs.

UTF-8 validation covers 6150 tracked files; documentation checks cover 4930 local
links. Formatting, diff whitespace and architecture checks pass. The dependency
graph remains at 29 packages, six domains and 74 normal/build edges, excluding
dev-only edges. No line-budget exception or runtime quota was increased.

## Scope And Next Gate

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Next support standalone i64 print statements inside selected prefixes, with
explicit site grants, original read readiness, shared budgets and exact order
relative to prefix faults and final-result publication. Do not widen this into
arbitrary effects, control flow or captures without separate proofs.

This is bounded host CPU evidence, not Linux/GPU parity, general resource or
I/O/async capture, stable ABI, arbitrary memory safety or completed self-hosting.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- state_return_prefix_exit iteration_prefix_join iteration_value_join scalar_control scalar_joins iteration_effects state_return_join_exit --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_return_prefix_exit state_return_join_exit --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- state_return_prefix_exit --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test file_line_limit --locked -j1 -- --test-threads=1
python3 -m unittest discover -s scripts/tests
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-host-absolute-paths.sh
python3 scripts/check-text-encoding.py
bash scripts/check-doc-links.sh
python3 scripts/check-workspace-architecture.py
cargo fmt --all -- --check
git diff --check
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --locked -j1
target/debug/nuis dev-tensor --json
```

## Predecessor Lineage

All six complete result-join fields and SHA-256 hashes are retained. Historical
validation commands remain quoted no-op here-document data. Syntax-check the
full command through stdin and execute only the inert history suffix, never
the historical command chains.

| Complete predecessor field | SHA-256 |
| --- | --- |
| `evidence` | `3f276e21b374afce828c9d2ffe23549302fa378e1e23684218bc308657ef61ab` |
| `next_step` | `10e286c8506f0b58018d4506870b0048b00792bd35934edae2949b8046fc2fc8` |
| `blocker` | `a2caa354d0d7d8a1d9dfc3868badb599be9f27ff7faedefabccb025c827b9fe5` |
| `next_action` | `33b2fd4e45b23cc1cdca8d93b0708fddb2c4d9350fc3a6b5ad39dd5890edb5e5` |
| `validation_command` | `a38dac0dcbeec27e907ed9a93036d6227d3baa903a05443e001127d5a50c397a` |
| `expected_artifact` | `554ec9b1e5c5025df30b52415df96830e6e15d8cc44e8c6f35066101f48d8946` |
