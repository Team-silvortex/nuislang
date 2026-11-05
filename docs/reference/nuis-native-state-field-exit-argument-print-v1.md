# Native State Field Exit Argument Print V1

Audit date: 2026-11-05. Checkout baseline: beta-0.18.5, `bbb00651`.
This completes the worktree follow-up to the
[grouped-exit checkpoint](nuis-native-state-grouped-argument-print-v1.md).
No version bump, commit or release is created here.

## Updated Field Exit Conditions

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/state_field_exit_argument_print.ns)
passes post-update `current.total` to both grouped exit confirmations. Continue
retains `A && (B || C)` and break retains `(D || E) && F`; only the two
confirmation operands change from index/target comparisons to exact f32/f64
field classification. Mode zero tests negativity, mode one tests zero, and
other modes test unordered self-comparison. Each reached confirmation performs
its checked operation before producing the boolean result.

Raw [callee observers](../../tools/nuisc/tests/native_application_bridge/state_field_exit_argument_print_cases.rs)
record the actual float bits, mode and guard. The f32 observer zero-extends the
bitcast i32 word; f64 preserves all 64 bits. Signed zero, subnormal values,
maximum finite values, infinities and signed NaN payloads distinguish argument
transport from comparison results. A successful continue skips the whole break
tree and suffix; failed outer conditions and successful inner alternatives keep
their original short-circuit boundaries.

[Source controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/parent_reentry_argument_print_tests.rs)
reject moving either exit before the record update. Print-only entry-field
permissions do not make those ordinary predicate operands ready. Wrong float
argument types, missing/duplicate/wrong continue steps, effects after exits,
forged control names, mutable bounds and inherited owner/alias refusals remain
rejected. Both widths and declaration orders are checked. No production
admission exception or new source syntax is needed.

## Independent Models And Accepted State

The [field-exit model](../../tools/nuisc/tests/native_application_bridge/state_field_exit_argument_print_cases.rs)
uses raw sign-bit transitions and IEEE bit classification to predict requested
and completed children, observed indices, six predicate counts, suffix calls
and final State. Separate checked lazy trace evaluation determines reached
faults and output prefixes. The shared predecessor grouped profile is retained,
not rewritten into a field-dependent model.

Each width covers 42 raw sequences, twelve finite sequences and 22 fault rows.
The 25-word input remains identical to the grouped profile. Subsequent callbacks
use accepted State rather than their initial-value decoys, including changes to
next-parent reservations. Reference execution counts each reached source helper
once and retains the last accepted State after a failed event and faulted close.
Native probes flush exact operand-entry traces and never inspect pending output
memory after a trapping callback.

The seeded schedule requests `[0, 3, 0, 4]` children and completes all of them,
with six predicate counts `[7, 2, 6, 4, 1, 2]`. A subsequent callback starts from
accepted negative State, requests `[0, 4, 0, 4]`, and changes those counts to
`[8, 2, 8, 6, 2, 2]`. Modes, skipped poisoned operands, confirmation vetoes,
later-parent failures and failures before the update have independent controls.

## Work And Entry Budgets

Positive, negative and zero-State schedules retain exact 11/10, 12/11 and 10/9
work accept/reject boundaries. Producer failure still precedes child reservation
and exit operands. Continue steps once, break does not step, and neither refunds
reserved work.

Initial native validation exposed helper-entry exhaustion in the sequential
return classifiers; rewriting them as nested conditional expressions still
outlined branch functions. This was a witness cost issue, not permission to
raise the quota. The final classifiers use equivalent total, helper-free boolean
comparisons after the same checked prefix. All healthy cases retain the existing
192-entry limit, including eight completed children and the IEEE matrix.

The [outlined-classifier regression](../../tools/nuisc/tests/native_application_bridge/state_field_exit_argument_print.rs)
restores the equivalent sequential-return form and verifies its entry-budget
rejection at 192 in both widths and declaration orders. It checks the diagnostic
and valid output prefix without State/status publication. Limits eight and 64
remain separate negative controls. No runtime entry accounting is bypassed or
changed.

## Artifact Delivery

The [CLI matrix](../../tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs)
exercises fresh builds, cache hits and two source-free restorations in each
width. Source, manifest and project cache are removed before restoration.
Healthy sequences run before and after each fault batch, checking State, output
and binary/container/host-object/envelope/lifecycle/packaging identity. Twelve
healthy sequences twice plus 22 fault rows across four delivery paths and two
widths produce 368 artifact runs. Non-finite payload evidence remains in native
word probes rather than being inferred from finite CLI serialization.

CLI trapping output may be buffered; direct native observers own exact
operand-order evidence. A rejected child does not certify a successful session
or replace the accepted State with partially computed output.

## Validation

Final selected runs after resolving the classifier quota failure:

| Selected Suite | Result |
| --- | --- |
| Compiler source/refusal regression | 60 passed |
| Shared parent-reentry native/reference regression | 53 passed |
| Updated-field CLI delivery matrix | 1 passed |
| Tensor and complete-predecessor lineage tests | 28 passed |
| Script unit tests | 40 passed |
| Repository file budgets | 2 passed |
| Host absolute-path policy | 1 passed |

Total: 185 distinct selected tests, including ten field-exit/lineage tests
(two source, six native/model, one CLI and one tensor). Initial failing runs,
focused subsets and reruns are not added again. The final shared native suite
completed in 346.64 seconds; CLI delivery completed in 642.82 seconds with all
368 artifact runs. This is not a full-workspace or multi-platform test run.

The ordinary CLI reports 4293 passing drift checks, zero failures and all 4283
predecessor IDs retained in order alongside ten field-exit checks. The unfinished
field-exit checkpoint's four initial drift failures are resolved. Coverage,
hierarchy and task lineage are clean; all six runtime/source fields agree and
retain their complete predecessor values/hashes. Historical command syntax and
only the quoted inert history suffix were checked, without replaying old work.

UTF-8 validation covers 6063 tracked files; local-link validation checks 4805
links. Formatting and whitespace checks pass. Dependencies, admission rules,
loop/helper limits and the active/99 scope marker are unchanged.

## Scope And Next Gate

The selected coordinate stays
`standard-library/ns-nova/persistent-application-session`, active/99.
Next compose checked scalar computations over updated fields at grouped
exit-helper arguments. Preserve left-to-right, once-only evaluation, argument
faults before predicate checks, IEEE behavior, accepted State and exact quotas.
This does not grant pre-update ordinary reads, arbitrary resource/I/O/async
capture, stable ABI, Linux/GPU parity, full native-engine closure or self-hosting.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- state_field_exit_argument_print state_grouped_argument_print state_short_circuit_argument_print state_predicate_argument_print state_mixed_argument_print state_continue_argument_print state_break_argument_print state_bound_argument_print fallible_bound_argument_print parent_bound_argument_print child_progress_argument_print parent_reentry_argument_print epoch_local_ child_print_epoch local_scalar_argument_print loop_bool_projections float_compare_argument_print_epoch iteration_effects_exits --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- parent_reentry_argument_print:: --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- parent_reentry_argument_print::state_field_exit --test-threads=1 --nocapture
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

All six complete grouped-exit fields are retained with these hashes. The
[persistent lineage regression](../../tools/nuis/src/dev_tensor_tests.rs)
checks the full preceding contents, not just checkpoint labels; its initial
negative run rejected the missing evidence checkpoint. Historical commands are
quoted no-op here-document data. Validate syntax through stdin and execute only
the inert historical suffix, never replay that history.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `2aab970eb74614ba56e4de4751aca04d8e4ef44e7d25c9d2330c1ff5bbe93f7e` |
| `next_step` | `ca3721b12d113c0d779f9655d47886e20267672fce504dfb3b4247dfca0796a3` |
| `blocker` | `c9571938cc6f7d1993addd33447bbaad97761bf2ae3c0c33e05a24ce8882c391` |
| `next_action` | `7ac7d3b1f5d3afd01c9c5f346aa9764607e9c39980e2d39ff92a050691d39771` |
| `validation_command` | `5a8d5cf5e406dc3a31296d6e0e43e24a56cc759746562fb90ff1afad19aeffef` |
| `expected_artifact` | `bce05e7a09be90d71553979fb572d25ee59017c88d144df2f01b2d40a2c13563` |
