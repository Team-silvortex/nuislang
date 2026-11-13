# Native State Return Logical Exit Argument Print V1

Audit date: 2026-11-13. Checkout baseline: beta-0.18.8, `283c7c28`.
This worktree extends the [owned-record prefix checkpoint](nuis-native-state-return-record-exit-argument-print-v1.md).
No version bump, commit, tag or release is created.

## Logical Record Fields

The [shared expression walker](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_prefix.rs)
now exposes a combined typed-record/call profile. Exact bool fields at original
record roots may contain short-circuit AND/OR trees; nested record constructors
retain the same field rules. Existing pure and call-only profiles are unchanged.
The initial [prefix proof](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_prefix.rs)
still rules out hidden logical children beneath ordinary calls, comparisons or
projections before the iteration profile validates all original roots together.
Print operands keep their independent pure-i64 admission, not field authority.

The [scalar control builder](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_control.rs)
lowers logical record initializers, constants and returned records through the
existing guarded predicate machinery. Helper calls remain at the original field,
after preceding fields and before following work; neither scope insertion nor
result publication moves ahead of the initializer. Effects in an admitted RHS
still need their own backend print-site grants. Unselected RHS effects/checks
are skipped; unused record fields are still constructed.

No backend-specific path or new opcode is introduced. Existing nominal layout,
complete-field, ownership, hygiene and readiness checks remain in force. Original
limits remain 32 statements per arm, 128 per tree, eight nested levels, 4096
source expression nodes/depth 64, and the separate shared 4096 neutral-node
budget. The existing 32 logical-edge allowance is shared across every original
predicate and field, not reset per field or arm.

[Compiler regressions](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_iteration_logical_record_tests.rs)
check guarded effectful RHS helpers, reversed declarations, private scope,
unproved callees and hidden logical roots. Nested fields across two arms accept
32 logical edges and reject 33; the call-only profile still rejects those fields.
Previous scalar, record, pure logical-field and ready-value regressions remain
selected. Admission bounds are not maximum-depth native execution claims.

## Execution Witness

The [Nuis source](../../tools/nuisc/tests/native_application_bridge/state_return_logical_exit_argument_print.ns)
uses AND in one selected record arm and OR in the other. The RHS helper prints
84 before its check and 85 after it. The resulting bool changes the computed
control word, distinguishing successful true/false results from a fabricated
constant. An earlier tag-field check and later prefix/common checks establish
constructor and statement failure order. Callbacks are 12/14/2 words; published
State stays f32/f64 plus i64, with 12 work units and 192 helper entries.

The [independent model](../../tools/nuisc/tests/native_application_bridge/state_return_logical_exit_argument_print_cases.rs)
contains 39 healthy and 27 reached-fault sequences per width. They include
AND/OR RHS true and false, skipped zero guards, changed event selection, zero
trips, later iterations, raw signed-zero/NaN State, full-width control and fault
collisions. A manual trace checks output/failure positions. Bit-41/sign-bit source
contrasts validate every sequence, including cases whose changed sign makes a
previously reached RHS skip; no case is dropped merely because it succeeds.

[Native/reference checks](../../tools/nuisc/tests/native_application_bridge/state_return_logical_exit_argument_print.rs)
remove each print grant independently and keep complete accepted reference State
through rejected/faulted events. A separate source contrast removes the bool
field consumer: its reached RHS must still execute and fail, while an earlier
field failure must prevent RHS entry. Native traps end the process; this is not
recoverable native transaction rollback.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/state_return_logical_exit_argument_print.rs)
uses six healthy sequences before and after eight fault sequences across fresh,
cache and two source-free restorations in both widths: 160 process runs passed. Source, manifest and project cache are removed before restoration;
State, completion, output and artifact identities are checked. Exact trapping
prefixes belong to direct probes; CLI stdout may be buffered.

## Validation

All 322 selected tests passed; this is not a full-workspace test run.

| Selected validation | Passed |
| --- | ---: |
| Compiler admission/lowering regressions | 220 |
| Native/reference logical and preceding record probes | 14 |
| CLI fresh/cache/two source-free delivery test | 1 |
| Status tensor regressions | 44 |
| Script regressions | 40 |
| File-line limits | 2 |
| Host absolute-path regression | 1 |

CLI delivery completed in 200.52 seconds, including 160 program runs. The CLI
was not rebuilt during delivery, keeping executable identity stable for cache
verification. Tensor audit passes all 4477 drift checks: 4465 predecessor IDs
remain in order and 12 checks cover this boundary. Coverage, manifests,
milestones, hierarchy and task lineage are clean; source/runtime agree on all
six fields and complete predecessor hashes. The existing call-profile guard now
tracks the combined record/call entry, retaining its original ID.

Formatting, text encoding, document links, workspace architecture and
`git diff --check` pass. No Linux or device-provider execution was performed.

## Scope And Next Gate

The coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Next admit fresh local short-circuit bool initializers in this effectful prefix
profile. Ordinary local roots remain closed here; field permission is not a
general logical-expression or resource/reference grant.

This is bounded host CPU evidence, not Linux/GPU parity, stable ABI, general
memory safety, a complete native engine or completed self-hosting.

## Reproduce

Run serially. Do not rebuild the compiler during delivery: executable identity
participates in cache keys. Historical command chains remain inert data.

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- state_return_logical_exit state_return_record_exit state_return_nested_exit state_return_print_exit scalar_joins scalar_control scalar_helpers conditional_values iteration_effects control_values logical_conditions ready_values --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_return_logical_exit state_return_record_exit --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test file_line_limit --locked -j1 -- --test-threads=1
python3 -m unittest discover -s scripts/tests
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-host-absolute-paths.sh
python3 scripts/check-text-encoding.py
bash scripts/check-doc-links.sh
python3 scripts/check-workspace-architecture.py
cargo fmt --all -- --check
git diff --check
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- state_return_logical_exit --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --locked -j1
target/debug/nuis dev-tensor --json
```

## Predecessor Lineage

All six complete record-prefix fields retain their bytes and SHA-256 hashes.
Syntax-check the current validation command through stdin and execute only the
quoted no-op historical suffix, never the preceding job chains.

| Complete predecessor field | SHA-256 |
| --- | --- |
| `evidence` | `2bb7db70fa17fc73fd50302b3177e2440c128db5eff21360fa32127cf55796d9` |
| `next_step` | `6a74c397b41819c46190661e322cd245095cd476d46fbacd041df89fd7719f73` |
| `blocker` | `9d1c7580f9dac22679088d702ce051ff4c0aa52438821f654e51d1be11851aac` |
| `next_action` | `3e1ef1ef9dd799eab42293a317c4d745e09c567adabb313d322c1ddeff5ff316` |
| `validation_command` | `b3e61f8a659f5681486aa8f58f2e2ce2d6e4a2d619fbca6b0b5dccda7b3e79ae` |
| `expected_artifact` | `aa1123fe98ec15108ca54ecf3a52651b37f9070f6ec7b15d56228deeda46ab1f` |
