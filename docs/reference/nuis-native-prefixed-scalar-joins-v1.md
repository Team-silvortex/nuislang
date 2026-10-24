# Native Prefixed Scalar Joins V1

This bounded CPU follow-up continues the [logical-value checkpoint](nuis-native-logical-scalar-values-v1.md)
on the Git beta-0.17.6 base at 1feb6761, without a version bump.

The [nested prefix follow-up](nuis-native-nested-prefix-joins-v1.md) adds a
separate typed-plan proof. This receipt retains its original flat-prefix
boundary and validation results as historical evidence.

## Pure Prefix Contract

The independent [prefix declaration proof](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_prefix.rs)
admits fresh pure branch-local let/const initializers before an explicit scalar
result. Both arms must end in the same fresh typed let observed outside the
branch; at least one arm has a prefix. Each arm has at most 32 bindings, including
the final result. The condition, every initializer and both final values share
one 4,096-work/64-depth/32-logical-edge source proof before recursive inference,
branch-local proof-scope cloning or helper allocation.

Completed pure-value dependencies prove each expression in its original lexical
scope. A local can read earlier locals but cannot overwrite an outer name,
redeclare another local, bind the final destination early or read a future/self
binding. Prefix scalar and supported record values are allowed; the final result
must remain scalar. Direct logical roots can occur in bool initializers and
values, but hidden logical call arguments/comparison operands remain excluded.
Effects, nested control-flow prefixes, references/resources, outer writes and
incomplete/untyped final arms retain separate proof gates.

The [shared selection factory](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values.rs)
now has one `prefix_selection` constructor for the existing selection route and
the new declaration proof. It preserves original initializers, collects original
captures, removes branch-local identities from helper parameters and converts
only the final value into a return. The existing `guard_selection` normalizes
logical local roots and the final bool return in source order, using the current
local types. Generated helper calls are not re-inferred against the old catalog.
The predicate is still evaluated once as argument zero; all local work stays
inside its selected arm. Strict scalar-write and pure/effect authority are not
relaxed, and no public ABI or engine/backend-specific case is introduced.

## Executable Witness

The [Nuis application](../../tools/nuisc/tests/native_application_bridge/prefixed_scalar_join.ns)
extends the bool-value record/field/loop-exit route with matching local names in
both arms. A fallible seed call precedes a dependent step call, a bool head
initializer, an unused checked initializer and the final logical value. The
unselected arm has poisoned prefix inputs; selected logical RHS poison remains
skippable. An unused initializer must still execute before the final logical
result, even if that result could already short-circuit. The record retains its
later unused-field check before complete State publication.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/prefixed_scalar_join_cases.rs)
reuses preceding input tables but evaluates all prefix stages independently.
Ordinary addition wraps; division checks remain fallible. Six new prefix faults
extend the ten preceding failures: seed failure, dependent-step failure and
unused-prefix failure for each arm. The last completed State remains -2 after
every failed event. Healthy rows cover all earlier truth paths, signed raw words,
live-State dependency, zero-width and unselected-call skipping.
Two extra native/reference ordering faults use RHS calls that would otherwise
run, distinguishing unused-prefix-before-RHS order from merely retaining work
on a path whose final value already short-circuits.

[Native/reference checks](../../tools/nuisc/tests/native_application_bridge/prefixed_scalar_join.rs)
observe actual callee entries using test-only markers 83-89. Marker 88 must precede
89 and the selected head; an unused-prefix fault must precede RHS entry and the
post-declaration print. Reference traces count both bool and integer callees.
All eighteen native/reference failure cases run with normal/reversed YIR;
exact flushed native fault prefixes, faulted admission and
last-State retention are pinned. Limits stay at 4 work, 64 entries and the
existing 64-function profile.

[Compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_prefix_tests.rs)
pin unchanged preceding proofs, completed dependency/effect revocation, unchanged
source/functions/names on rejection, local ordering and non-export, idempotence,
function-order independence, all scalar result kinds, and unused fields in pure
record locals. Joint expression/edge budgets, 32/33 arm sizes, depth, shadowing,
forward/self reads, type/reference mismatches and hidden logical trees have
positive or fail-closed controls.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/prefixed_scalar_join.rs)
checks fresh/cache and two independent source-free restorations. Each image runs
healthy sessions before and after all sixteen failures, retaining binary,
host-object, envelope, lifecycle, packaging and source artifact identities.
CLI trapping children need not flush stdout: State, trap status and any flushed
prefix are pinned there; the direct native probe pins the complete fault prefix.

## Validation

Bounded local M2 CPU validation passed 147 distinct tests, including eight new
tests. Repeated invocations are counted once:

- Compiler: 57 focused/adjacent checks in 13.07 seconds, including five new
  prefix controls, all preceding declaration proofs and the existing prefix and
  logical-root selection regressions.
- Native/reference: 14 checks in 14.35 seconds, including two new prefix witnesses,
  four preceding declaration families and four full-fixture bounds controls.
  The prefix witnesses cover eighteen failure cases, including both reached-RHS
  ordering probes; their earlier smaller-table invocation is not counted again.
- CLI delivery: two checks in 200.12 seconds. Prefix and preceding bool-value
  families each pass fresh/cache and two independent source-free images, with
  healthy sessions before and after respectively sixteen and ten core failures.
  The two extra native ordering probes are not claimed as CLI delivery cases.
- Borrowed-buffer integration: 17 checks in 37.72 seconds; host failure state:
  three; repository maintenance: 24; host absolute-path policy: one; file-line
  limits: two; status tensor: 27 in 5.85 seconds.

The ordinary CLI rebuild passed in 9.17 seconds. Formatting, whitespace, UTF-8
across 5,676 files, 4,246 local documentation links and locked offline metadata
for 29 packages passed. Drift matches all 3,644 checks and 24,895 required
patterns. The rebuilt CLI matches all six source fields, complete predecessor
hashes and inert historical commands. Coverage, hierarchy and lineage are clean;
the next task card is ready and the coordinate remains active/99.

These are targeted local CPU checks, not a full workspace run. The earlier
four-variant independent/shared matrix and 31-failure delivery, the guarded,
computed and predicate-only CLI delivery families, other historical delivery
families, GPU/Linux execution and physical performance were not rerun for this
checkpoint. Their receipts remain historical evidence, not new prefix results.

## Remaining Frontier

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. A bounded nested pure local selection inside a prefix needs a separate
source-declaration proof, including branch-local scope joins and shared budgets.
Effectful prefixes, outer writes, resources/I/O/async captures, stable ABI, full
GPU/Linux parity and self-hosting remain separate gates.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- scalar_joins:: conditional_values::prefix_tests --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- prefixed_scalar_join:: logical_value_join:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- prefixed_scalar_join logical_value_join --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

These are focused entry points. The coordinate's `validation_command` field
retains the complete focused/adjacent command chain; predecessor commands remain
inert history, not additional commands to execute.

## Predecessor Lineage

All six complete logical-value fields are preserved by SHA-256, with quoted
inert historical commands. Historical results are not counted as rerun tests.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `338e8b3d076412f73e1070a3b469767191f8d6cc59177b9a590de524eb4c56ac` |
| `next_step` | `b8f796ed311463063ea3530899b8d5154c727fe4a47f3f4f511f970145ca6d4d` |
| `blocker` | `84078e1c9ce098bb76ae121b7da25e1fa64d822eaa59d4fd4194000697534e88` |
| `next_action` | `a0f236a9aca6115752e82cac56d4e2f6284794b5ed60009433fddbfd79c5cc7d` |
| `validation_command` | `e985f4289d31dd31d2ececf62d94663c26cecafe84fe3155a18f901a41fbb303` |
| `expected_artifact` | `0e5497626bc05404caee48557417cc83e4c587515a077961fdedc6858a43c743` |
