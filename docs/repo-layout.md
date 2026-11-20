# Repository Layout

This file is the short map of what each top-level directory is for.

Use it when you already know the project roughly, but need to re-orient
quickly.

The [dated architecture and storage audit](repo-architecture-audit.md) records
the actual normal/build dependency graph, enforced domain boundaries and remaining
extraction debt. In particular, `tools/nuisc`, `tools/nsld` and `tools/nsdb` contain
reusable libraries as well as frontdoors; directory names alone are not dependency
boundaries.

The [repository README](../README.md), [documentation index](README.md) and
[current mainline map](current-mainline-map.md) are short entry routers.
Detailed native execution receipts are catalogued once in the
[native session proof index](reference/nuis-native-session-proof-index.md);
minor identities and historical acceptance belong in the
[versioning index](versioning/README.md), not repeated entrypoint changelogs.

## Current Mainline Directories

* [tools](../tools)
  executable front doors and compiler/tool binaries such as `nuis`, `nuisc`,
  `yir-run`, and packaging/export helpers
* [crates](../crates)
  reusable Rust implementation crates for semantics, verifier logic, runtime
  support, shader/domain helpers, and related internals
* [nustar-packages](../nustar-packages)
  checked-in `nustar` manifests and package registration metadata, including
  ABI targets and lane defaults
* [examples](../examples)
  canonical source examples, invalid/verifier examples, historical bridge
  examples, and a source-first bundle rebuild guide
* [stdlib](../stdlib)
  standard-library layout plus staged `.ns` source assets, especially the
  `std`, `PixelMagic`, `WitSage`, and `ns-nova` official library/galaxy
  surfaces
* [subprojects](../subprojects)
  sibling ecosystem projects hosted in-tree for now, currently `vulpoya` and
  `yalivia`, with boundaries kept explicit rather than merged into `nuisc`
* [tests](../tests)
  repository-level fixtures and cross-toolchain integration support
* [scripts](../scripts)
  portable repository maintenance and developer-machine helpers
* [docs](./)
  current reference docs, grammar/front-end notes, design notes, and historical
  archive material

## Current Mainline Vs Experimental Surfaces

Use this as the shortest repo-level split:

* mainline today
  - [tools](../tools)
  - [crates](../crates)
  - [nustar-packages](../nustar-packages)
  - [examples/projects](../examples/projects)
  - [examples/ns](../examples/ns)
  - [stdlib/core](../stdlib/core)
  - [stdlib/std](../stdlib/std)
  - [stdlib/pixelmagic](../stdlib/pixelmagic)
  - [stdlib/witsage](../stdlib/witsage)
  - [stdlib/ns-nova](../stdlib/ns-nova)
  - [docs/reference](reference)
* experimental or softer-edged today
  - [examples/yir](../examples/yir)
    when used as handwritten probes rather than current front-door behavior
  - [docs/fabric-spec](fabric-spec)
  - [docs/glm-spec](glm-spec)
  - [docs/yir-spec](yir-spec)
  - future-sketch notes scattered under
    [examples/projects](../examples/projects)
    and
    [examples/ns/ffi](../examples/ns/ffi)

This is not a value judgment. It is just the practical rule for deciding what
to trust first when implementation and future direction are both present.

## Support / Secondary Directories

* [nuis-logo](../nuis-logo)
  branding assets
* `target/`
  local build outputs and scratch artifacts; not part of the curated source
  layout and absent from a clean checkout
* [examples/bins](../examples/bins)
  rebuild guide only; generated example bundles belong in ignored
  `target/example-builds/`, not in the source checkout
* [.github](../.github)
  repository automation/workflow metadata

## Native Resource Session Ownership

The current application prerequisite crosses existing package boundaries; it is
not a new engine-specific compiler module. Change the narrow owner below rather
than importing a tool frontdoor into a shared contract crate.

| Responsibility | Implementation Entry |
| --- | --- |
| Registration, State layouts and versioned owner trailers | [yir-core session contracts](../crates/yir-core/src/native_resource_session.rs), [call contract](../crates/yir-core/src/guarded_owned_call.rs) |
| Registered lexical identities and post-semantic canonical type admission | [resource scalar bindings](../tools/nuisc/src/resource_session_bindings.rs), [resolved type gate](../tools/nuisc/src/resource_session_bindings/types.rs) |
| Source branch outlining, shared continuations and captures | [nuisc outliner](../tools/nuisc/src/lowering/guarded_owned_returns.rs), [branch pairs](../tools/nuisc/src/lowering/guarded_owned_returns/branches.rs), [private continuation builder](../tools/nuisc/src/lowering/guarded_owned_returns/continuations.rs) |
| Typed source function result boundary | [direct-call return boundary](../tools/nuisc/src/lowering/direct_calls/return_boundary.rs) |
| Domain operation description and diagnostic behavior | [CPU control operations](../crates/yir-domain-cpu/src/describe_post_control.rs) |
| Independent native admission and LLVM emission | [native resource lowering](../crates/yir-lower-llvm/src/native_resource_session.rs), [call emission](../crates/yir-lower-llvm/src/guarded_owned_call.rs) |
| Pure scalar/record-helper admission and private value transport | [resource adapter](../crates/yir-lower-llvm/src/native_resource_session/admission/value_helpers.rs), [shared helper validator](../crates/yir-lower-llvm/src/native_session/value_helpers.rs), [value return transport](../crates/yir-lower-llvm/src/native_session/aggregate_values.rs) |
| Short-circuit fallback safety and exact native predicates | [core expressions](../tools/nuisc/src/lowering/core_exprs.rs), [strict native values](../crates/yir-lower-llvm/src/function_lowering/strict_scalar.rs) |
| Direct negation and bounded guarded predicate normalization | [predicate lowering](../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_logical.rs), [source-shape budgets](../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_prefix.rs) |
| Selected Buffer/Bytes reads in nested bool declarations | [owned-region normalization](../tools/nuisc/src/lowering/guarded_owned_returns/resource_predicates.rs), [native read regressions](../tools/nuisc/tests/native_resource_session/resource_predicates.rs), [nested graph and ownership regressions](../tools/nuisc/tests/native_resource_session/nested_resource_predicates.rs) |
| Selected reads composed with pure helper arguments | [shared completed-call proof](../tools/nuisc/src/lowering/buffer_loop_outline/pure_value_calls.rs), [native helper/read regressions](../tools/nuisc/tests/native_resource_session/resource_helper_predicates.rs) |
| Ordered logical arguments and comparison/XOR operands in resource predicates | [scalar operand snapshots](../tools/nuisc/src/lowering/guarded_owned_returns/resource_predicate_arguments.rs), [native argument regressions](../tools/nuisc/tests/native_resource_session/resource_argument_predicates.rs), [wrapper regressions](../tools/nuisc/tests/native_resource_session/resource_wrapped_predicates.rs) |
| Direct i64 initializers with guarded resource arguments | [exact-kind source proofs](../tools/nuisc/src/lowering/guarded_owned_returns/resource_scalar_initializers_tests.rs), [native execution and trap cases](../tools/nuisc/tests/native_resource_session/resource_i64_initializers.rs) |
| Direct resource-bearing bool conditions | [condition normalization proofs](../tools/nuisc/src/lowering/guarded_owned_returns/resource_condition_tests.rs), [native branch/owner regressions](../tools/nuisc/tests/native_resource_session/resource_conditions.rs) |
| Returned mixed-State fields and terminal cleanup bindings | [layout-driven field normalization](../tools/nuisc/src/lowering/guarded_owned_returns/resource_return_fields.rs), [source proofs](../tools/nuisc/src/lowering/guarded_owned_returns/resource_return_fields_tests.rs), [native order and cleanup regressions](../tools/nuisc/tests/native_resource_session/resource_return_fields.rs) |
| Direct checked Err codes | The same layout-driven return normalization; [native error code and failure regressions](../tools/nuisc/tests/native_resource_session/resource_error_codes.rs) also cover borrowed-only owned continuations, which are excluded from speculative pure-helper routing. |
| Borrowed Buffer length in guarded scalar expressions | Shared `ref Buffer` proof in owned-region normalization; [native length composition and failure regressions](../tools/nuisc/tests/native_resource_session/resource_buffer_lengths.rs) cover declarations, conditions, helper arguments and error returns without widening the pure-helper ABI. |
| Helper-returned value-record locals with lazy scalar arguments | The same owned-region declaration staging uses the shared pure-call/layout proof; [record initializer source proofs](../tools/nuisc/src/lowering/guarded_owned_returns/resource_record_initializers_tests.rs) and [native local-record regressions](../tools/nuisc/tests/native_resource_session/resource_record_initializers.rs) retain nominal types, argument order and the independent native value ABI. |
| Nested literal locals with lazy scalar fields | [Constructor normalization](../tools/nuisc/src/lowering/guarded_owned_returns/resource_record_literals.rs) stages typed leaves in source order after complete nominal proof; [native literal regressions](../tools/nuisc/tests/native_resource_session/resource_record_literals.rs) cover execution and cleanup. Native admission shares pure-value bounds without widening owned State. |
| Record-valued helper fields in literal locals | Constructor normalization materializes each helper once and projects its scalar leaves before later fields; [field-value source proofs](../tools/nuisc/src/lowering/guarded_owned_returns/resource_record_field_values_tests.rs) and [native field-value regressions](../tools/nuisc/tests/native_resource_session/resource_record_field_values.rs) cover nominal typing, bounded expansion, once-only calls and cleanup. |
| Previously bound record fields before later resource logic | A separate block-local type table proves nominal availability without record captures; [scope proofs](../tools/nuisc/src/lowering/guarded_owned_returns/resource_record_local_fields_tests.rs) and [native local-field regressions](../tools/nuisc/tests/native_resource_session/resource_record_local_fields.rs) cover invalidation, nested projection, completed reads and cleanup. |
| Previously bound record fields after earlier resource logic | The [liveness planner](../tools/nuisc/src/lowering/guarded_owned_returns/resource_record_liveness.rs) saves only completed record projections and shares repeated paths; [source proofs](../tools/nuisc/src/lowering/guarded_owned_returns/resource_record_liveness_tests.rs) and [native regressions](../tools/nuisc/tests/native_resource_session/resource_record_liveness.rs) retain evaluation order, capture bounds and cleanup. |
| Local-record scalar operands inside resource constructor predicates | [Operand proof](../tools/nuisc/src/lowering/guarded_owned_returns/resource_record_operands.rs) checks exact scalar witnesses after bounding the original tree; the existing planner shares completed projections without hoisting reads or calls. [Source proofs](../tools/nuisc/src/lowering/guarded_owned_returns/resource_record_operands_tests.rs) and [native regressions](../tools/nuisc/tests/native_resource_session/resource_record_operands.rs) retain types, ordering, ownership and bounds. |
| Standalone bool/i64 initializers using local-record fields | The same operand proof requeues the original declaration after shared scalar projections, without whole-record captures. [Declaration source proofs](../tools/nuisc/src/lowering/guarded_owned_returns/resource_record_scalar_initializers_tests.rs) and [native initializer regressions](../tools/nuisc/tests/native_resource_session/resource_record_scalar_initializers.rs) cover scope, types, one-time rewriting, State and owner cleanup. |
| Direct resource conditions using local-record fields | The same operand proof requeues the original branch after exact scalar projections. [Condition source proofs](../tools/nuisc/src/lowering/guarded_owned_returns/resource_record_condition_tests.rs) retain arms and suffix; [native condition regressions](../tools/nuisc/tests/native_resource_session/resource_record_conditions.rs) cover paired-arm ownership, failure cleanup, registered error identity and unchanged capture limits. |
| Ready scalar selection in owned callbacks | [independent native admission](../crates/yir-lower-llvm/src/native_resource_session/admission.rs), [exact-type and native selection regressions](../tools/nuisc/tests/native_resource_session/scalar_selection.rs) |
| Nested pure projections without resource aliases | [mixed admission](../crates/yir-lower-llvm/src/native_resource_session/mixed_admission.rs), [nested native regressions](../tools/nuisc/tests/native_resource_session/nested_records.rs) |
| Host State transport, publication and teardown | [runtime session](../crates/yir-runtime-host/src/native_resource_session.rs) |
| Project identity and artifact assembly | [project session registration](../tools/nuisc/src/project/application_sessions.rs), [pipeline boundary](../tools/nuisc/src/pipeline_resource_session.rs), [AOT boundary](../tools/nuisc/src/aot_resource_session.rs) |
| Executable launch adapters | [nuis artifact adapter](../tools/nuis/src/artifact_runtime_native_session.rs), [packer adapter](../tools/yir-pack-aot/src/host_resource_session.rs) |
| Semantic/native ownership regressions | [compiler resource suite](../tools/nuisc/tests/native_resource_session.rs) |
| Cache, NSB relocation and binary-only execution | [artifact workflow suite](../tools/nuis/tests/native_resource_session_workflow.rs) |

Start execution from the [resource example ladder](../examples/projects/domains/README.md#native-cpu-resource-sessions).
Use the [proof index](reference/nuis-native-session-proof-index.md#resource-session-ladder)
for scoped evidence and the [mainline map](current-mainline-map.md) for the next
task. These native CPU callbacks remain separate from embedded-YIR window and
provider/device sessions; directory proximity does not merge their admission.

## Reading Order By Goal

If you want the current user-facing path:

* [README.md](../README.md)
* [examples/projects/README.md](../examples/projects/README.md)
* [docs/reference/README.md](reference/README.md)

If you want the current practical systems/library path:

* [stdlib/README.md](../stdlib/README.md)
* [stdlib/std/README.md](../stdlib/std/README.md)
* [examples/ns/ffi/README.md](../examples/ns/ffi/README.md)
* [examples/projects/README.md](../examples/projects/README.md)

If you want implementation internals:

* [tools](../tools)
* [crates](../crates)
* [nustar-packages](../nustar-packages)

If you want framework/library evolution:

* [stdlib/README.md](../stdlib/README.md)
* [stdlib/pixelmagic/README.md](../stdlib/pixelmagic/README.md)
* [stdlib/witsage/README.md](../stdlib/witsage/README.md)
* [stdlib/ns-nova/README.md](../stdlib/ns-nova/README.md)

If you want older design background:

* [docs/historical/README.md](historical/README.md)
