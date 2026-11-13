# Native State Return Print Exit Argument Print V1

Audit date: 2026-11-11. Checkout baseline: beta-0.18.8, `283c7c28`.
This worktree follow-up extends the
[selected-prefix witness](nuis-native-state-return-prefix-exit-argument-print-v1.md).
It creates no version bump, commit, tag or release.

## Selected Prefix Prints

The [shared prefix plan](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_prefix.rs)
has a separate print-enabled entry for the iteration-effect profile. Its pure
entry still refuses print statements, even if supplied the completed effect
catalog. Both entries share the paired fresh typed destination, observed suffix,
sequential local typing and 32-statement arm limit. Predicate, declarations,
print operands and final results share the 4096-node/depth-64 expression bounds.
Print expressions must have exact i64 type and completed pure dependencies;
an effectful call may initialize a local but cannot hide inside a print operand.

The [iteration adapter](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_iteration.rs)
keeps ordinary initializer/print roots non-logical. The
[hygiene pass](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_control_hygiene.rs)
renames each print use against its current arm-local map, exporting only the
final joined identity. The existing guarded value-returning builder preserves
source order without fixture names or special backend logic.

[General regressions](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_iteration_print_tests.rs)
cover five exact result kinds, reversed declarations, colliding moved suffixes,
private-name/forward-read/type/effect refusals, 32/33 statements, 63/64 expression
depth, shared cross-arm work and linear NIR growth over 64 joins.
The five result kinds and 64-join stress are normalized-NIR evidence; native
sessions retain i64 joined values in f32/f64 two-word State.

## Failure Order And Authority

The [source witness](../../tools/nuisc/tests/native_application_bridge/state_return_print_exit_argument_print.ns)
prints an arm marker, evaluates a checked full-width local print operand, performs
the later prefix check, then prints an arm completion marker. Common post-join
work follows only after the selected result is ready. Independent called/skipped
print-fault inputs extend 46 input words to 48; callback widths are 48/50/2.
State remains two words, with the same 12-work and 192-helper-entry ceilings.

The [independent transition model](../../tools/nuisc/tests/native_application_bridge/state_return_print_exit_argument_print_cases.rs)
uses predecessor input rows but computes its own print/check sequence from the
actual first helper return; it does not rewrite finished traces or inspect IR.
It covers 25 healthy and 51 fault sequences per width: all 42 predecessor fault
roles plus nine new print-operand/collision failures. Separate healthy controls
poison unused prefix checks and unused print operands. The called/skipped arm
markers distinguish selected work even where final State matches.

[Native/reference regressions](../../tools/nuisc/tests/native_application_bridge/state_return_print_exit_argument_print.rs)
withdraw each of twelve leaf, six post-join and twelve prefix print grants
independently in both source orders and widths. Prefix operand failure retains
only completed output; subsequent checks and publication do not run. A later
prefix failure retains the computed output but not its completion marker.
Post-join failure remains separate. Bit-41/sign-bit source contrasts retain
complete computed words. Rejected reference events and faulted close retain
both previously accepted State words; native traps terminate the process,
not recoverable native rollback.

The [delivery driver](../../tools/nuis/tests/native_session_workflow/state_return_print_exit_argument_print.rs)
runs four healthy sequences before and after twelve failures per artifact form:
fresh build, cache hit and two source-free restorations, for both widths.
That is 160 process runs, not the entire raw matrix. Source, manifest and project
cache are absent before restoration. State, stdout, completion and artifact
identities are checked. Direct native probes own exact failure prefixes because
CLI stdout may be buffered when the child traps.

## Validation

Fresh selected validation on 2026-11-11:

| Selected Suite | Result |
| --- | --- |
| Source admission, scalar joins/control and iteration effects | 107 passed |
| Prefix/print native, reference, grant and budget controls | 12 passed |
| Fresh, cached and source-free CLI delivery | 1 passed; 160 process runs |
| Tensor selection, coverage, drift and complete predecessor lineage | 41 passed |
| Development and repository scripts | 40 passed |
| Repository line budgets and exception ratchet | 2 passed |
| Host absolute-path policy | 1 passed |

These are 204 selected test functions, including Python tests. Repeated runs,
matrix rows and drift guards are not additional tests. Native/reference checks
completed in 97.64 seconds; the uninterrupted CLI rerun completed in 449.85
seconds. Its compiler image hash matched before/after the run. These are audit
timings, not performance claims. The earlier delivery attempt failed the cache
assertion after an in-test CLI rebuild and is not counted as acceptance.

The rebuilt ordinary CLI reports 4441 passing drift checks and zero failures.
All 4429 predecessor IDs retain order, with twelve new feature guards.
All six complete predecessor fields and hashes match source and runtime;
coverage, hierarchy and task lineage remain clean. Historical commands pass
syntax checks and their quoted no-op suffix executes without replaying jobs.

UTF-8 validation covers 6158 tracked files; documentation checks cover 4942
local links. Formatting, diff whitespace and architecture checks pass.
The graph remains at 29 packages, six domains and 74 normal/build edges,
excluding dev-only edges. No line-budget exception or runtime quota increased.
This is selected host validation, not full workspace or CI acceptance.

## Scope And Next Gate

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Next admit nested fresh typed scalar joins inside effectful prefixes, retaining
one recursive budget, selected-only evaluation and private locals.
Nested control, arbitrary effects/rebinding, general resources and async capture
are not granted by this straight-prefix extension.

This is bounded host CPU evidence, not Linux/GPU parity, stable ABI, general
memory safety, a completed native engine or completed self-hosting.

## Reproduce

Run these commands serially. The compile-cache key includes the running compiler
image; rebuilding the CLI during the delivery test invalidates its cache-hit
control. The first delivery attempt failed that assertion after a concurrent CLI
rebuild. The subsequent uninterrupted rerun passed all 160 process controls;
the compiler image hash remained identical throughout.

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- state_return_print_exit iteration_print_join state_return_prefix_exit iteration_prefix_join scalar_joins scalar_control iteration_effects --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_return_print_exit state_return_prefix_exit --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- state_return_print_exit --test-threads=1 --nocapture
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

All six complete selected-prefix fields and SHA-256 hashes are retained.
Historical commands are quoted no-op here-document data. Syntax-check the full
command through stdin and execute only its inert suffix, never old job chains.
The old prefix-plan drift guard follows the explicit print-enabled entry; its
identifier and all predecessor test/receipt paths remain intact. The existing
snapshot guard advances to the verified beta-0.18.8 identity while retaining
earlier commit anchors; no predecessor guard is removed.

| Complete predecessor field | SHA-256 |
| --- | --- |
| `evidence` | `bf7a468a572a7c011d15ae294eeca3d646cc4069b307d5f1f908f3d606652aba` |
| `next_step` | `4d8573be3d1ade4c9c689c10e0af1a3bb5b1ed9c04cea82ec9e959d80062ba61` |
| `blocker` | `4cfd27a1a17b17cb8e4fc0ceb5e9e9e843ad6ee73908e3c7bc9e6aa3215d68ea` |
| `next_action` | `581bd02a8cf51970b4820616f2c88f84643d994898fcc1f7a5c154a8ade3d04c` |
| `validation_command` | `77738a36bc46acedd127683895539eeb9045f6256753ed3af2a16892065c58b4` |
| `expected_artifact` | `96b6ad707f1cca34a39a29c70c69d53d094a04f24a2803a0e25eb23603d0211e` |
