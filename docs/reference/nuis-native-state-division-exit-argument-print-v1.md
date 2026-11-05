# Native State Division Exit Argument Print V1

Audit date: 2026-11-05. Checkout baseline: beta-0.18.5, `bbb00651`.
This extends the worktree [updated-field checkpoint](nuis-native-state-field-exit-argument-print-v1.md)
without replacing its fixtures or acceptance. No version bump, commit or release
is created here.

## Checked Division Exit Arguments

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/state_division_exit_argument_print.ns)
passes `current.total / continue_divisor` and `current.total / break_divisor` to
the existing grouped confirmations. Both divisors are ready inputs of exactly
the field's f32/f64 kind. Continue retains `A && (B || C)` and break retains
`(D || E) && F`; each quotient is evaluated only when its confirmation is selected.

Either sign of a zero divisor fails before entering the confirmation helper,
even when that helper's own integer check would also fail. False outer gates,
successful alternative branches and selected continue paths still skip their
unused computations. Failure cannot publish the callback's pending State.
The existing checked-float division policy is reused, not replaced with
infinity-returning division by zero.

[Source controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_division_exit_argument_print_tests.rs)
exercise both widths, declaration orders and inherited refusal cases. Moving
either exit before the record update remains rejected. Bool/i32 operand mistakes
and cross-width divisors are type errors; this proof does not redefine the
generic frontend arithmetic surface. Print-only epoch permissions do not make
ordinary quotient arguments ready, and no new syntax or production admission
exception is introduced.

## IEEE Values And Independent Models

The [quotient and schedule models](../../tools/nuisc/tests/native_application_bridge/state_division_exit_argument_print_cases.rs)
perform arithmetic at the declared width, with a separate raw-bit zero-divisor
test. They predict State sign transitions, parent reservations, completed work,
observed indices, six predicate counts and suffix calls. Separate lazy trace
evaluation checks selected argument failures before appending any callee-entry
observation. Scheduling stops at a reached arithmetic fault rather than inventing
later parents.

Each width covers 28 raw sequences, ten finite sequences and 19 fault rows.
The raw matrix includes thirds, signed zero, normal/subnormal underflow, maximal
finite overflow, infinities, infinity/infinity, signed NaN inputs and NaN divisors.
Known word controls pin f32 one-third to `0x3eaaaaab` and f64 one-third to
`0x3fd5555555555555` independently of the source compiler.

Only arithmetic NaNs are normalized in the native observation stream and its
oracle. Finite, infinite and signed-zero quotient words remain exact. The
observer does not modify the actual callee argument or production arithmetic;
input/carry NaN payloads remain checked as raw State words. There is no promise
about arithmetic NaN payload propagation. This follows the earlier
[binary float boundary](nuis-native-binary-float-field-epoch-print-v1.md).

The [native controls](../../tools/nuisc/tests/native_application_bridge/state_division_exit_argument_print.rs)
also reverse the division operands in source. Their independently calculated
reciprocal observations differ while the selected predicates and State remain
unchanged in the short control profiles. This tests noncommutative argument
values, not ordering between effectful producers; such producers remain the
next gate.

## Faults, State And Input Transport

Nine retained source-fault rows cover bound production, initial reads, record
update, checked predicates, suffixes and final publication. Ten new argument-fault
rows cover both zero signs at continue/break confirmations in first or later
parents, plus fallback-selected break confirmation. Colliding confirmation
checks are deliberately poisoned, but the native trace ends before confirmation
entry. Completed child counts distinguish early faults from seven completed
updates before later-parent faults.

The accepted prefix first publishes negative State, then a two-parent callback
publishes positive State. Failing callbacks carry negative initial-value decoys,
which must not replace that accepted positive State. Reference sessions preserve
it after failure and faulted close. Healthy sequences likewise reuse accepted
State across callbacks and recompute State-dependent reservations.

The fixture appends two typed float divisor inputs to the unchanged 25-word
predecessor prefix, for 27 input words. The shared native/reference transport
recognizes only those appended positions for this profile; earlier variants
retain their own layouts. This is fixture input evidence, not a new stable ABI
claim. The predecessor field profile and its raw observers remain separate.

## Quotas And Artifact Delivery

Loop work remains capped at 12 and helper entries at 192. Positive, negative and
zero-State controls retain exact 11/10, 12/11 and 10/9 work boundaries. A late
zero-divisor poison additionally loses to the earlier parent reservation check.
Limits eight and 64 still reject helper-entry exhaustion without publishing
status/State. Division adds no exemption to entry or reservation accounting.

The [CLI matrix](../../tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs)
runs ten healthy sequences before and after nineteen faults on fresh, cached
and two independently restored artifacts in each width: 312 artifact runs.
Source, manifest and project cache are removed before restoration. State,
stdout, completion status and binary/container/host-object/envelope/lifecycle/
packaging identity are checked. Non-finite payload evidence stays in native word
probes rather than finite CLI serialization. Direct probes own exact fault
prefixes; trapping CLI output can be buffered.

## Validation

Final selected runs in this checkout, without adding predecessor totals:

| Selected Suite | Result |
| --- | --- |
| Compiler source/refusal regression | 62 passed |
| Shared parent-reentry native/reference regression | 58 passed |
| Division-exit CLI delivery matrix | 1 passed |
| Tensor and complete-predecessor lineage tests | 29 passed |
| Script unit tests | 40 passed |
| Repository file budgets | 2 passed |
| Host absolute-path policy | 1 passed |

Total: 193 distinct selected tests, including nine division-exit/lineage tests
(two source, five native/model, one CLI and one tensor). Focused subsets, initial
runs and reruns are not added again. The shared native/reference run took 406.72
seconds; CLI delivery took 568.39 seconds across all 312 artifact runs. These are
functional checks, not performance measurements or a full-workspace/platform run.

The rebuilt ordinary CLI reports 4304 passing drift checks and zero failures.
All 4293 predecessor IDs retain their order alongside eleven division-exit checks.
Coverage, hierarchy and task lineage remain clean; the six source/runtime fields
agree and retain their full preceding contents and hashes. Historical command
syntax and only the inert quoted history suffix were checked without replaying
prior validation chains.

UTF-8 validation covers 6069 tracked files, and 4816 local documentation links
resolve. Formatting and whitespace checks pass. No dependency, release, source
admission, runtime quota or active/99 scope change accompanies this checkpoint.

## Scope And Next Gate

The selected tensor coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Next compose independently fallible helper-produced operands at grouped exit
arguments. Distinguish left/right producer entry, arithmetic checks and callee
guards while retaining once-only source order, skipped faults, accepted State,
source-free delivery and exact quotas. This checkpoint does not grant pre-update
ordinary reads, transitive effects, arbitrary resource/I/O/async capture, stable
ABI, Linux/GPU parity, full native-engine closure or completed self-hosting.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- state_division_exit_argument_print state_field_exit_argument_print state_grouped_argument_print state_short_circuit_argument_print state_predicate_argument_print state_mixed_argument_print state_continue_argument_print state_break_argument_print state_bound_argument_print fallible_bound_argument_print parent_bound_argument_print child_progress_argument_print parent_reentry_argument_print epoch_local_ child_print_epoch local_scalar_argument_print loop_bool_projections float_compare_argument_print_epoch iteration_effects_exits --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- parent_reentry_argument_print:: --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- parent_reentry_argument_print::state_division_exit --test-threads=1 --nocapture
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

All six complete field-exit values are retained, not abbreviated summaries. The
[persistent lineage regression](../../tools/nuis/src/dev_tensor_tests.rs)
checks their full SHA-256 values alongside the preceding grouped checkpoint.
Historical validation commands remain a quoted no-op here-document: check syntax
through stdin and execute only the inert suffix, never replay historical chains.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `15e61462dd2f9ddcb722522b85e366b8ac913e22fb42ca046cbca666747ca7da` |
| `next_step` | `71744b494a0ed788ca57b8c430894fc2489ece1702b801e085f83b2c4277a3d9` |
| `blocker` | `4a21d28b8e0782d1e94b18490804e25a7316b24c24449b119c8c5e8833f5483e` |
| `next_action` | `07b21bc487919f23bbdbcc45d6a1c591c629010e450b58fd849c551059b01f93` |
| `validation_command` | `198ca27e54366a001545f7d5119a0c72d1839e62da5d6fc8014b0c0a012e58ba` |
| `expected_artifact` | `c94482d3e50f756b426235344e2a38f45e9b4d5265b008f13182363e0e3198d9` |
