# Native Logical Scalar Predicates V1

This bounded CPU follow-up continues the [computed predicate proof](nuis-native-computed-scalar-predicates-v1.md)
from Git beta-0.17.6 at 1feb6761 without changing the version.

The [logical value follow-up](nuis-native-logical-scalar-values-v1.md) adds a
separate bool-arm proof. This receipt retains its original predicate-only
boundary and validation results as historical evidence.

## Logical Predicate Contract

[Scalar join analysis](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins.rs)
adds a separate logical-root proof; the preceding ready, guarded and non-logical
computed rules are unchanged. The original condition and both value arms share
one 4,096-work/64-depth syntax budget and a maximum of 32 logical edges before
recursive inference, cloning or helper allocation. Only the condition's direct
logical tree gains lazy edges. Logical expressions inside call arguments,
non-logical operands and value arms remain excluded. Completed pure-value
dependencies must prove a bool condition and matching explicit scalar arms.
The destination is fresh and observed outside the complete two-arm declaration.

The [shared helper factory](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values.rs)
composes the [existing logical normalizer](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_logical.rs)
only after the complete source proof. Each guarded edge evaluates its original
LHS once as argument zero and leaves the original RHS inside its selected helper
arm. A total logical subtree needs no new helper. The final predicate is argument
zero of the value helper; both value arms stay guarded. Generated names are not
re-proved against an incomplete catalog. No engine-specific case, new public ABI
or relaxation of strict scalar-write or pure/effect authority is introduced.

## Executable Witness

The [Nuis application](../../tools/nuisc/tests/native_application_bridge/logical_scalar_join.ns)
uses `(decide(choice, condition) && right(rhs)) || fallback(other)` in an
effectful record callee, projected directly by an early return from an effectful
loop. All three original predicate callees can fail. The selected value divides
or computes remainder, and a late unused field must execute before publication.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/logical_scalar_join_cases.rs)
uses explicit Rust branches, wrapping ordinary addition and checked division/
remainder. Healthy rows cover both outcomes and all nested short-circuit paths,
skipped poisoned RHS calls, a zero-width loop and an unselected call site.
Eleven failure cases cover caller arguments, each predicate stage, selected
value checks, signed minimum/-1 division/remainder and the late unused field.
The accepted prefix retains State 64 -> 18 -> -2; failed events never replace -2.

[Native/reference checks](../../tools/nuisc/tests/native_application_bridge/logical_scalar_join.rs)
run normal and reversed YIR. Test-only markers 83/84/85 observe the actual entries
of decide/right/fallback without granting source effects. Exact flushed native
traces prove once-only LHS, skipped RHS, selected failure prefixes and failure
before value/State publication. Reference events count each original callee and
pin last-complete-State retention, faulted admission and failed completion.

[Compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_logical_tests.rs)
pin shared lazy-helper structure, total predicates without extra helpers,
completed dependency/effect revocation, idempotence, source/function-name
preservation on rejection, order independence and work/depth/edge/scope vetoes.
The compact fixture keeps the existing 4-work/64-entry/64-function profile.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/logical_scalar_join.rs)
runs fresh/cache and two independent source-free restorations. Every image
runs healthy sessions before and after all eleven failures. Binary, host-object,
envelope, lifecycle, packaging and source artifact identities must remain intact.
CLI trapping children may leave stdout buffered: those checks pin published
State, trap status and any flushed prefix; the native probe pins the full prefix.

## Validation

Bounded local M2 CPU validation passed 134 distinct tests, including seven new
tests. Repeated invocations are counted once:

- Compiler: 47 focused/adjacent checks in 13.12 seconds, including four new
  logical-predicate controls and the three preceding declaration contracts.
- Native/reference: ten checks in 10.22 seconds, including two new witnesses,
  both preceding guarded/computed families and four full-fixture bounds controls.
- CLI delivery: three checks in 153.17 seconds. The logical, computed and guarded
  families each pass fresh/cache and two independent source-free images, with
  healthy sessions before and after respectively eleven, eight and six failures.
- Borrowed-buffer integration: 17 checks in 38.42 seconds; host failure state:
  three; repository maintenance: 24 in 21.462 seconds; host absolute-path policy:
  one; file-line limits: two; status tensor: 27 in 5.79 seconds.

The ordinary CLI rebuild passed in 7.73 seconds. Formatting, whitespace, UTF-8
across 5,661 files, 4,217 local documentation links and locked offline metadata
for 29 packages passed. Drift matches all 3,626 checks and 24,781 required
patterns. The rebuilt CLI matches all six source fields, complete predecessor
hashes and inert historical commands. Coverage, hierarchy and lineage are clean;
the next task card is ready and the coordinate remains active/99.

These are targeted local CPU checks, not a full workspace run. The earlier
four-variant independent/shared matrix and 31-failure delivery, other historical
delivery families, GPU/Linux execution and physical performance were not rerun.
Their receipts remain historical evidence, not new logical-predicate results.

## Remaining Frontier

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. Logical value arms, branch-local prefixes, effectful predicates and
outer writes need separate admission and execution proofs. Resource/I/O/async
captures, stable ABI, full GPU/Linux parity and self-hosting are separate gates.
The next bounded task is a pure logical bool value arm, using the same shared
normalizer while keeping each original arm lazy and all roots budgeted together.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- scalar_joins:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- logical_scalar_join:: computed_scalar_join:: guarded_scalar_join:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- logical_scalar_join computed_scalar_join guarded_scalar_join --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete computed-predicate fields are preserved by SHA-256, with quoted
inert historical commands. This is evidence continuity, not a new claim that all
historical tests were rerun.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `48b4def82ba32dc601455c8e141ce5f68af99ef9286272b49a6e178a3bcaf68d` |
| `next_step` | `2ee64938580bb2a436631b69527db1917dc9c822bef215830ab94e0531ea7b4b` |
| `blocker` | `ffdd6efe8daa6af03bad59e3dbc587d753fe5e36bd581524c72a9f8031877614` |
| `next_action` | `131c4461b331c924bc03d173a32c2bdd5512570613842bddf7df682fb681597a` |
| `validation_command` | `af848a7110510abf0b63f1b3a8bd88f630e6d1d5092faef12bac800e5b25cc49` |
| `expected_artifact` | `f049da74e7dfbd5b70dc4e3d42512d00399206e9573b6e8ba3cb9290b746106a` |
