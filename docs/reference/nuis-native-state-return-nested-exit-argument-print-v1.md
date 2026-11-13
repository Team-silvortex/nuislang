# Native State Return Nested Exit Argument Print V1

Audit date: 2026-11-11. Checkout baseline: beta-0.18.8, `283c7c28`.
This worktree follow-up extends the
[selected-prefix print checkpoint](nuis-native-state-return-print-exit-argument-print-v1.md).
It creates no version bump, commit, tag or release.

## Nested Selected Prefixes

The [existing nested-prefix traversal](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_nested_prefix.rs)
now exposes shared root collection. Pure and iteration consumers retain separate
expression/effect permissions but share the 32-statement arm, 128-statement tree
and eight-nested-level bounds before recursive type inference or helper cloning.
All predicates, initializers, print operands and results share the existing
4096 expression-node/depth-64 budget; predicates alone retain logical-root rights
in the iteration profile. Descending into a child does not reset these budgets.

The [prefix proof](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_prefix.rs)
recursively validates complete matching fresh scalar destinations in the
print-enabled profile. Each child's result must be used by its remaining parent
prefix. Ancestor result names stay blocked until complete, and only the child's
final typed identity enters the parent scope. Inner locals, incomplete/discarded
joins, rebinding, early exits and arbitrary control remain rejected. Pure flat
prefix and pure nested value plans keep their existing effect vetoes.

The [hygiene pass](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_control_hygiene.rs)
preflights paired prefix structure before recursively renaming independent
lexical maps. A child's final identity is then visible only to its own parent's
remaining work. Ordinary non-join branch traversal retains its fallback.
The existing guarded value-returning builder handles execution without new
fixture-specific paths or backend-specific rules.

[Compiler regressions](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_iteration_nested_tests.rs)
cover five scalar result kinds, reversed declaration order and colliding moved
suffixes, leaked/forward/ancestor reads, types and print-operand purity.
Boundary controls cover eight/nine levels, 128/129 shared statements, individually
bounded child expression trees exceeding the combined limit, and linear helper
growth for sixteen nested pairs. Those are NIR/admission checks, not native
execution of every kind or maximum-depth source.

## Focused Execution Witness

The [compact Nuis session](../../tools/nuisc/tests/native_application_bridge/state_return_nested_exit_argument_print.ns)
has a two-word f32/f64 plus i64 State and 10/12/2 open/event/close words.
A selected outer arm contains a second result-producing branch, an effectful
leaf call, distinct full-width result transforms and local prints/checks.
The common suffix observes the joined value before a final check and complete
State publication. Loops carry the last computed value into later iterations;
subsequent events consume accepted State, not their decoy initial fields.

This deliberately isolates the new feature instead of copying another large
historic input matrix. It is not a superset of that matrix: the predecessor's
larger nested-parent/child fixture remains unchanged and is rerun separately.
The runtime ceilings remain 12 work units and 192 helper entries.

The [independent transition model](../../tools/nuisc/tests/native_application_bridge/state_return_nested_exit_argument_print_cases.rs)
models source order directly, without deriving results from IR or rewriting
finished traces. There are 27 healthy and 19 fault sequences per width, including
three independently poisoned unused paths, changed event selections, zero trips,
wrapping full-width control values and raw signed-zero/NaN words. Faults cover
both leaf arms, the enclosing/skip prefix and common suffix, later iterations
and collisions. A manual trace checks first-failure priority independently.
Bit-41/sign-bit source contrasts change the inner result itself.

[Native/reference tests](../../tools/nuisc/tests/native_application_bridge/state_return_nested_exit_argument_print.rs)
withdraw every discovered print site's grant separately under both function
orders and widths. Rejected reference events and faulted close preserve both
accepted State words. Native traps terminate the process rather than providing
recoverable rollback. Work and helper-entry rejection remain separate controls.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/state_return_nested_exit_argument_print.rs)
uses four finite healthy sequences before and after eight fault sequences on
fresh builds, cache hits and two source-free restorations in both widths:
128 process runs completed. Source, manifest and project cache are removed
before restoration. State, stdout, completion and artifact identities are checked.
Direct native probes own exact trapping prefixes; CLI stdout may be buffered.

## Validation

Selected validation passed on the host checkout:

| Gate | Result |
| --- | --- |
| Compiler admission, shared pure/effectful joins and lowering regressions | 190 passed |
| New and preceding native/reference suites | 12 passed |
| CLI fresh/cache/source-free delivery | 1 passed; 128 process runs; 145.41 seconds |
| Development tensor tests | 42 passed |
| Script regressions | 40 passed |
| File line budgets | 2 passed |
| No host absolute paths in checked-in docs | 1 passed |

This is 288 selected tests, not the entire repository suite. All 4453 drift
checks pass: 4441 predecessor IDs retain their order and twelve new guards were
added. The selected coordinate remains active/99; coverage, manifest, milestone,
hierarchy and task lineage audits are clean. All six current fields match the
CLI report and retain the complete predecessor bytes and hashes below.

UTF-8, documentation links, workspace architecture, formatting and whitespace
checks pass. The delivery compiler SHA-256 remained
`a6d4424ad5ebd750af646be86940a7e7cba32cf543f9696501f65654d48362f8`
throughout the run; final metadata rebuilding happens only after delivery ends.
The earlier reference-test adapter incorrectly treated the new i64 `trips`
parameter as an old i32 slot; its mapping was fixed and both native/reference
suites rerun successfully. No production type rule was relaxed for that test.

## Scope And Next Gate

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Next support nested exact value-owned record results and their field consumers
inside this effectful prefix profile. Do not infer resource/reference admission
from scalar joins.

This is bounded host CPU evidence, not Linux/GPU parity, stable ABI, general
memory safety, a completed native engine or completed self-hosting.

## Reproduce

Run serially. Do not rebuild the compiler during the delivery test: its executable
identity participates in compile-cache keys. Historical commands remain inert data.

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- state_return_nested_exit state_return_print_exit iteration_nested_join scalar_joins scalar_control scalar_helpers conditional_values iteration_effects --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_return_nested_exit state_return_print_exit --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test file_line_limit --locked -j1 -- --test-threads=1
python3 -m unittest discover -s scripts/tests
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-host-absolute-paths.sh
python3 scripts/check-text-encoding.py
bash scripts/check-doc-links.sh
python3 scripts/check-workspace-architecture.py
cargo fmt --all -- --check
git diff --check
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- state_return_nested_exit --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --locked -j1
target/debug/nuis dev-tensor --json
```

## Predecessor Lineage

All six complete print-prefix fields and their SHA-256 hashes are retained.
Syntax-check the full current command through stdin and execute only its quoted
no-op history suffix, never previous job chains. Existing prefix-plan guards
follow the shared root collector and recursive print proof; IDs remain intact.

| Complete predecessor field | SHA-256 |
| --- | --- |
| `evidence` | `2a8999d4f2cce77acf6adbe12492e15f9d834a07c75df2a802109d1da25ae38f` |
| `next_step` | `105e8cb26cbf416fc97e6547112d1fb8f910ecd040a504960995487a72d2f9f8` |
| `blocker` | `e926ca4fea7a9f97466c58b8c06682ed4dd7759aeeba00d8ac7f3db8f367ccc5` |
| `next_action` | `e1882cf69649fea94dde4f3394b939c075630d8ead24b79c9b8380ed7a79f737` |
| `validation_command` | `7bc4d14d8a0f4a81e07a8200b03c9d82062d718e0f57a19588cb5a6273b74326` |
| `expected_artifact` | `f4bdeee321f5612ce949fe9a162ee1c8780aaca3e8b4d7d262585c85a45ff6f6` |
