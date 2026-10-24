# Native Record Prefix Joins V1

This bounded CPU follow-up continues the [nested scalar checkpoint](nuis-native-nested-prefix-joins-v1.md)
on Git beta-0.17.6 at 1feb6761 without changing the version.

## Record Prefix Contract

The [typed-plan proof](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_nested_prefix.rs)
now has a separate record-result entry. Only a local conditional inside a pure
prefix may export a supported owned record. Both paths must end in the same
explicit type and local name. Equal-looking layouts with different record names
do not coerce. The enclosing declaration still has an observed scalar result.
The preceding scalar-only entry remains unchanged in authority.

The existing [constructor](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_nested_prefix.rs)
consumes original types and reuses the shared extractor, logical normalizer,
guard selection and `Selection::with_prefix` assembly. It preserves source field
construction order, not sorted layout order. Only the joined record escapes its
child scope; branch-local consts do not become outer captures. No engine case,
backend-specific path, new public ABI or relaxed graph limit is introduced.

All original conditions, local values and record fields share the existing
4,096-work/64-depth/32-logical-edge budget, 128 descendant statements, 32 statements
per block and eight nested-selection levels. The syntax pass runs before local
scope/capture/helper allocation. These limits apply to the unrewritten NIR after
earlier simplification, not to the pre-simplification source AST. Existing value
layout bounds and completed pure-dependency checks remain in force.

This entry requires complete record paths. A pruned one-sided record result is
not given a synthetic aggregate return. The earlier discarded scalar/condition
rules remain available independently. References/resources, recursive layouts,
partial or mismatched records, effects, outer writes, hidden logical field roots,
shadowing, self/forward reads and leaked child names remain outside this proof.

## Executable Witness

The [Nuis application](../../tools/nuisc/tests/native_application_bridge/record_prefix_join.ns)
extends the effectful-record/direct-field/loop-exit route. Each outer bool arm
selects a `Choice` with signed word, checksum, bool tag and an unprojected fallible
field. Child paths construct fields in different orders, with fallible calls
before and after the unprojected field. The joined record feeds a pure scalar
helper, dependent prefix work and the final short-circuit value. A reached final
comparison pins the signed-word/checksum/tag-derived result, including MIN/MAX
words, rather than merely accepting any nonzero seed. A later unused `Packet`
field still executes before the enclosing returned field is published.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/record_prefix_join_cases.rs)
uses wrapping arithmetic and checked division, separately modelling true/false
field construction order. Healthy sessions cover both parent/child paths, mixed
bool/word transport, MIN/MAX checksums, skipped poison and live State dependencies.
Thirty-two core failures include the preceding twenty-four and eight reached
record-field faults. Four extra ordering probes give thirty-six native/reference failures.
No source effects are added to pure helpers: test-only markers 83-89 and 94-98
observe twelve original callees in native code.

[Native/reference tests](../../tools/nuisc/tests/native_application_bridge/record_prefix_join.rs)
exercise normal/reversed YIR, once-only calls and exact flushed failure prefixes.
Failure preserves the accepted State sequence 64 -> 72 -> -2, prevents later events
and leaves completion faulted. The compact profile stays at 4 work, 64 entries
and at most 64 functions.

[Compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_record_prefix_tests.rs)
test dependency/effect revocation, atomic parent rejection, nominal record layout,
source field order, private scopes, order independence, idempotence and nested
record layouts. Budget controls cover 128/129 statements, 32/33 logical edges
and joint field-expression exhaustion. Full source compilation/reference execution
and LLVM emission retain failures in unprojected fields.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/record_prefix_join.rs)
checks fresh/cache and two independent source-free images. Each image runs healthy
sessions before and after all thirty-two core failures. Binary, host-object,
envelope, lifecycle, packaging and original artifact identities must remain fixed.
Trapping children need not flush stdout: native probes prove complete prefixes,
while CLI checks pin State, trap status and any flushed prefix. The four extra
native ordering cases are not claimed as CLI cases.

## Validation

Bounded local M2 CPU validation passed 163 distinct tests, including eight new
tests. Repeated runs, including the initial pre-comparison witness and the
intermediate helper-count assertion correction, are not counted again:

- Compiler: 68 focused/adjacent checks in 15.25 seconds, including five new
  record-prefix controls and preceding scalar, prefix and nested regressions.
- Native/reference: 19 checks in 26.81 seconds, including two new witnesses,
  the preceding join families and full-fixture bounds controls. The new witness
  exercises thirty-six failures in both normal and reversed YIR order.
- CLI delivery: two checks in 375.20 seconds. Record and nested-scalar families
  each pass fresh/cache and two independent source-free images, with healthy
  sessions before and after respectively thirty-two and twenty-four core failures.
- Borrowed-buffer integration: 17 checks in 38.34 seconds; host failure state:
  three; repository maintenance: 24; host absolute-path policy: one; file-line
  limits: two; status tensor: 27 in 5.27 seconds.

The ordinary CLI build was already current and passed in 0.02 seconds after
delivery tests finished. Formatting, whitespace, UTF-8 across 5,693 files,
4,275 local documentation links and locked offline metadata for 29 packages
passed. Drift matches all 3,663 checks and 25,009 required patterns. The CLI
matches all six source fields, complete predecessor hashes and inert historical
commands. Coverage, hierarchy and lineage are clean; the next task card is ready
and the coordinate remains active/99.

These are targeted local CPU checks, not a full workspace run. The earlier
four-variant independent/shared matrix and 31-failure delivery, non-nested CLI
families, other historical delivery families, GPU/Linux execution and physical
performance were not rerun for this checkpoint. Their receipts remain historical
evidence, not new record-prefix results.

## Remaining Frontier

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. Next, prove discarded record-valued local work after dead-binding
pruning without exporting an erased identity or constructing unbounded neutral
layouts. Complete two-sided record results are covered here; partially erased
aggregate paths are not. Effectful prefixes, outer writes, resources/I/O/async
captures, stable ABI, full GPU/Linux parity and self-hosting remain separate gates.

The [discarded-record follow-up](nuis-native-discarded-record-prefix-v1.md) adds
the separate pruned-path proof. This checkpoint's complete-path results stay historical.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_values::record_prefix_tests --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- record_prefix_join:: nested_prefix_join:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- record_prefix_join nested_prefix_join --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

These are focused entry points; the tensor retains the full adjacent chain.
Complete predecessor commands remain inert history, not commands to execute.

## Predecessor Lineage

All six complete nested-scalar fields are preserved by SHA-256, with quoted inert
historical commands. Historical results are not counted as rerun tests.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `5a0f456c264c1dfe9675c6b29d2c1ee05f5dc612d3110bcfb5c00be6eb2c5fa4` |
| `next_step` | `e8a04eb1f60922786379096ce2b7068da59db96a381a8207c28f0fac3bcfe9a7` |
| `blocker` | `93ae82a2083b875b92d30f81188ccfe3f3b2c1de8800f84707ba08cd8d7fb31b` |
| `next_action` | `fd52526a68fd98652966d5dab5720aa45644baea72cf105d958fdf6c7e59092e` |
| `validation_command` | `bfaa8fed0da91791e50a2b38dab2afbb375204647f31eb1e2dcefa75c4cc5541` |
| `expected_artifact` | `e9d7baddbd57f45aa3229bb73f72a296d25af7fdc7bb0a9b26f73ab1623e97cd` |
