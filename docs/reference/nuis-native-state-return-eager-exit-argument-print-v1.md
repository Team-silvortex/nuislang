# Native State Return Eager Exit Argument Print V1

Audit date: 2026-11-13. Checkout baseline: beta-0.18.8, `283c7c28`.
This worktree extends the [local-initializer checkpoint](nuis-native-state-return-local-exit-argument-print-v1.md).
No version bump, commit, tag or release is created.

## Eager Boolean Operands

Fresh bool local initializers now admit short-circuit subtrees inside `==`,
`!=` and `^`. Their operands are evaluated eagerly, left before right;
short-circuit AND/OR inside each operand retains its own guarded RHS.
Explicit/inferred `let` and `const bool` use the same sequential type proof.

The [shared root walker](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_prefix.rs)
adds an explicit `Root` permission carried only by initializer roots. It is
retained through logical/eager-bool operators but reset at calls, constructors,
projections and casts. Predicate and print roots do not receive it. Record fields
retain their previous logical-root policy, not the new eager-child permission.
The [prefix collector](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_nested_prefix.rs)
shares one traversal between existing callers and the initializer-aware profile.

The [prefix proof](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_prefix.rs)
still bounds the complete source tree before sequential type inference. The
completed call-catalog check is unchanged. The
[scalar control builder](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_control.rs)
uses existing recursive guarded predicate lowering for eager roots as well as
logical roots and record fields, before binding publication and after preceding
statements. No opcode, provider-specific implementation or new ABI is added.

Original limits remain 32 statements per arm, 128 per tree, eight nested levels,
4096 source expression nodes/depth 64, a shared 32 logical-edge allowance, and
the separate shared 4096 neutral-node budget. Tests combine eager local, predicate
and field roots at the 32/33-edge boundary; neither field nor argument roots can
inherit initializer permission. Pure profiles and nominal/scope checks remain.

## Execution Witnesses

The [persistent source](../../tools/nuisc/tests/native_application_bridge/state_return_eager_exit_argument_print.ns)
wraps the earlier local AND value in equality with the selected outer predicate.
The OR value uses inequality and XOR with its selected inner predicate. These
are identity transformations in their selected arms, so the existing independent
39-healthy/27-fault model remains valid without copied expectations. Existing
f32/f64 State, print grants, 12/14/2 callback words and 12/192 work/helper ceilings
remain unchanged. This witness proves integration/delivery, not independent
right-operand side effects.

The separate [paired-effect source](../../tools/nuisc/tests/native_application_bridge/state_return_eager_pair.ns)
has an AND call on the left and an OR call on the right. Its
[independent execution model](../../tools/nuisc/tests/native_application_bridge/state_return_eager_pair.rs)
checks all three eager operators with both let and const declarations, all gate
and return-value combinations, skipped poisons, left/right/following failures,
and simultaneous failures. Every variant retains 81 sequence cases, including
poisoned but unselected calls. Manual traces distinguish left failure from right
failure and ensure the right side runs even when the left result is false.
Native probes check exact trap prefixes; reference execution checks accepted
State preservation, fault phase and close behavior. Native traps are not
recoverable transaction rollback. This paired probe uses f64 plus i64 State.

The pair is invoked through a one-iteration state loop, matching this checkpoint's
entry contract. An initial direct, non-loop aggregate-return version instead hit
`conditional fallible return requires guarded helper lowering` in `work`.
Adding the loop made the existing helper-normalization route available; it did
not fix that separate direct-entry limitation. Do not claim general function
entry coverage from these witnesses.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/state_return_eager_exit_argument_print.rs)
reuses the predecessor driver for six healthy sequences before/after eight fault
sequences across fresh builds, cache hits and two source-free restores in both
widths. Source, manifest and project cache are removed before restoration; State,
completion, output and artifact identities are checked. Each source contributes
160 program runs. Exact trap traces belong to direct probes, not buffered CLI stdout.

## Validation

All 335 selected tests passed. This is not a full-workspace test run.

| Selected validation | Passed |
| --- | ---: |
| Compiler admission/lowering regressions | 231 |
| Native/reference eager, paired and preceding local probes | 13 |
| CLI fresh/cache/two source-free delivery tests | 2 |
| Status tensor regressions | 46 |
| Script regressions | 40 |
| File-line limits | 2 |
| Host absolute-path regression | 1 |

The final native/reference set passed in 109.97 seconds. Delivery completed 320
program runs (160 per source) in 448.82 seconds, without rebuilding the CLI during
the test. The SHA-256 sampled during the first f32 restore phase and at completion
matched: `6de8a09c55069cb6c9899ae1e6737bccb6daacc13e7b85c47eed14ae521cac29`.

Tensor audit passes all 4501 drift checks: 4489 predecessor IDs remain in order
and 12 checks cover this boundary. All six source/runtime fields and complete
predecessor hashes match. Coverage, manifests, milestones, hierarchy and task
lineage are clean. Legacy context guards retain their IDs and now check the
extended context; old modes explicitly retain disabled initializer permission.

Formatting, text encoding, document links, workspace architecture and
`git diff --check` pass. No Linux or device-provider execution was performed.

## Scope And Next Gate

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. Next normalize direct non-loop effectful aggregate-return helpers
through the same guarded path, without requiring a loop to activate it.
Ordinary bool argument gates, condition/record-field eager permission and
resource/reference captures remain separate boundaries.

This is bounded host CPU evidence, not Linux/GPU parity, stable ABI, general
memory safety, a complete native engine or completed self-hosting.

## Reproduce

Run serially. Do not rebuild the CLI during delivery: executable identity is part
of cache keys. Historical command chains remain quoted no-op data.

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- state_return_eager_exit state_return_local_exit state_return_logical_exit state_return_record_exit state_return_nested_exit state_return_print_exit scalar_joins scalar_control scalar_helpers conditional_values iteration_effects control_values logical_conditions ready_values --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_return_eager state_return_local_exit --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test file_line_limit --locked -j1 -- --test-threads=1
python3 -m unittest discover -s scripts/tests
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-host-absolute-paths.sh
python3 scripts/check-text-encoding.py
bash scripts/check-doc-links.sh
python3 scripts/check-workspace-architecture.py
cargo fmt --all -- --check
git diff --check
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- state_return_eager_exit state_return_local_exit --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --locked -j1
target/debug/nuis dev-tensor --json
```

## Predecessor Lineage

All six complete local-initializer predecessor fields retain their bytes and
SHA-256 hashes. Syntax-check the current command via stdin and execute only its
quoted historical suffix, never the preceding job chains.

| Complete predecessor field | SHA-256 |
| --- | --- |
| `evidence` | `3cd11f64bd9fa030451fd8b1eaf7e64925d00ee52a399334647adcd2404adc81` |
| `next_step` | `c8b57940b1d0ea47b14d7a582f806462bc9f253cebe55e2cf590e757978e2460` |
| `blocker` | `b240e6e29207ce2183fb33365a024ee4537ba9fabde38675e6b986d81982662e` |
| `next_action` | `23b3ad4248cbe9f2373e4e043d02a07d74a62af9ce8c236e791d80292272946b` |
| `validation_command` | `bf517060b756f3a15e347e74075fc85c8c5fe1cea80bf9c1c4db1c5aeecd80c9` |
| `expected_artifact` | `f43856bcc9e4318afdd33c16edb348fea8ea3b22adfcf3cdc3588d9e48fed9f0` |
