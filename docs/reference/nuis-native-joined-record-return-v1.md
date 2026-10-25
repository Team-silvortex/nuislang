# Native Joined Record Return V1

This bounded CPU follow-up continues the [direct-return checkpoint](nuis-native-direct-record-return-v1.md)
on Git beta-0.17.7 at 7eb90ed4 without changing the version.

## Continuation Join Contract

The [whole-function return plan](../../tools/nuisc/src/lowering/buffer_loop_outline/record_return_plan.rs)
now consumes the existing [typed pure join proof](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_nested_prefix.rs).
Both arms must end in a fresh result with the same name and exact supported owned
type, and the original suffix must observe that result. Scalars, bools and nominal
records share this entry. Only that result enters the continuation scope; all
branch-private bindings remain private. This does not merge arbitrary branch state.

Nested pure local joins, fresh let/const work and typed lazy record fields reuse
their existing proof. Partial joins, nominal mismatches, outer-name writes,
shadowing, unknown/effectful calls, references/resources, hidden logical call
arguments and loops do not gain authority. An ordinary branch can still use its
private cloned scope, but cannot export an unproved result through this fallback.

The complete function is checked before recursive plans or helper allocation:
128 statements, 32 per block, eight branch levels, one 4,096-work/64-depth/
32-logical-edge expression budget. One additional 4,096-node neutral-expansion
budget covers all potential one-sided record locals across the whole function,
not each join separately. Existing strict declaration preflight is unchanged;
ordinary return branches do not synthesize neutrals. The explicit nominal return,
complete return paths and at least one direct lazy-field literal remain required.
Budgets apply to input NIR after earlier simplification.

## Shared Construction

The [return renderer](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_record_returns.rs)
uses the same [typed join constructor](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_nested_prefix.rs)
as declaration normalization. It inserts the original proven result type into
the continuation only after rendering that selection. Source-positioned record
fields, logical helpers, private discarded results and guard construction remain
shared. Local inferred types become explicit before generated calls appear, so
an old catalog is not asked to infer those calls. Rejected whole-function plans
leave body, helper names and generated helpers unchanged.

Print boundaries stay at their source positions. Lazy fields do not move before
join conditions, selected local work, unused record fields or subsequent effects.
No engine/backend cases, foreign shim, public ABI or native profile expansion is
introduced. This is still a bounded CPU normalization contract, not general CFG
or effectful-join lowering.

## Executable Witness

The [Nuis application](../../tools/nuisc/tests/native_application_bridge/joined_record_return.ns)
puts a pure record-valued join after producer entry print 86. Its selected branch
contains a fallible local prefix; one branch also contains a scalar local join.
A genuinely unused record word is still evaluated. Only then may print 89, branch
prints 87/88 and the direct lazy-field returns execute. The completed effectful
record is stored by its caller and feeds an existing loop early-exit State update.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/joined_record_return_cases.rs)
explicitly models the new condition/local/unused-field stages before prints 89
and 87/88; it does not delegate execution expectations to the preceding oracle.
Wrapping arithmetic, checked division, both join arms, nested local selection,
skipped-arm poison and MIN/MAX divisors remain observable. Twelve test-only pure
callee markers use 94 and 96-106. They do not introduce source-side effects.
Thirty core failures retain the preceding 24 and add two condition failures plus
four selected-unused-word failures. Four prior cross-poison probes produce 34
native/reference failures. Later work is poisoned simultaneously to detect
hoisting or premature effects.

[Compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_joined_return_tests.rs)
pin original signatures/effects, private scopes, scalar/bool/record joins,
dependency revocation, invalid original NIR, idempotence, function-order
independence and shared logical work. A separate neutral control proves cumulative
function-wide expansion, including ordinary one-sided return branches.

[Native/reference checks](../../tools/nuisc/tests/native_application_bridge/joined_record_return.rs)
use normal/reversed YIR, exact flushed fault prefixes, reached-call counts and
canary-guarded output. The profile stays at 4 work, 64 entries and at most 64
functions. Failure retains accepted State 64 -> 72 -> -2, faults the session and
denies later events; no aggregate heap allocation/drop is added to the bridge.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/joined_record_return.rs)
checks fresh/cache and two independent source-free images, each with healthy
sessions before and after 30 core failures. Binary/container, host-object,
envelope, lifecycle and packaging identities remain fixed. A trapping process
need not flush stdout: native probes prove the complete prefix while CLI checks
State, trap status and any flushed prefix. The four extra native probes are not
claimed as CLI cases.

## Validation

Bounded local M2 CPU validation passed 207 distinct tests, including nine new
tests. Repeated focused runs are counted once:

- Compiler: 103 focused/adjacent checks in 49.77 seconds, including five joined
  return controls and one cumulative-neutral control. Existing direct returns,
  declaration families and logical-tree/type/budget controls were rerun.
- Native/reference: 29 checks in 48.10 seconds, including two new witnesses,
  preceding join families and full-fixture bounds controls. The new witness runs
  34 failures in both normal and reversed YIR order.
- CLI delivery: one new check in 309.53 seconds. Fresh/cache and two independent
  source-free images each run healthy sessions before and after 30 core failures.
- Borrowed-buffer integration: 17 checks in 44.66 seconds; host failure state:
  three; repository maintenance: 24 in 19.868 seconds; host absolute-path policy:
  one; file-line limits: two; status tensor: 27 in 5.97 seconds.

The ordinary CLI build passed in 0.06 seconds after delivery finished. Formatting,
whitespace, UTF-8 across 5,734 files, 4,353 local documentation links and locked
offline metadata for 29 packages passed. Drift matches all 3,719 checks and 25,299
required patterns. All six source fields match the actual CLI; complete
predecessor hashes are verified and historical commands remain inert. Coverage,
hierarchy and lineage are clean, the next task card is ready, and the coordinate
remains active/99.

These are targeted local CPU checks, not a full workspace run. Earlier CLI
delivery families, the four-variant independent/shared matrix and 31-failure
delivery, GPU/Linux execution and physical float/performance tests were not rerun
here. Their receipts remain historical evidence, not new joined-return results.

## Remaining Frontier

The [local-return follow-up](nuis-native-local-record-return-v1.md) subsequently
adds fresh local-initializer return admission; this receipt retains its original scope.

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. Next is a locally constructed lazy-field record returned only through
its fresh local binding, without borrowing a separate direct literal return's
admission. General effects/I/O/async captures, outer writes, loop-local return
normalization, stable ABI, full GPU/Linux parity and self-hosting remain separate.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_values::joined_return_tests continuation_record_neutrals_share --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- joined_record_return:: direct_record_return:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- joined_record_return --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

These are focused entry points; the tensor retains the full adjacent chain.
Historical commands remain quoted inert history, not commands to execute.

## Predecessor Lineage

All six complete direct-return fields are preserved by SHA-256, with quoted inert
historical commands. Historical results are not counted as newly rerun tests.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `97e60cdbc38cb2ac49e26c5c88a57618fb22bc8eb1c994ae06a133f76f084b67` |
| `next_step` | `2348ad2a5411f28b2655d63ff989dcd4b9cb8c097a48d4649485935bef9f1f34` |
| `blocker` | `4b393bf9bf70ac5cf59d0c247f758c1d8f621f0f6d860019b1203b22b2de554c` |
| `next_action` | `1952a0fdab7f57405f5c3361ac04d9fd6a5c889344b3ad85c5302249d368cd63` |
| `validation_command` | `c6a02f5a556f1795c4e481792c641530d079a02fc05daa706180fbe1a1fe0493` |
| `expected_artifact` | `21f2ce212da0451b397ca2795dbd9fb9699ee2d908ac46fb836b52a5906d0236` |
