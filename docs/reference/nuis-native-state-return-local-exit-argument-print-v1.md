# Native State Return Local Exit Argument Print V1

Audit date: 2026-11-13. Checkout baseline: beta-0.18.8, `283c7c28`.
This worktree extends the [logical-record checkpoint](nuis-native-state-return-logical-exit-argument-print-v1.md).
No version bump, commit, tag or release is created.

## Local Boolean Initializers

The [iteration prefix proof](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_iteration.rs)
now admits short-circuit roots for fresh local `let` and `const` bindings.
Both passes still walk every original predicate, initializer and record field
under one shared expression budget. Sequential inference checks the original
scope and exact declared types before introducing a binding. Inferred `let`
types and explicit `bool` declarations use the same proof.

The [shared prefix traversal](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_nested_prefix.rs)
keeps print operands closed to logical-root permission. Its separate pure-i64
print admission still applies. Logical children hidden below ordinary call
arguments, equality/inequality/XOR or projections do not gain permission.
Existing pure profiles are unchanged. Ill-typed, self-referential, forward-read,
shadowing and escaping locals remain rejected.

The [scalar control builder](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_control.rs)
normalizes admitted root logical values and record fields through one guarded
value entry. This happens at the original statement, before scope insertion,
and also handles join destinations converted into returns. No new opcode,
provider-specific implementation or speculative RHS evaluation is introduced.

Original limits remain 32 statements per arm, 128 per tree, eight nested levels,
4096 source expression nodes/depth 64, a shared 32 logical-edge allowance, and
the separate shared 4096 neutral-node budget. A mixed local/record/predicate
regression accepts 32 edges and rejects 33 across the complete pair of arms.

## Execution Witness

The [Nuis source](../../tools/nuisc/tests/native_application_bridge/state_return_local_exit_argument_print.ns)
moves the earlier record-field expressions into a `let audit: bool` AND
initializer and a `const audit: bool` OR initializer. Each follows its preceding
tag check and drives a subsequent local value branch before record construction.
The resulting audit bit still changes the published control word.

The witness has the same mathematical operation and observable trace as the
logical-record source. [Native tests](../../tools/nuisc/tests/native_application_bridge/state_return_local_exit_argument_print.rs)
and [CLI delivery](../../tools/nuis/tests/native_session_workflow/state_return_local_exit_argument_print.rs)
reuse its independent case model and parameterized assertions instead of copying
expected-output logic. The original logical-record tests remain registered.
Callbacks remain 12/14/2 words; State is f32/f64 plus i64. Work/helper ceilings
remain 12/192, with explicit rejection probes rather than increased limits.

The shared model covers 39 healthy and 27 reached-fault sequences per width,
including RHS true/false, skipped zero guards, unselected arms, changed choices,
zero trips, later iterations, raw signed-zero/NaN and full-width control words.
It checks exact native failure prefixes, accepted reference State, independently
removed print grants and bit-41/sign-bit source contrasts. Removing the final
bool consumer must not remove reached local RHS effects in either let/const arm;
an earlier tag failure
must still prevent RHS entry. Native traps are not recoverable transaction rollback.

Delivery runs six healthy sequences before and after eight fault sequences for
fresh builds, cache hits and two source-free restores in both widths. Each source
therefore contributes 160 process runs. Source, manifest and project cache are
removed before restoration. State, completion, output and artifact identity are
checked; direct native probes own exact trap traces because CLI stdout may buffer.

## Validation

All 329 selected tests passed. This is not a full-workspace test run.

| Selected validation | Passed |
| --- | ---: |
| Compiler admission/lowering regressions | 226 |
| Native/reference local and preceding logical-record probes | 13 |
| CLI fresh/cache/two source-free delivery tests | 2 |
| Status tensor regressions | 45 |
| Script regressions | 40 |
| File-line limits | 2 |
| Host absolute-path regression | 1 |

The 13 native/reference tests were rerun after extending unused-result coverage
to both the AND/let and OR/const arms; all passed in 75.09 seconds. Delivery
completed 320 process runs (160 per source) in 414.55 seconds. The CLI was not
rebuilt during delivery; its SHA-256 matched between the first fresh phase and
completion: `a1f9b4511fa14faad7d8103ecd1a378a1da5d070f947ccf40e2336a192256184`.

Tensor audit passes all 4489 drift checks: 4477 predecessor IDs remain in order
and 12 checks cover this boundary. Coverage, manifests, milestones, hierarchy
and task lineage are clean. All six source/runtime fields and complete predecessor
hashes match. Existing shape/lowering guard IDs are retained and now track the
shared logical-value entry and local-root permission.

Formatting, text encoding, document links, workspace architecture and
`git diff --check` pass. No Linux or device-provider execution was performed.

## Scope And Next Gate

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. The next gate is typed eager bool comparison/XOR operands containing
short-circuit subtrees inside these local initializers. Those hidden children
remain rejected in this checkpoint; ordinary call/print permission stays separate.

This is bounded host CPU evidence, not Linux/GPU parity, stable ABI, general
memory safety, a complete native engine or completed self-hosting.

## Reproduce

Run serially. Do not rebuild the CLI during delivery, because executable identity
participates in cache keys. Historical command chains are quoted no-op data.

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- state_return_local_exit state_return_logical_exit state_return_record_exit state_return_nested_exit state_return_print_exit scalar_joins scalar_control scalar_helpers conditional_values iteration_effects control_values logical_conditions ready_values --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_return_local_exit state_return_logical_exit --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test file_line_limit --locked -j1 -- --test-threads=1
python3 -m unittest discover -s scripts/tests
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-host-absolute-paths.sh
python3 scripts/check-text-encoding.py
bash scripts/check-doc-links.sh
python3 scripts/check-workspace-architecture.py
cargo fmt --all -- --check
git diff --check
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- state_return_local_exit state_return_logical_exit --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --locked -j1
target/debug/nuis dev-tensor --json
```

## Predecessor Lineage

All six complete logical-record predecessor fields retain their bytes and SHA-256
hashes. Syntax-check the current command through stdin; execute only the quoted
historical suffix, never the preceding job chains.

| Complete predecessor field | SHA-256 |
| --- | --- |
| `evidence` | `cee4aaf0f7aa4527f838c9ee363d37d1f58183603dde3a4aadda4a919c3d1cca` |
| `next_step` | `3d3bfbd2b6cfd0a188203fe35c75bc9f1ec2fd7a7246e153263a0a10d3ec4624` |
| `blocker` | `f747c823707e2ac72e02b5c17c9cd84dd1d6af1631f1489c9abe01c2c4b6155f` |
| `next_action` | `2de7ded9ea1a8139ca258b617c325b98b4b60512bec6813c65f723ef18ac0cbc` |
| `validation_command` | `665a98be5d14b76601fe8a89ed505c6052e5d5e82035c4b51b5549c059342cc2` |
| `expected_artifact` | `4508c4a14030895fe88b2c785fa67f7c0a0e131e02c33a66ca73b7167a6f6051` |
