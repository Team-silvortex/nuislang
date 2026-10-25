# Native Local Record Return V1

This bounded CPU follow-up continues the [continuation-join checkpoint](nuis-native-joined-record-return-v1.md)
on Git beta-0.17.7 at 7eb90ed4 without changing the version.

## Certified Local Origins

The [whole-function return plan](../../tools/nuisc/src/lowering/buffer_loop_outline/record_return_plan.rs)
now admits an owned record returned through its fresh local binding, without a
direct record-literal return anywhere in the function. After proving a complete
pure let/const initializer, its exact supported type must match the function
result and its original record literal must contain typed lazy bool fields.
Only then is that binding certified as a local origin. A return of that exact
in-scope variable can satisfy the existing lazy-return admission condition.

The origin set follows original lexical scope: children inherit prior origins,
but child-only names do not escape. Existing freshness checks reject shadowing
and writes before an origin can become stale. Parameters, helper-call results,
ordinary records, unreturned lazy locals, fresh value aliases and joined result
identities do not become origins merely by sharing the same type. This is a
bounded initializer/return identity proof, not general alias analysis or a new
reference/pointer model. Earlier simplification can still remove ordinary aliases
before this input-NIR boundary.

Complete exact-type return paths remain required. Source preflight is unchanged:
128 statements, 32 per block, eight branch levels, one 4,096-work/64-depth/
32-logical-edge expression budget, plus the existing whole-function 4,096-node
neutral-expansion preflight. Unknown/effectful initializers, resources/references,
hidden logical leaves, loops and unreachable suffixes remain outside this entry.
An unrelated lazy local that is not returned cannot unlock an otherwise plain
function. No plan failure mutates the original body or allocates helper names.

## Ordered Construction

The existing [return renderer](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_record_returns.rs)
already places normalized field work at each proven binding. This follow-up does
not add a second renderer or move construction to the return site. The returned
variable stays a variable; all fields, including genuinely unused fallible bool
fields, finish before intervening prints and checked work. A later failure does
not publish a partial result or new State.

The [field constructor](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_record_fields.rs),
logical helpers, typed pure join construction and original capture scopes remain
shared. No engine/backend cases, foreign source shim, public ABI, reference
authority or enlarged native profile is introduced.

## Executable Witness

The [Nuis application](../../tools/nuisc/tests/native_application_bridge/local_record_return.ns)
retains the prior pure record join and opposite field orders, but both producer
branches now construct `completed` with let/const and return only that binding.
Print 83 follows complete initialization; a fallible `local_after` call is then
bound to an unused scalar, followed by print 84 and the return. There are no direct record-literal
returns in this producer. Its caller stores the completed effectful record and
feeds an existing selected loop early exit and State update.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/local_record_return_cases.rs)
explicitly models the new post-construction stage, including checked unused work
and test-only pure-callee marker 107. It does not delegate expected execution to
the preceding oracle. Thirteen original pure callees use markers 94 and 96-107.
Thirty-two core failures retain the preceding 30 and add two isolated post-construction
faults with all later work otherwise healthy, so a later trap cannot hide removal
of the unused checked computation. Four prior
cross-poison probes produce 36 native/reference failures. Healthy rows include
both branches, unused lazy fields, skipped poison and MIN/MAX post divisors.

[Compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_local_return_tests.rs)
exercise source and simplified NIR, let/const/inferred bindings, inherited scope,
order independence, idempotence and atomic dependency revocation. They reject
unreturned/unrelated origins, general aliases, joined-only origins, nominal/type/
reference mismatches, private-name escape, rebindings, hidden logic and loops.
Whole-function statement/logical bounds and source/reference/LLVM checks retain
unprojected-field and post-construction failures.

[Native/reference checks](../../tools/nuisc/tests/native_application_bridge/local_record_return.rs)
run normal/reversed YIR with reached-call counts, exact flushed failure prefixes
and canary-guarded native output. Failure retains accepted State 64 -> 72 -> -2,
faults completion and rejects later events. Limits remain 4 work, 64 entries and
at most 64 functions, without aggregate heap allocation/drop in the bridge.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/local_record_return.rs)
checks fresh/cache and two independent source-free images, each with healthy
sessions before and after 32 core failures. Binary/container, host-object,
envelope, lifecycle and packaging identities stay fixed. Trapping children need
not flush stdout: native probes prove complete prefixes; CLI pins State, trap
status and any flushed prefix. Four extra native probes are not CLI cases.

## Validation

Bounded local M2 CPU validation passed 215 distinct tests, including nine new
tests. Repeated focused runs are counted once:

- Compiler: 109 focused/adjacent checks in 60.90 seconds, including six new local
  return controls and the preceding direct-return, continuation-join, declaration,
  logical-tree, type and budget families.
- Native/reference: 31 checks in 52.13 seconds, including two new witnesses,
  preceding join families and full-fixture bounds controls. The final new witness
  runs 36 failures in both normal and reversed YIR order, including isolated
  post-construction faults with no later trap to mask an eliminated check.
- CLI delivery: one new check in 295.49 seconds. Fresh/cache and two independent
  source-free images each run healthy sessions before and after 32 core failures.
- Borrowed-buffer integration: 17 checks in 41.38 seconds; host failure state:
  three; repository maintenance: 24 in 28.820 seconds; host absolute-path policy:
  one; file-line limits: two; status tensor: 27 in 6.24 seconds.

The ordinary CLI build passed in 0.06 seconds after delivery finished. Formatting,
whitespace, UTF-8 across 5,741 files, 4,368 local documentation links and locked
offline metadata for 29 packages passed. Drift matches all 3,728 checks and 25,347
required patterns. All six source fields match the actual CLI; complete
predecessor hashes are verified and historical commands remain inert. Coverage,
hierarchy and lineage are clean, the next task card is ready, and the coordinate
remains active/99.

An earlier native trial with a computed post-construction print was rejected by
the existing literal-print profile. It is not a passing result; the final witness
uses an unused checked scalar binding instead and leaves that print gate explicit.

These are targeted local CPU checks, not a full workspace run. Earlier CLI
delivery families, the four-variant independent/shared matrix and 31-failure
delivery, GPU/Linux execution and physical float/performance tests were not rerun
here. Their receipts remain historical evidence, not new local-return results.

## Remaining Frontier

The [value-alias follow-up](nuis-native-alias-record-return-v1.md) subsequently adds
fresh exact-type copy chains; this receipt retains its original scope.

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. Next is fresh pure value-alias propagation from a certified local,
for example `let alias = completed; return alias`, without granting parameter,
helper-result, arbitrary join or pointer/reference origins. Preserve exact types,
scope, original construction order and complete budgets. General effects/I/O/
async captures, outer writes, loop-local return normalization, stable ABI,
full GPU/Linux parity and self-hosting remain separate gates.

The native iteration-call route still admits literal prints only. Computed
scalar print expressions can pass the early return proof but require separate
downstream admission in that route; this witness deliberately keeps prints
literal and retains fallible post-construction work as a scalar binding.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_values::local_return_tests --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- local_record_return:: joined_record_return:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- local_record_return --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

These are focused entry points; the tensor retains the full adjacent chain.
Historical commands remain quoted inert history, not commands to execute.

## Predecessor Lineage

All six complete continuation-join fields are preserved by SHA-256, with quoted
inert historical commands. Historical results are not counted as rerun tests.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `fdee8ccc9c8dbe0b4ce16605a863cad6179ecb8bbeccfd98b00c5568eaeab92d` |
| `next_step` | `be6d49d0f3fd729297e825fa2c6793c4327132847b390f80f2d12a4a84b1bcf8` |
| `blocker` | `e738b6a1cae5ea1bd1e1945d23658197a263171137af4ebec9510c835138a363` |
| `next_action` | `c7c50380c5ebfb1e1d3c8fc27ae42f5c44b3cacf8edea0141b1496d555880f3c` |
| `validation_command` | `520364925ce6260bdbff981cb1176ba6ca345e24b649f844a7136f3560a13d63` |
| `expected_artifact` | `2f9add355cb7848a87db1fbb498a6b63d6c65d03a90e8985c485cae5b6173d80` |
