# Native Origin Record Join V1

This bounded CPU follow-up continues the [value-alias checkpoint](nuis-native-alias-record-return-v1.md)
on Git beta-0.17.7 at 7eb90ed4 without changing the version.

## Two Original Arms

The [whole-function return plan](../../tools/nuisc/src/lowering/buffer_loop_outline/record_return_plan.rs)
now certifies a fresh pure record result join only after the existing complete
[typed join proof](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_nested_prefix.rs)
has succeeded. Both original result types must exactly equal the owned function
result, and each arm must independently retain certified provenance. Only the
common fresh result enters the continuation's origin set; private names do not.

The new [origin checker](../../tools/nuisc/src/lowering/buffer_loop_outline/record_return_origins.rs)
owns the shared literal/copy classifier and consumes the already typed arm plan.
Each arm clones its incoming origin set, walks original locals in order and
certifies exact-type literal/copy bindings. A nested join exports an origin only
when its own two arms qualify and its typed plan explicitly exports a result.
Discarded joins and condition-only work do not export origins. Final values must
be borrowed original expressions: synthetic defaults cannot create provenance.

An unused lazy local cannot certify an unrelated result. Parameters, helper-call
results, repacked/plain records, one uncertified arm, partial joins, nominal/type/
reference mismatches, rebinding and private-name escape remain excluded. There is
no union-of-seen-origins shortcut or general branch-state merge. Existing direct
returns and local copy chains reuse the same classifier without separate rules.

Original syntax/type/freshness proof precedes provenance recursion. The entire
function retains 128 statements, 32 per block, eight branch levels, one shared
4,096-work/64-depth/32-logical-edge expression budget and the 4,096-node neutral
preflight. There is no per-join reset or new origin-specific cap. The complete
body is rendered only after every function path passes; rejection leaves the
source body, helper names and generated helpers unchanged.

## Shared Rendering

The existing [return renderer](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_record_returns.rs)
still uses the [typed join constructor](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_nested_prefix.rs).
This follow-up changes certification, not field construction or guard assembly.
Original conditions, branch-local work, lazy fields, unused fallible fields and
intervening effects retain their source positions. No engine/backend cases,
foreign shim, public ABI or reference/pointer authority are added.

## Compiler Controls

[Six source/plan controls](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_origin_join_tests.rs)
exercise raw and simplified NIR, reversed order, idempotence, result-only export,
direct literals, local copies, nested joins, inherited roots and discarded work.
They reject either uncertified arm, unused-origin shortcuts, repacking, malformed
types, private-name escape, effects, writes and loops. Dependency revocation is
atomic. Shared 32/33 logical-edge and eight/nine branch-level boundaries apply
before the new provenance walk.

A seventh control in the origin checker explicitly rejects synthetic values on
either side, even when they resemble lazy literals, and rejects a borrowed
unrelated result. Raw-NIR execution bypasses simplification, runs reference and
LLVM lowering, checks both selected values and skipped failures, and pins one
predicate call plus only the selected logical RHS call.

## Application Equivalence

The [Nuis application](../../tools/nuisc/tests/native_application_bridge/origin_record_join.ns)
keeps the prior fallible Leaf join, source entry/branch prints and opposite record
field orders, but constructs the final Record through one fresh pure result join.
Each arm builds a private `completed`, copies it into `alias` and yields only that
value. After the join, print 83, unused checked post-work, a final value copy and
print 84 precede return. The stored effectful caller result still drives a selected
loop early exit and atomic State update.

The [oracle facade](../../tools/nuisc/tests/native_application_bridge/origin_record_join_cases.rs)
deliberately reuses the predecessor's independent evaluator unchanged: moving
literal branch prints before the pure join preserves the expected effect order,
and the new proof must not add field evaluations or invent new fault sites.
Thirty-two core failures and 36 native/reference failures retain all thirteen
pure-callee counts, skipped poison, opposite field order and isolated later traps.

[Native/reference checks](../../tools/nuisc/tests/native_application_bridge/origin_record_join.rs)
run normal/reversed YIR, exact flushed fault prefixes and canary-guarded output.
Accepted State remains 64 -> 72 -> -2; failure keeps the last State, faults
completion and rejects further events. Limits stay at 4 work, 64 entries and at
most 64 functions, without aggregate heap allocation/drop in the native bridge.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/origin_record_join.rs)
checks fresh/cache and two independent source-free images, each with healthy
sessions before and after 32 core failures. Binary/container, host-object,
envelope, lifecycle and packaging identities stay fixed. Native probes pin full
failure prefixes; CLI pins State, trap status and any flushed prefix, since a
trapping child need not flush stdout. Four extra native probes are not CLI cases.

## Validation

Bounded local M2 CPU validation passed 232 distinct tests, including ten new
tests. Repeated focused runs are counted once:

- Compiler: 122 focused/adjacent checks in 51.08 seconds, including six new
  source/plan controls, one synthetic-arm control and the preceding local/copy,
  direct-return, typed join, declaration, logical-tree, type and budget families.
  Raw-NIR execution checks predicate and reached RHS counts without simplification.
- Native/reference: 35 checks in 71.39 seconds, including two new application
  equivalence witnesses, preceding join families and full-fixture bounds controls.
  The new witness retains 36 failures in both normal and reversed YIR order.
- CLI delivery: one new check in 383.70 seconds. Fresh/cache and two independent
  source-free images each run healthy sessions before and after 32 core failures.
- Borrowed-buffer integration: 17 checks in 38.22 seconds; host failure state:
  three; repository maintenance: 24 in 21.677 seconds; host absolute-path policy:
  one; file-line limits: two; status tensor: 27 in 6.46 seconds.

The ordinary CLI build passed in 0.06 seconds after delivery finished. Formatting,
whitespace, UTF-8 across 5,756 files, 4,400 local documentation links and locked
offline metadata for 29 packages passed. Drift matches all 3,747 checks and 25,451
required patterns. All six source fields match the actual CLI; complete
predecessor hashes are verified and historical commands remain inert. Coverage,
hierarchy and lineage are clean, the next task card is ready, and the coordinate
remains active/99.

These are targeted local CPU checks, not a full workspace run or a performance
benchmark. Earlier CLI delivery families, the four-variant independent/shared
matrix and 31-failure delivery, GPU/Linux execution and physical float/performance
tests were not rerun here. Their receipts remain historical evidence, not new
origin-join delivery results.

## Remaining Frontier

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. Next is selected computed scalar printing in the native iteration-call
route, which at this checkpoint admits literal prints only. The later
[computed-print proof](nuis-native-computed-iteration-print-v1.md) adds explicit
exact-i64 authority without upgrading the legacy policy. Prove complete pure print-value
evaluation under the original guard and lexical scope, without moving it before
record fields or publishing partial State. Keep ordinary hidden logical print
roots, general effects/I/O/async/resource capture, outer writes, loop-local return
normalization, stable ABI, full GPU/Linux parity and self-hosting separate.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_values::origin_join_tests origin_join_rejects_synthetic --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- origin_record_join:: alias_record_return:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- origin_record_join --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

These are focused entry points; the tensor retains the full adjacent chain.
Historical commands remain quoted inert history, not commands to execute.

## Predecessor Lineage

All six complete value-alias fields are preserved by SHA-256, with quoted inert
historical commands. Historical results are not counted as newly rerun tests.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `e300c61662e01319470c81dcb825f96f3d0d57c80f8a462b5fcef47e92d1533c` |
| `next_step` | `6cc3315af83d7e3d6fc137a13e784408182ef246eab8b595a1cb79185d18ecf4` |
| `blocker` | `f29a0c65efcef9de431845850e706a087322a9d8faf54a3619cd9ccc809810f3` |
| `next_action` | `79a5e03bb08b295dc491417ac00e97ad7a09797b244a3160e92d429aa3238a63` |
| `validation_command` | `0a2aea9ea72b9ab1f76295619e602e5588ff3779f0fc28c621fd7ee5d3104be8` |
| `expected_artifact` | `73298bd3c95deefef8c9482ae1f83bfe96e74dcafb163a823fa6f6f844b97959` |
