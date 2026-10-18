# Native Stateful Effectful Loop Returns V1

This receipt retains its historical i64 execution and typed-admission boundary.
The subsequent [typed loop-return proof](nuis-native-stateful-loop-return-values-v1.md)
repairs ordered word-operation admission and adds real bool/f32/f64/mixed-record
native/source-free execution, without enabling effectful coalescing.

This CPU receipt advances the
[parent-local exit checkpoint](nuis-native-stateful-loop-parents-v1.md).
The Git baseline is beta-0.17.2, commit 00ab72d3. It does not bump the release
or freeze the ABI.

## Independent Return Storage

The existing shared counted-return transport now has an explicit
literal-print capability path. Pure admission still uses
`returns::normalize`; iteration-effect admission uses `normalize_effects`
and its completed dependency catalog. Neither seeds the other with provisional
signatures. Unsupported callees and dependency cycles still reject their callers.

`prepare_effects` revalidates the independently stored typed payload and
private pending signal. It does not run pure coalescing or snapshot-invariant
optimizations: those transformations have not separately proved effect-prefix
preservation. Source references and future bindings are reserved before private
names are chosen. Computed prints and general effects remain outside this profile.

The return expression, including argument checks and the callee's body, executes
before the pending bit is written. Scoped breaks then propagate that signal
through child and parent loops to the source function's return. There is no new
backend return opcode, fixture-name dispatch or host-scheduled replacement loop.

The validator was extracted into
[control_body.rs](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers/control_body.rs)
to keep the existing scalar-helper module below the 800-line source limit.

## Real Source And Selected Effects

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/state_loop_returns.ns)
has a leading parent loop and a leading/trailing child. A selected child return
follows `print(10)`, a current-value update and `print(11)`/`print(12)`.
An alternate child path continues locally. A later parent return follows the
completed child and `print(23)`/`print(24)`.

Both returns call the real `returned(current, index, outer, 1 / root)`.
The callee prints 30 and computes
`current + 10 * index + 100 * outer + 1 / proof`.
Thus root 0 traps before the callee's print, while root 2 traps after it.
A successful child return skips the remaining child body, parent prints 23/21/22,
later parent/child invocations and fallthrough print 31. Callback work after
the returning function still executes: prints 35 and 40 compute the new State.

Four source variants cross original/mirrored guards with leading/trailing child
steps. The independent oracle uses actual source loops and an immediate Rust
closure return, not generated pending bits or normalized break transport.
Starting at seed 4, the first selected child return publishes total 111 for a
trailing child or 121 for a leading child, with trace
`5,8,10,11,12,30,35,40`.

## Full Reached Bounds

The callback-local reservation policy is unchanged. A two-trip parent plus a
three-trip child reserves five units even when the child returns on its first
iteration. Budget 4 rejects after prints 5/8, before child print 10. Budget 5
succeeds without reserving a later, now-unreached child. Budget 9 also succeeds.

An unentered child with an i64::MAX bound and invalid return divisor does not
reserve work or evaluate that divisor. Reached oversized child and parent bounds
still fail. Early returns never refund an entered loop's declared full bound.

## Native And Reference Evidence

[Unit tests](../../tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects_returns_tests.rs)
check explicit profile isolation, typed private transport, bool/float/record return admission,
manual witnesses, wrong values, unknown/cyclic callees, free references,
computed prints and unreachable suffixes.
Bool/float/record payloads have admission/outlining evidence here; this receipt's
complete stateful/source-free early-return workload returns i64 from `walk`.
It must not be read as a new complete typed-payload execution matrix.

[Native/reference tests](../../tools/nuisc/tests/native_application_bridge/state_loop_returns.rs)
run both normal and reversed YIR order. Native observers inspect the actual
return callee's four arguments and the final bool-computing callee. The word
driver feeds each real callback output into the next input and checks status,
canaries and raw retained f64 State words, including negative zero and NaN bits.

The main native matrix has 64 input combinations, four marker words and four
source variants. Across both YIR orders it checks 10,240 healthy accepted
callbacks, plus 64 selected arithmetic traps after 192 accepted prefix callbacks.
A further budget matrix checks 80 healthy accepted callbacks and 56 reservation
traps after 168 accepted prefix callbacks.
The reference engine additionally verifies the last published State after
failed events, Faulted phase, rejection of later events and failed completion.
Native fatal evidence checks flushed prefixes, not post-abort memory inspection.

[AOT checks](../../tools/nuisc/src/aot_application_effect_call_state_loop_returns_tests.rs)
retain exact three-word State transport, explicit print grants, ordered Effect
edges, shared loop/helper budgets and no owned-aggregate heap calls in the bridge.
Removing any required grant or transitive Effect order rejects the artifact.

## Source-Free Delivery

[CLI delivery tests](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_returns.rs)
check four source variants in fresh, cache-hit and two source-free restored
images: sixteen delivered images. They verify exact stdout, evolving State,
retained marker bits and completion before and after selected failures.
The matrix checks 640 healthy accepted callbacks, 128 fatal events and 384
accepted callbacks preceding those faults.

Before restoration, source, manifest, project cache and original build output
are removed. Binary bytes, host objects, lifecycle, envelope, packaging mode and
input-container identity stay intact through repeated materialization.

## Acceptance And Tensor

Focused serial validation passes 227 distinct tests, including nine new tests:
182 compiler tests, three new native/reference tests, eleven existing native
counted-return/typed-carry regressions, one CLI delivery test, three host fault
tests and 27 tensor tests. Changed Rust files pass targeted rustfmt checks.
This is not full-workspace, strict-Clippy, new Linux/GPU/Windows, performance,
formal-verification or global-memory-safety acceptance.

The selected tensor coordinate remains
`standard-library/ns-nova/persistent-application-session`, active at 99.
This receipt does not complete this coordinate.
All six predecessor fields remain unabridged with complete SHA-256 lineage;
previous validation commands are quoted inert here-document data.
The rebuilt normal CLI must match all six source fields, the ready task card
and clean drift/coverage/manifest/milestone/hierarchy/lineage status.
There are 3,356 drift checks with no failures or duplicate IDs.

The predecessor hashes are:

| Field | Complete SHA-256 |
| --- | --- |
| `evidence` | `2a2a48f6b984109783b8722bd4db5920bfb5e1bd66677c9f2501c4acc50df877` |
| `next_step` | `a7e62c748c2367c232e1bb9243426a1147725f865c147ca94c6ea839556b968b` |
| `blocker` | `fedb8c2916402e3da4e9707e0c713184b8c036f70474b53816735354676dedf7` |
| `next_action` | `028ed52acb596292f5a91fe97ec012232452fb3bafa9c8031a6c9cbf2f96aa9c` |
| `validation_command` | `a6ddd84fabf73a9d2e055a951bb9829ecbbc5696ce9730da23f28c7e2bcf8bf2` |
| `expected_artifact` | `8b271113441f3276fe069e27d39bf8196a0c1042e1bbff0cf12d69cf58276cb9` |

This checkpoint's next bool/float/record execution boundary is advanced by the
[typed loop-return follow-up](nuis-native-stateful-loop-return-values-v1.md).
Effectful snapshot/coalescing optimization still requires a separate proof.
Trailing-parent composition, generalized iteration I/O/resources, asynchronous
effects, stable ABI, broader entering-State writes and completed self-hosting
remain separate.
