# Native Nested Prefix Joins V1

This bounded CPU follow-up continues the [flat-prefix checkpoint](nuis-native-prefixed-scalar-joins-v1.md)
on Git beta-0.17.6 at 1feb6761 without changing the version.

## Nested Prefix Contract

The independent [nested proof](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_nested_prefix.rs)
builds a typed plan for a complete original declaration before helper allocation.
The budget applies to the unrewritten NIR presented to this admission pass,
after existing simplification, not to the pre-simplification source AST.
A nested local selection exports only its same-name, same-explicit-scalar result;
all other branch locals remain private. Each scope forbids outer-name shadowing,
duplicate bindings, forward/self reads and early binding of ancestor results.
Prefix bindings may hold supported records, but conditional results remain scalar.

All predicates, initializers and final values share one 4,096-work/64-depth/
32-logical-edge expression budget. Its descendant blocks share 128 statements
(excluding the outer declaration root), at most 32 statements per block and
eight nested local-selection levels.
Structural limits are checked iteratively before branch-local proof scopes,
capture collection or recursive type inference. Completed pure dependencies
prove all original expressions. Effects, resources/references, outer writes,
hidden logical arguments/comparisons and mismatched/partial exported joins
remain excluded. The preceding flat-prefix proof is unchanged.

The [nested constructor](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_nested_prefix.rs)
consumes that plan, keeping explicit original types when earlier local joins
become generated calls absent from the original catalog. It reuses the shared
conditional extractor, logical normalizer, guard selection and
`Selection::with_prefix` assembly; it does not infer generated calls against
an incomplete catalog. Child helpers remain inside the selected parent arm,
and every predicate stays argument zero of its own value helper.

## Unused Conditional Work

Early dead-binding pruning can erase a total side of an unused local result,
or erase both result bindings while retaining a fallible condition. The plan
handles these only as discarded work: a synthetic zero return on an erased
side never exports a source name, and a condition-only private binding retains
the original evaluation and short-circuit behavior. Later reads of an erased
identity are rejected. This does not change global optimization rules or permit
partial source results to escape. Unused fallible local selections still run.

## Executable Witness

The [Nuis application](../../tools/nuisc/tests/native_application_bridge/nested_prefix_join.ns)
extends the effectful-record/direct-field/loop-exit route with a typed `seed`
selection in each outer arm. A fallible nested predicate chooses a seed or
alternate-seed callee, each with a private const local. The joined seed feeds
the dependent step, head and unused checked initializer before the final
short-circuit value. The later unused record-field check remains intact.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/nested_prefix_join_cases.rs)
retains wrapping addition and checked division. Healthy rows cover both nested
paths, computed nested-choice inversion, skipped poison and live-State dependency.
Twenty-four core failures retain the preceding sixteen and add both nested
predicate failures plus six alternate-prefix failures. Native/reference checks
also include four reached-RHS ordering probes, for twenty-eight failures total.

[Native/reference tests](../../tools/nuisc/tests/native_application_bridge/nested_prefix_join.rs)
run normal/reversed YIR. Test-only markers 83-89 and 94-95 observe nine original
callees without adding source effects. Exact flushed traces pin parent predicate,
nested predicate, selected child and later prefix order. Failure retains State -2,
rejects later events and leaves completion faulted. The compact profile remains
4 work, 64 entries and the existing 64-function bound; no graph cap is relaxed.

An [unused-result application](../../tools/nuisc/tests/native_application_bridge/nested_prefix_discarded.ns)
also runs directly as native code in two forms: a retained fallible arm with an
erased total side, and a fallible condition whose two results were erased.
Both forms retain State 73 -> 80 -> 87, skip poison and trap before the following
print on a selected zero divisor. These are native controls, not CLI deliveries.

[Compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_nested_prefix_tests.rs)
exercise direct admission separately from the general child-normalization walk,
so failed parents cannot borrow independent successful child rewrites. They pin
atomic helper/name/scope rejection, child-local non-export, completed dependency/
effect revocation, order independence and idempotence. Positive/negative boundaries
cover 128/129 descendant statements, eight/nine levels, joint logical edges and expression
work. Source execution also covers all scalar result types, unused fallible local
results, fallible conditions whose results were erased, and a logical initializer
feeding a later local selection.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/nested_prefix_join.rs)
runs fresh/cache and two independent source-free images, with healthy sessions
before and after twenty-four core failures per image. Binary, host-object,
envelope, lifecycle, packaging and original artifact identities remain fixed.
Trapping children need not flush stdout: the native probe proves the complete
prefix, while CLI checks pin State, trap status and any flushed prefix.
The four extra native ordering probes are not claimed as CLI delivery cases.

## Validation

Bounded local M2 CPU validation passed 156 distinct tests, including ten new
tests. Repeated invocations are counted once:

- Compiler: 63 focused/adjacent checks in 12.77 seconds, including six new nested
  controls and the preceding declaration, prefix and logical-root regressions.
- Native/reference: 17 checks in 20.54 seconds, including three new witnesses,
  five preceding declaration families and four full-fixture bounds controls.
  The main witness covers twenty-eight failures. The extra native test executes
  both optimized unused-result forms; the earlier smaller batch is not counted
  again.
- CLI delivery: two checks in 243.93 seconds. Nested and flat-prefix families
  each pass fresh/cache and two independent source-free images, with healthy
  sessions before and after respectively twenty-four and sixteen core failures.
  The additional native ordering and unused-result controls are not claimed as
  CLI delivery cases.
- Borrowed-buffer integration: 17 checks in 49.41 seconds; host failure state:
  three; repository maintenance: 24; host absolute-path policy: one; file-line
  limits: two; status tensor: 27 in 5.38 seconds.

The ordinary CLI rebuild passed in 7.60 seconds. Formatting, whitespace, UTF-8
across 5,686 files, 4,261 local documentation links and locked offline metadata
for 29 packages passed. Drift matches all 3,654 checks and 24,956 required
patterns. The rebuilt CLI matches all six source fields, complete predecessor
hashes and inert historical commands. Coverage, hierarchy and lineage are clean;
the next task card is ready and the coordinate remains active/99.

These are targeted local CPU checks, not a full workspace run. The earlier
four-variant independent/shared matrix and 31-failure delivery, the earlier
non-prefix CLI delivery families, other historical delivery families, GPU/Linux
execution and physical performance were not rerun for this checkpoint. Their
receipts remain historical evidence, not new nested-prefix results.

## Remaining Frontier

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. Record-valued local conditional results inside a prefix need a
separate proof for exact branch layouts, original scopes and native field words.
Effectful prefixes, outer writes, resources/I/O/async captures, stable ABI,
full GPU/Linux parity and self-hosting remain separate gates.

The [record-prefix follow-up](nuis-native-record-prefix-joins-v1.md) supplies the
separate complete-record proof. This checkpoint's scalar-only results stay historical.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_values::nested_prefix_tests --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- nested_prefix_join:: prefixed_scalar_join:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- nested_prefix_join prefixed_scalar_join --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

These are focused entry points. The coordinate's `validation_command` field
retains the complete focused/adjacent command chain; predecessor commands remain
inert history, not additional commands to execute.

## Predecessor Lineage

All six complete flat-prefix fields are preserved by SHA-256, with quoted inert
historical commands. Historical results are not counted as rerun tests.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `474c311eef3fb8ad45ed7c559616d5f867216d147d4707e1e18ed666e41d4845` |
| `next_step` | `fdfca8beae93c07bd9d90ebbb96dc1da09bf618bd7de8af18f5b7c0579fb657a` |
| `blocker` | `20eba72e959d28092689a22bcaeee09f11ced5fecf76ee4c701a8af3a64a9a97` |
| `next_action` | `8d86ce033790102a5bd25476847e0f4ba91b828326a7e25d6ad14246a9761b33` |
| `validation_command` | `e725c0c89fcd006d8c3043339a50356832a09bdd27788a49eb61f7dac50fb204` |
| `expected_artifact` | `05b5c76e40b67dfef2e45c1ac316a5fc49a9466faef010ce0f67108cbe3fecb8` |
