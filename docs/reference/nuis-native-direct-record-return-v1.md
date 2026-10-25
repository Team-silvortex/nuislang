# Native Direct Record Return V1

This bounded CPU follow-up continues the [logical-field checkpoint](nuis-native-logical-record-fields-v1.md)
on Git beta-0.17.7 at 7eb90ed4 without changing the version.

## Direct Return Contract

The [return plan](../../tools/nuisc/src/lowering/buffer_loop_outline/record_return_plan.rs)
proves a complete synchronous, nongeneric function before any generated helper
or body mutation. Its explicit result must be a supported owned record, all
return paths must agree with that exact nominal type, and at least one direct
record-literal return must contain a lazy bool field. An unrelated plain return
cannot borrow this entry's authority.

Supported preceding work consists of fresh pure let/const bindings, scalar print
boundaries and pure bool conditions. Branches use cloned original scopes; child
locals do not escape into the continuation. Record-valued local initializers may
use typed bool fields too. Unknown/effectful bound callees, references/resources,
outer-name rebinding, loops, unreachable suffixes and incomplete return paths
remain outside this proof. This is not general function/control-flow lowering.

The whole original function shares 128 statements, at most 32 per block and eight
branch levels. Conditions, locals, prints and return fields share one 4,096-work/
64-depth/32-logical-edge expression budget before recursive scope/type proof.
Print expressions remain ordinary roots: this entry does not add inline logical
print permission. Bool field roots use the existing typed-layout traversal;
ordinary arguments/comparisons/casts/projection bases remain closed. Budgets apply
to input NIR after prior simplification, not the source AST.

## Ordered Return Construction

The [renderer](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_record_returns.rs)
consumes original proven types, preserves print/condition/local/return positions,
and uses the [shared field constructor](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_record_fields.rs)
and forced-edge logical helpers. No field is hoisted across preceding work or
earlier fields. Inferred local types become explicit so generated calls are not
re-inferred against an old catalog. The complete replacement is installed only
after proof succeeds; rejection leaves body, names and helpers unchanged.

This runs at the existing early declaration-normalization boundary, before
return/region helpers. Shared record-field detection lives in the pure value
contract rather than a declaration-only module. No neutral record or new return
wrapper is synthesized by this pass. Existing declaration/neutral proofs, strict
write/effect authority and compact native limits remain unchanged; no engine or
backend case, foreign source shim or public ABI is added.

## Executable Witness

The [Nuis application](../../tools/nuisc/tests/native_application_bridge/direct_record_return.ns)
adds an effectful record-producing function with direct early and terminal
record-literal returns. Prints 86/87/88 identify its entry and selected branch.
A fallible pure prefix follows those effects and precedes every field. The
literal keeps opposite field orders, short-circuit tags and a genuinely unused
bool field. A caller stores the completed record, checks bool/checksum/word
transport and returns another effectful record whose desired field drives a loop
early exit. This exercises the existing stored effectful-record path, not an
interpreter-only rewrite.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/direct_record_return_cases.rs)
retains wrapping arithmetic and checked division while explicitly modelling
producer entry/branch/prefix/field stages. Healthy rows include skipped prefix
poison and MIN/MAX prefix divisors. Twenty-four core failures retain the preceding
twenty-two and add two prefix faults with later fields simultaneously poisoned.
Four existing cross-poison probes give twenty-eight native/reference failures.
Test-only markers 94 and 96-104 observe ten original pure callees without adding
source effects; the producer's own prints remain original source work.

[Native/reference checks](../../tools/nuisc/tests/native_application_bridge/direct_record_return.rs)
run normal/reversed YIR, exact flushed fault prefixes, once-only reached calls
and canary-guarded native output. Failure preserves accepted State 64 -> 72 -> -2,
rejects later events and leaves completion faulted. The profile stays at 4 work,
64 entries and at most 64 functions, without owned-aggregate heap allocation/drop
in the native bridge.

[Compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_record_return_tests.rs)
pin original effects/scopes/signatures, exact helper counts, idempotence,
function-order independence, atomic dependency/effect rejection, nominal return
types, hidden leaves, resource captures, non-total paths, async exclusion and
unsupported control. Budget controls cover 128/129 statements, 32/33 logical edges,
eight/nine branch levels and cross-return expression exhaustion. Full source/
reference execution and LLVM emission retain unprojected bool failures after
preceding effects and checked prefix work.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/direct_record_return.rs)
checks fresh/cache and two independent source-free images, each with healthy
sessions before and after twenty-four core failures. Binary, host-object, envelope,
lifecycle, packaging and original artifact identities stay fixed. Trapping
children need not flush stdout: native probes prove full prefixes, while CLI pins
State, trap status and any flushed prefix. The four extra native cross-poison
probes are not claimed as CLI delivery cases.

## Validation

Bounded local M2 CPU validation passed 199 distinct tests, including eight new
tests. Repeated focused runs are counted once:

- Compiler: 97 focused/adjacent checks in 55.07 seconds, including five new return
  controls, preceding declaration families and existing logical-tree budget/type/
  native controls. The final run includes the logical-print rejection control.
- Native/reference: 27 checks in 44.99 seconds, including two new witnesses,
  preceding join families and full-fixture bounds controls. The new witness runs
  twenty-eight failures in both normal and reversed YIR order.
- CLI delivery: one new check in 232.35 seconds. Fresh/cache and two independent
  source-free images each run healthy sessions before and after twenty-four core failures.
- Borrowed-buffer integration: 17 checks in 37.41 seconds; host failure state:
  three; repository maintenance: 24; host absolute-path policy: one; file-line
  limits: two; status tensor: 27 in 5.57 seconds.

The ordinary CLI build was already current and passed in 0.02 seconds after
delivery finished. Formatting, whitespace, UTF-8 across 5,727 files, 4,337 local
documentation links and locked offline metadata for 29 packages passed. Drift
matches all 3,708 checks and 25,240 required patterns. All six source fields match
the CLI, complete predecessor hashes are verified and historical commands remain
inert. Coverage, hierarchy and lineage are clean; the next task card is ready and
the coordinate remains active/99.

These are targeted local CPU checks, not a full workspace run. Earlier CLI
delivery families, the four-variant independent/shared matrix and 31-failure
delivery, GPU/Linux execution and physical float/performance tests were not rerun
here. Their receipts remain historical evidence, not new direct-return results.

## Remaining Frontier

The [continuation-join follow-up](nuis-native-joined-record-return-v1.md) subsequently
adds the bounded pure result export below; this receipt retains its original scope.

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. A pure branch-joined binding feeding a later direct record return needs
an explicit continuation-scope join proof; this pass deliberately does not merge
child bindings into that scope. General effects/I/O/async captures, loop-local
return normalization, outer writes, stable ABI, full GPU/Linux parity and
self-hosting remain separate gates.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_values::record_return_tests --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- direct_record_return:: logical_record_fields:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- direct_record_return --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

These are focused entry points; the tensor retains the full adjacent chain.
Complete predecessor commands remain inert history, not commands to execute.

## Predecessor Lineage

All six complete logical-field fields are preserved by SHA-256, with quoted inert
historical commands. Historical results are not counted as rerun tests.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `2155c324a6014811137d23947933d51f973c277f2ee81e42431b05af094e51da` |
| `next_step` | `edb5c15b34745f91847515e7be8071690af3b4d2af2f222ab13d3e63b40b788f` |
| `blocker` | `e16f6661ea43e70d3afda9c30f140d72349fcd1b8bd61221754078d59fb8ef87` |
| `next_action` | `e06ad9cfdeac7f8e803895187b49f08e8282bdda1c91dfb35eba6def8f0dc968` |
| `validation_command` | `32cc2b7c5c14e42f2dd1f5191602ca362f2926892caeec123d5cde78828cd73d` |
| `expected_artifact` | `871892ce8d5834d7509ddddc9ac254770f2818e0692113af57565b7213e0f079` |
