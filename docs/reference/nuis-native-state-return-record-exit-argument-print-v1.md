# Native State Return Record Exit Argument Print V1

Audit date: 2026-11-11. Checkout baseline: beta-0.18.8, `283c7c28`.
This worktree follows the [nested scalar prefix proof](nuis-native-state-return-nested-exit-argument-print-v1.md).
No version bump, commit, tag or release is created.

## Owned Record Results

The [shared prefix proof](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_prefix.rs)
now admits exact value-owned record destinations in its iteration-effect profile.
A result may contain nested owned records and supported scalar leaves; both arms
must produce the same nominal type, not merely the same physical field shape.
Bare paired record arms are supported so recursive lowering does not lose a
child result that has no local prefix. Pure scalar prefix discovery remains
separate and does not gain effect authority or record-result permission.

Original layout collection rejects references, resources, unknown types and
cycles. Every field is checked for completeness, uniqueness and exact type.
Only the complete, fresh paired destination escapes its arms; inner locals,
ancestor/forward reads, outer rebinding and independent print-operand effects
remain rejected. Existing guarded value-returning helpers and lexical hygiene
handle outlining without new backend-specific or fixture-specific paths.

The [shared record-neutral preflight](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_nested_prefix.rs)
charges two potential guarded neutral records per record-valued join across the
entire original prefix tree. It reuses the [existing neutral walker](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_discarded_records.rs),
including i32 cast costs and depth checks, with 4096 shared nodes before inference
or cloning. This conservative budget is separate from the existing source
expression budget; it does not reset at a nested join. Original limits remain
32 statements per arm, 128 across the tree, eight nested levels and shared
4096 source-expression nodes/depth 64.

[Compiler regressions](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_iteration_record_tests.rs)
cover all five scalar leaf kinds, reversed function/type order, colliding moved
suffixes, simple and prefixed record arms, exact nominal types and rejected
scope/type/effect leaks. Two child joins individually fit the neutral budget;
together they accept 4096 nodes and reject 4100. Existing pure/iteration budget
and layout regressions remain selected. This does not claim maximum-depth or
all-five-kind native execution from admission tests alone.

## Execution And Delivery

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/state_return_record_exit_argument_print.ns)
returns `Selected { payload: Payload, tag: i64, audit: i64 }` from nested
branches, consumes both payload and tag fields in the parent, and returns a
record across the helper boundary before updating the complete two-word State.
Callbacks retain 10/12/2 words, and runtime ceilings remain 12 work units and
192 helper entries. The f32/f64 State is unchanged in shape; the record payload
exercised here is integer-valued, not five-kind native record transport.

The [transition model](../../tools/nuisc/tests/native_application_bridge/state_return_record_exit_argument_print_cases.rs)
computes values and traces directly. Its 27 healthy and 19 fault sequences per
width include unused-path poisons, changed event selections, later-iteration
failures, zero trips, raw signed zero/NaN words and full-width controls. This is
a focused record witness; the preceding scalar fixture stays intact and is
rerun, rather than being silently replaced.

[Native/reference checks](../../tools/nuisc/tests/native_application_bridge/state_return_record_exit_argument_print.rs)
withdraw each discovered print grant independently, retain both accepted State
words through rejected/faulted reference events, and distinguish work/entry
limits. A separate source contrast puts a division into the unprojected
`audit` field: it must fail after the selected leaf effects but before the
parent's next print. Native traps are not recoverable transaction rollback.
Bit-41/sign-bit contrasts ensure complete field values affect the result.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/state_return_record_exit_argument_print.rs)
checks fresh/cache/two source-free restorations in both widths, with four healthy
sequences before and after eight fault sequences: 128 process runs completed.
Source, manifest and project cache are removed before restoration; State,
completion, stdout and artifact identity are checked. Direct probes own exact
trapping output prefixes; CLI child stdout can be buffered.

## Validation

Selected host validation passed:

| Gate | Result |
| --- | --- |
| Compiler admission/lowering, scalar and record regressions | 206 passed |
| New record and preceding nested-scalar native/reference suites | 13 passed |
| CLI fresh/cache/source-free delivery | 1 passed; 128 process runs; 155.97 seconds |
| Tensor and lineage tests | 43 passed |
| Script regressions | 40 passed |
| File line budgets | 2 passed |
| No host absolute paths in checked-in docs | 1 passed |

Total: 306 selected tests, not the entire repository suite. The original valid
record case was first observed failing admission, then passed with the shared
production change. Existing scalar/pure paths remain covered by the regression
selection. No runtime ceiling or backend ABI was widened to pass this witness.

All 4465 drift checks pass, retaining 4453 predecessor IDs in order and adding
twelve guards. All six complete predecessor fields retain the hashes below;
source fields and the CLI report match. Coverage, manifest, milestone, hierarchy
and task lineage audits are clean. The coordinate remains active/99.

Formatting, UTF-8, documentation links, workspace architecture and whitespace
checks pass. The compiler SHA-256 stayed
`094a6053e6bed52c50b9858b062f89df74db0048b14cd0c11cdb4fa5169fda06`
throughout delivery; final metadata rebuilding occurs only after that test ends.

## Scope And Next Gate

The selected coordinate stays
`standard-library/ns-nova/persistent-application-session`, active/99.
Next admit short-circuit boolean fields in record constructors inside this
profile, reusing the existing lazy-field contract. Logical field expressions
remain rejected here today; pure logical-record support is not effect permission.

This is bounded host CPU evidence, not Linux/GPU parity, stable ABI, general
memory safety, a complete native engine or completed self-hosting.

## Reproduce

Run serially; do not rebuild the compiler during delivery because its executable
identity participates in cache keys. Historical commands are inert data.

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- state_return_record_exit state_return_nested_exit state_return_print_exit scalar_joins scalar_control scalar_helpers conditional_values iteration_effects control_values --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_return_record_exit state_return_nested_exit --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test file_line_limit --locked -j1 -- --test-threads=1
python3 -m unittest discover -s scripts/tests
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-host-absolute-paths.sh
python3 scripts/check-text-encoding.py
bash scripts/check-doc-links.sh
python3 scripts/check-workspace-architecture.py
cargo fmt --all -- --check
git diff --check
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- state_return_record_exit --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --locked -j1
target/debug/nuis dev-tensor --json
```

## Predecessor Lineage

All six complete nested-prefix fields retain their bytes and SHA-256 hashes.
Syntax-check the full current command via stdin; execute only its quoted no-op
history suffix, never prior job chains. The new lineage test has its own file
so the existing history test does not exceed its 1000-line limit.

| Complete predecessor field | SHA-256 |
| --- | --- |
| `evidence` | `f85debe9b73bf997ccc240c5deeba5f5b453dd4b4edd651996eae6b9adab47fc` |
| `next_step` | `9b993feafa2312326b11d919c32822e24f00a03f0bd31f8d527193095b84c111` |
| `blocker` | `cb78801b5d1f34676e59f38d5df1148692e0f0d8238fc51700df0ee6be8f0eee` |
| `next_action` | `0812ceab0a83f35903ec44a8d4b2f2249da8c70dd099f4541124d4b527e5e925` |
| `validation_command` | `93f7ce0c47658ca132849c8a0df72be36849cffb883ca8b04df93fb91058a02b` |
| `expected_artifact` | `44ee1c52bf7eaa4bbf09680563af79d882d81b654ea9fab1d63e6fa77cf7f032` |
