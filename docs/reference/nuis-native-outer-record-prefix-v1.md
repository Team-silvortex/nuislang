# Native Outer Record Prefix V1

This bounded CPU follow-up continues the [discarded-record checkpoint](nuis-native-discarded-record-prefix-v1.md)
on Git beta-0.17.7 at 7eb90ed4 without changing the version.

## Outer Record Contract

The [typed-plan proof](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_nested_prefix.rs)
adds an independent outer-record entry. The observed fresh declaration may now
return a supported owned record, with exactly the same explicit type and result
name on both paths. Structurally similar but differently named types do not
coerce. This is a new local value, not permission to overwrite an outer carry.
Old scalar-result entry points continue requiring a scalar outer result.

The new entry supports flat or nested pure prefixes, including an empty prefix,
while retaining original-scope proof, completed pure dependencies and private
child names. Complete/discarded local joins continue using their existing rules.
Outer result paths must both exist; discarded child handling cannot synthesize
an absent outer result or export an erased child identity.

The [shared constructor](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_nested_prefix.rs)
consumes the typed plan through the existing extractor, logical normalizer,
guarded selection and `Selection::with_prefix`. Original field construction
order is preserved, including nested records and unprojected fallible fields.
There is no engine/backend case, foreign source shim, public ABI or relaxed
native profile limit.

All original conditions, prefix work and fields retain one 4,096-work/64-depth/
32-logical-edge expression budget, 128 descendant statements, 32 statements per
block and eight nested-selection levels before scope/capture/helper allocation.
The new mode also runs the [neutral-layout preflight](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_discarded_records.rs)
before constructing discarded child returns: all record zeros share 4,096 nodes
per declaration, with depth below 64 and explicit conversion-node accounting.
Limits apply to unrewritten input NIR after earlier simplification, not the source
AST. Existing value-layout admission is not narrowed.

## Executable Witness

The [Nuis application](../../tools/nuisc/tests/native_application_bridge/outer_record_prefix.ns)
composes an outer `Record` declaration with discarded nested-record work inside
both arms, an effectful `Packet` return, direct field projection and loop early
exit. The outer record transports nested signed words, a bool tag and checksum.
A later helper checks exact tag/checksum values before exposing its word, so
MIN/MAX checksum transport is exercised rather than merely checking nonzero.

The two outer paths construct fields in different orders. A genuinely unused
nested field can fail before or after the desired word field. The original
discarded child calls retain their own opposite order, and a later unused
`Packet` field still fails before State publication. Test-only markers 94 and 96-100
observe six original pure callees without adding source effects.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/outer_record_prefix_cases.rs)
uses wrapping State arithmetic and checked division, with explicit source-ordered
child/outer field stages. Healthy sessions cover both parent paths, skipped poison,
condition inversion, live State and MIN/MAX words/checksums. Sixteen failure inputs
include the preceding twelve plus four outer-field failures across both orders.

[Native/reference tests](../../tools/nuisc/tests/native_application_bridge/outer_record_prefix.rs)
exercise normal/reversed YIR, exact flushed prefixes, once-only reached calls and
canary-guarded native output. Failure preserves the accepted State sequence
64 -> 72 -> -2, prevents later events and leaves completion faulted. The profile
stays at 4 work, 64 entries and at most 64 functions, without owned-aggregate
heap allocation/drop in the native bridge.

[Compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_outer_record_tests.rs)
pin independent authority, nominal layouts, field order, private scopes,
dependency/effect revocation, atomic rejection, order independence and idempotence.
Source controls cover 128/129 statements, 32/33 logical edges and joint expression
exhaustion. Large but globally supported records prove the outer mode retains the
whole-declaration neutral bound: two 2,048-node zeros pass; two 2,049-node zeros
fail before helper allocation. Flat/empty prefixes cover all five plain scalar
field kinds at typed-NIR admission/verification, not new physical float delivery.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/outer_record_prefix.rs)
checks fresh/cache and two independent source-free images. Each image runs healthy
sessions before and after all sixteen failures. Binary, host-object, envelope,
lifecycle, packaging and original artifact identities stay fixed. Trapping
children need not flush stdout: native probes prove full prefixes, while CLI
checks pin State, trap status and any flushed prefix.

## Validation

Bounded local M2 CPU validation passed 177 distinct tests, including eight new
tests. Repeated focused runs are counted once:

- Compiler: 79 focused/adjacent checks in 14.50 seconds, including five new
  outer-record controls and preceding scalar/local-record/discarded families.
- Native/reference: 23 checks in 34.63 seconds, including two new witnesses,
  previous join families and full-fixture bounds controls. The new witness
  exercises all sixteen failures in both normal and reversed YIR order.
- CLI delivery: one new check in 107.43 seconds. Fresh/cache and two independent
  source-free images each run healthy sessions before and after all sixteen failures.
- Borrowed-buffer integration: 17 checks in 41.35 seconds; host failure state:
  three; repository maintenance: 24; host absolute-path policy: one; file-line
  limits: two; status tensor: 27 in 5.88 seconds.

The ordinary CLI build was already current and passed in 0.06 seconds after
delivery finished. Formatting, whitespace, UTF-8 across 5,709 files, 4,306 local
documentation links and locked offline metadata for 29 packages passed. Drift
matches all 3,685 checks and 25,116 required patterns. All six source fields match
the CLI, complete predecessor hashes are verified and historical commands remain
inert. Coverage, hierarchy and lineage are clean; the next task card is ready and
the coordinate remains active/99.

These are targeted local CPU checks, not a full workspace run. Earlier CLI
delivery families, the four-variant independent/shared matrix and 31-failure
delivery, GPU/Linux execution and physical float/performance tests were not
rerun here. Their receipts remain historical evidence, not new outer-record results.

## Remaining Frontier

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. Bool record fields containing inline logical gates need a separate
lazy-field proof that preserves whole-record field order and shared budgets.
Logical arguments/comparisons, effectful prefixes, outer writes, resource/I/O/async
captures, stable ABI, full GPU/Linux parity and self-hosting remain separate gates.

The [logical-field follow-up](nuis-native-logical-record-fields-v1.md) adds the
separate typed field-root proof. This checkpoint's ordinary-leaf results stay historical.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_values::outer_record_tests --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- outer_record_prefix:: discarded_record_prefix:: record_prefix_join:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- outer_record_prefix --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

These are focused entry points; the tensor retains the full adjacent chain.
Complete predecessor commands remain inert history, not commands to execute.

## Predecessor Lineage

All six complete discarded-record fields are preserved by SHA-256, with quoted
inert historical commands. Historical results are not counted as rerun tests.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `981bbf6873c69534e0a902084d9056296238cfb1938f96a8b667827c7ce51784` |
| `next_step` | `eb0fe85636c7dc6792681dd8b69be6ad51260829e85b53d5588ab3d3a028fb3e` |
| `blocker` | `911ec06b5f36ecf49e0284ba4a25570c4d83d9a11a44ca2d49bf32b74219da95` |
| `next_action` | `aaa0adcc94942f7c22cb87d885091c4968765de4386d9a7513806f18cdd8bcea` |
| `validation_command` | `47fa60152e10e611ba541fe5c10205194f672891030cb54362c0c43a91ed5e50` |
| `expected_artifact` | `be4210b6a183858148cf7effffd588f7c841b30f507386b7768ec8f8ce39bbbf` |
