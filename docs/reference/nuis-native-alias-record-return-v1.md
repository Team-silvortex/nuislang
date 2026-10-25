# Native Alias Record Return V1

This bounded CPU follow-up continues the [local-origin checkpoint](nuis-native-local-record-return-v1.md)
on Git beta-0.17.7 at 7eb90ed4 without changing the version.

## Certified Value Copies

The [whole-function return plan](../../tools/nuisc/src/lowering/buffer_loop_outline/record_return_plan.rs)
now propagates an existing local origin through fresh exact-type let/const value
copies. Each original initializer is proved before its new name enters either
scope or the origin set. Its supported owned type must equal the function result,
and its variable source must already be certified in the current lexical scope.
Multi-step, inferred and fan-out copies share this rule. A return of the resulting
binding can satisfy lazy-record return admission without a direct literal return.

The only roots remain completed original record literals with typed lazy bool
fields. Parameters, helper-call results, plain records, rebuilt/projected records
and joined-only results cannot seed a copy chain. An unrelated certified local
that is not returned grants no admission. Children inherit prior origins but do
not export private names. Existing freshness checks reject rebinding, shadowing,
self/forward references and stale names before certification. This is owned-value
provenance, not general memory alias analysis or new pointer/reference authority.

No alias-specific budget is introduced. Copies and their reads consume the
existing whole-function 128-statement, 32-per-block, eight-branch-level and shared
4,096-work/64-depth/32-logical-edge expression budgets. The complete function's
4,096-node neutral preflight remains unchanged. Complete exact-type return paths,
pure initializers, hidden-leaf/resource/loop vetoes and atomic rejection still
apply before names or helpers are allocated.

## No Reconstruction

The existing [binding-position renderer](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_record_returns.rs)
retains each copy as a variable initializer and makes inferred original types
explicit. The [shared field constructor](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_record_fields.rs)
still normalizes only the original record construction. Copies do not synthesize
new record literals, repeat field calls or move work across intervening effects.
There is no second renderer, engine/backend case, foreign shim or new public ABI.

The [compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_alias_return_tests.rs)
test original and simplified NIR, reversed function order, idempotence and unchanged
helper counts against the predecessor. Raw-NIR checks retain multiple let/const
copies, fan-out, inference and inherited scopes. Negative controls retain exact
types, unrelated/unreturned origins, repacking, joined-only results, private-name
escape, stale bindings and transitive dependency revocation. Budget checks pin
27/28 copies at the complete 32/33-statement boundary and 32/33 logical edges.

One execution control explicitly bypasses source simplification: it proves the
raw copy chain, lowers that NIR, runs the reference executor and emits LLVM. Thus
this admission does not depend on a prior optimizer erasing aliases. Both lazy
operators, unprojected-field failures and isolated later checked work are covered.

## Equivalent Application Witness

The [Nuis application](../../tools/nuisc/tests/native_application_bridge/alias_record_return.ns)
adds two pure copies to each producer return path, with different let/const and
inferred forms interleaved with existing prints 83/84 and unused checked work.
Both paths return only `final_record`. The original pure record join, lazy bool
fields, unused fallible fields, opposite field orders, effectful caller storage,
selected loop exit and atomic State update remain intact.

Pure value copies add no observable computation. The [oracle facade](../../tools/nuisc/tests/native_application_bridge/alias_record_return_cases.rs)
therefore deliberately reuses the predecessor's independent evaluator unchanged,
rather than duplicating it or computing expectations from the new compiler output.
The expected source trace and all thirteen original pure-callee counts must stay
identical. Thirty-two core failures and 36 native/reference failures retain the
predecessor matrix, including two isolated post-construction traps with all later
work healthy. There are no invented new fault sites or new pure-callee markers.

[Native/reference checks](../../tools/nuisc/tests/native_application_bridge/alias_record_return.rs)
use normal/reversed YIR, exact flushed prefixes, reached-call counts and guarded
output. Accepted State remains 64 -> 72 -> -2; failure retains the last State,
faults completion and rejects further events. Limits stay at 4 work, 64 entries
and at most 64 functions; no aggregate heap allocation/drop is added to the bridge.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/alias_record_return.rs)
checks fresh/cache and two independent source-free images, each with healthy
sessions before and after 32 core failures. Binary/container, host-object,
envelope, lifecycle and packaging identities stay fixed. A trapping child need
not flush stdout: native probes prove full prefixes; CLI pins State, trap status
and any flushed prefix. Four extra native probes are not CLI delivery cases.

## Validation

Bounded local M2 CPU validation passed 223 distinct tests, including nine new
tests. Repeated focused runs are counted once:

- Compiler: 115 focused/adjacent checks in 45.90 seconds, including six new alias
  controls and the preceding local-return, direct-return, continuation-join,
  declaration, logical-tree, type and budget families. Raw-NIR execution is part
  of the new controls, not a source-optimized substitute.
- Native/reference: 33 checks in 55.53 seconds, including two new equivalence
  witnesses, preceding join families and full-fixture bounds controls. The new
  witness retains 36 failures in both normal and reversed YIR order.
- CLI delivery: one new check in 447.06 seconds. Fresh/cache and two independent
  source-free images each run healthy sessions before and after 32 core failures.
- Borrowed-buffer integration: 17 checks in 40.77 seconds; host failure state:
  three; repository maintenance: 24 in 29.504 seconds; host absolute-path policy:
  one; file-line limits: two; status tensor: 27 in 6.31 seconds.

The ordinary CLI build passed in 0.05 seconds after delivery finished. Formatting,
whitespace, UTF-8 across 5,748 files, 4,383 local documentation links and locked
offline metadata for 29 packages passed. Drift matches all 3,737 checks and 25,395
required patterns. All six source fields match the actual CLI; complete
predecessor hashes are verified and historical commands remain inert. Coverage,
hierarchy and lineage are clean, the next task card is ready, and the coordinate
remains active/99.

These are targeted local CPU checks, not a full workspace run or a performance
benchmark. Earlier CLI delivery families, the four-variant independent/shared
matrix and 31-failure delivery, GPU/Linux execution and physical float/performance
tests were not rerun here. Their receipts remain historical evidence, not new
value-alias delivery results.

## Remaining Frontier

The [origin-join follow-up](nuis-native-origin-record-join-v1.md) subsequently adds
two-arm origin certification; this receipt retains its original scope.

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. Next is origin certification for a pure fresh record result join only
when both original arms independently retain certified provenance. Export only
the proven common result, not private bindings or arbitrary branch state, and
preserve exact types, original work, complete budgets and atomic rejection.
The native iteration-call route remains literal-print-only. General effects,
I/O/async/resource capture, outer writes, loop-local return normalization, stable
ABI, full GPU/Linux parity and self-hosting remain separate gates.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_values::alias_return_tests conditional_values::local_return_tests --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- alias_record_return:: local_record_return:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- alias_record_return --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

These are focused entry points; the tensor retains the full adjacent chain.
Historical commands remain quoted inert history, not commands to execute.

## Predecessor Lineage

All six complete local-origin fields are preserved by SHA-256, with quoted inert
historical commands. Historical results are not counted as newly rerun tests.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `bc8170f2e2b0378a956fd76e940d51385fde253985758ee068c8091ee793dc71` |
| `next_step` | `c0e48d0c8c3a4e71863c1d8ab8ac31c314b53c7cd7a28555568c61ae9938e0ea` |
| `blocker` | `713bd202e27eabb736cbdefb7a8afc9550fc83ee497ad5cba44083910f66efdd` |
| `next_action` | `85a9e0153d455ab7c52c325ee7866a548edd48d486a0f29d5df719f6836a4638` |
| `validation_command` | `71d5bab09ef0663d681bbb3ae80c6cf418c65da870c6fe28da36c5c4a0ffec4f` |
| `expected_artifact` | `0f9604c31937b3839bb61d1a97c3c2506ea755fb591f9f5a4352b62eade0d271` |
