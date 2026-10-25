# Native Logical Record Fields V1

This bounded CPU follow-up continues the [outer-record checkpoint](nuis-native-outer-record-prefix-v1.md)
on Git beta-0.17.7 at 7eb90ed4 without changing the version.

## Typed Field Root Contract

The [typed-plan entry](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_nested_prefix.rs)
now independently admits inline `&&` / `||` roots in explicitly bool-typed fields
of owned record literals. It requires an actual logical field in the plan and
keeps the enclosing fresh observed result an exact nominal record type. Older
outer-record/scalar entries retain their preceding authority.

The [shared syntax traversal](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_prefix.rs)
uses layout metadata to grant permission only to a bool field root, recursively
through directly nested record literals. Ordinary call arguments, comparison
operands, casts and projection bases reset both logical and record-root permission.
A record literal embedded in those ordinary leaves cannot smuggle in gates.
The old traversal supplies no field permissions at all; there is no global
relaxation of ordinary-leaf syntax.

Conditions, local initializers and every field retain one original 4,096-work/
64-depth/32-logical-edge budget, 128 descendant statements, 32 statements per
block and eight nested-selection levels. Complete original scopes, layouts,
dependencies and types are proved before generated names/helpers. Discarded
records keep the existing cumulative 4,096-node/depth-below-64 neutral preflight.
Budgets apply to unrewritten input NIR after prior simplification, not the source
AST. Effects, outer writes, resources/references and mismatched/partial results
remain excluded by this entry.

## Source-Ordered Construction

The [field constructor](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_record_fields.rs)
rewrites each bool gate at its original field, not into a statement ahead of
earlier record fields. It also handles nested literals and record-valued local
initializers using their original scopes. No generated call is re-inferred
against the old dependency catalog.

It reuses the [logical-tree constructor](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_logical.rs)
with a field-only forced-edge mode. Every original field gate becomes a guarded
helper, including gates with total right operands, so downstream ordinary record
leaves contain no remaining logical nodes. The old root mode retains its previous
speculation decision. Each original guarded left operand runs once as argument
zero and its complete right subtree remains selected. Shared extractor, guard
selection and prefix assembly remain in use; no engine/backend case, foreign
source shim, public ABI or relaxed native limit is added.

## Executable Witness

The [Nuis application](../../tools/nuisc/tests/native_application_bridge/logical_record_fields.ns)
composes bool field gates with outer/nested/discarded records, an effectful record
return, direct field projection and loop early exit. One outer arm constructs
fallible word fields before its logical tag; the other evaluates its logical tag
before the word fields. A second logical field is genuinely unprojected, yet its
selected failure must still happen. That spare field also changes position across
the two source paths. Exact tags/checksums and MIN/MAX word transport remain checked.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/logical_record_fields_cases.rs)
models wrapping arithmetic, checked division, source field order and both bool
outcomes independently of compiler output. Healthy sessions cover reached and
skipped RHS calls, skipped poison, unused bool results and live State.
Twenty-two core failures include the preceding sixteen plus left/RHS and unused
bool-field failures. Four additional cross-poison ordering probes give
twenty-six native/reference failures. Test-only markers 94 and 96-103 observe
nine original pure callees without adding source effects.

[Native/reference tests](../../tools/nuisc/tests/native_application_bridge/logical_record_fields.rs)
run normal/reversed YIR, exact flushed failure prefixes, once-only reached calls
and canary-guarded native outputs. Failure preserves the accepted State sequence
64 -> 72 -> -2, prevents later events and leaves completion faulted. The compact
profile remains 4 work, 64 entries and at most 64 functions, without owned-aggregate
heap allocation/drop in the native bridge.

[Compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_logical_field_tests.rs)
pin independent authority, helper counts for every original field edge, removal
of all field logic including total RHS gates, original child scopes, idempotence,
function-order independence and atomic dependency/effect revocation. They reject
type-correct hidden arguments/comparisons/projections, wrong field types and erased
names. Shared-budget controls cover 128/129 statements, 32/33 combined logical edges
and cross-field expression exhaustion. Nested local-record fields also retain
unprojected failures in full source/reference execution and LLVM emission.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/logical_record_fields.rs)
checks fresh/cache and two independent source-free images. Each image runs healthy
sessions before and after all twenty-two core failures. Binary, host-object,
envelope, lifecycle, packaging and original artifact identities stay fixed.
Trapping children need not flush stdout: native probes pin full prefixes, while
CLI checks pin State, trap status and any flushed prefix. The four extra native
cross-poison probes are not claimed as CLI delivery cases.

## Validation

Bounded local M2 CPU validation passed 192 distinct tests, including eight new
tests. Repeated focused runs and the corrected erased-name negative fixture are
not counted twice:

- Compiler: 92 focused/adjacent checks in 51.19 seconds, including five new field
  controls, preceding declaration families and eight additional existing logical-tree
  budget/type/native controls for the shared traversal/constructor refactor.
- Native/reference: 25 checks in 35.48 seconds, including two new witnesses,
  preceding join families and full-fixture bounds controls. The new witness runs
  twenty-six failures in both normal and reversed YIR order.
- CLI delivery: one new check in 176.40 seconds. Fresh/cache and two independent
  source-free images each run healthy sessions before and after twenty-two core failures.
- Borrowed-buffer integration: 17 checks in 41.58 seconds; host failure state:
  three; repository maintenance: 24; host absolute-path policy: one; file-line
  limits: two; status tensor: 27 in 5.98 seconds.

The ordinary CLI build was already current and passed in 0.04 seconds after
delivery finished. Formatting, whitespace, UTF-8 across 5,718 files, 4,322 local
documentation links and locked offline metadata for 29 packages passed. Drift
matches all 3,697 checks and 25,180 required patterns. All six source fields match
the CLI, complete predecessor hashes are verified and historical commands remain
inert. Coverage, hierarchy and lineage are clean; the next task card is ready and
the coordinate remains active/99.

These are targeted local CPU checks, not a full workspace run. Earlier CLI
delivery families, the four-variant independent/shared matrix and 31-failure
delivery, GPU/Linux execution and physical float/performance tests were not rerun
here. Their receipts remain historical evidence, not new logical-field results.

## Remaining Frontier

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. Direct record-valued function returns with inline bool fields need
their own return-boundary proof; this checkpoint owns fresh observed declarations.
Ordinary argument/comparison embedding, effectful prefixes, outer writes,
resource/I/O/async captures, stable ABI, full GPU/Linux parity and self-hosting
remain separate gates.

The [direct record-return follow-up](nuis-native-direct-record-return-v1.md) adds
the separate return-boundary proof. This checkpoint's declaration results stay historical.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_values::logical_field_tests conditional_logical_trees --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- logical_record_fields:: outer_record_prefix:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- logical_record_fields --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

These are focused entry points; the tensor retains the full adjacent chain.
Complete predecessor commands stay inert history, not commands to execute.

## Predecessor Lineage

All six complete outer-record fields are preserved by SHA-256, with quoted inert
historical commands. Historical results are not counted as rerun tests.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `9a82cabd1a3dcd0c76950b073e05f051e3d9bf72ba6b90e2d91852e38d912e07` |
| `next_step` | `4bcaea1870d91f0a38b3d8a18810884bfcdea4e01bf826c7f3e1b0528e79235c` |
| `blocker` | `cba8e6035d0c62f542e33e680fbb9c44b5964469db113abb301206cb79d9f616` |
| `next_action` | `ecaef85cf1cef51ff3b7c3074aab2e93a54d50d70de18bc874b550147ca3e671` |
| `validation_command` | `157edd7402b60956c18bb2256a8848ad6fcfa48e5d7fa62df6ba02954ae0eeb7` |
| `expected_artifact` | `39e59a31540823026bfe1b0a60afbba4fa9f7b85db09bef261d8e2d4940dfcd8` |
