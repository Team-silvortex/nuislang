# Native Private I64 Ancestor Continuations V1

This is the frozen i64/bool checkpoint. The subsequent
[dual-i64 proof](nuis-native-private-dual-i64-continuations-v1.md) also admits the
exact i64/i64 private pair without adding a second outward target.

This is the frozen i64 checkpoint. The later
[float-ancestor proof](nuis-native-private-float-ancestor-continuations-v1.md)
extends exact outward types to f32/f64; the receipt and next task below remain
historical, not a restatement of current eligibility.

The persistent application-session CPU path supports one exact `i64`
prefix-owned ancestor beside one private `bool` child through bounded terminal
mixed selections and sequential continuations. This extends the
[i32-ancestor checkpoint](nuis-native-private-i32-ancestor-continuations-v1.md)
using the existing typed owned aggregate mechanism rather than scalar packing.

The surrounding source helper remains **bool-returning**. An internal selected
region exports one typed integer result to the surrounding return prefix,
which computes the source bool return. Neither the private child nor the carrier
becomes a second public result or a public tuple ABI.

## Single Outward Target

Only exact unqualified `bool`, `i32` and `i64` owner types are admitted.
Integer eligibility requires either a root prefix-owned definition or the exact
delegated target. Delegation carries one target's write permission, not a new
source definition. The first RHS captures the entering version; later RHSs see
proven current versions. Retained branches preserve the incoming owner.

An integer owner admits only one private bool child, defined in the selected
prefix and absent from the entering scope. A numeric private child beside an
integer owner is rejected. Third writes, other entering bindings, const seals,
qualifier/type mismatches, hidden ordinary-leaf logical work, scope leaks and
partial exits remain atomic vetoes. Both source write orders are preserved.

## Exact Private Transport

The compiler-private record has real semantic fields:

```text
owner: i64
child: bool
```

The record installer accepts both exact field types. It seeds inactive guards
with `owner = 0`, `child = false` and projects both fields from the same selected
result. It borrows no integer bit, performs no narrowing or scalar packing, and
adds no division, remainder or fallible work. The observed bridge introduces no
owned aggregate allocation/drop calls.

Existing bool-owner/i64/f32/f64-child records keep their field names, order and
types. The existing bool/bool and both i32 orientations retain their established
closed codecs. Shared installation does not grant a second outward target or
expand source effect/backend authority.

Whole-forest preflight remains ahead of installation. Original statements are
proved once, without copying shared continuations into arms. Limits remain
64 original nodes, 256 expressions, 32 logical edges, 16 statements per local
body/suffix, depth eight and 31 captures plus a gate. The new exact unit boundary
checks seven nodes / seven expressions / zero logical edges, adjacent rejection,
16/17 statements, 31/32 captures and depth 7/8.

## Source And Probes

[The Nuis source](../../tools/nuisc/tests/native_application_bridge/return_i64_ancestor_private_bool.ns)
uses full-width initialization, signed inversion, current owner/child writes in
both orders, nested selections, retained paths, a shared suffix, a const seal,
unused effectful work, a bool return and caller continuation. Inactive proof
operands are zero, making speculative evaluation fail.

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_i64_ancestor_oracle.rs)
uses imperative signed inversion, not compiler fields or codecs. Seven seeds
cover `i64::MIN`, `i64::MAX`, -1, zero, one and values outside the i32 range,
crossing both bool gates for 28 cases.

The shared native test harness observes the **actual compiled callee argument**,
feeds actual callback output into its successor and checks canaries surrounding
the output. It changes only test entry/observations, not source computations,
private helpers, joins or real traps. Production computed-print permissions
remain closed; explicit literal-site grants and effect order are still required.

## Executed Acceptance

The 2026-10-11 serial scoped cohort contains **292 distinct tests**:
232 compiler unit/artifact tests, 27 native bridge tests, three CLI tests,
three host failure-policy tests and 27 tensor tests. Nine units are new.
Repeated/repaired executions and the complete 288-test predecessor receipt
are not added to the current count.

The new native matrix runs 56 successful native processes and
18 fatal native processes across normal/reversed inventories, proving
204 accepted callbacks, exact full-width callee arguments, source order and
output canaries. Nine original argument-proof stages trap before callee entry;
an answer-argument trap occurs before its callee observation.

The new CLI matrix checks four images: fresh build, cache hit and two consecutive
source-free restores. It proves 36 CLI event traps and 516 accepted callbacks,
including healthy reentry after each failure. Only the last accepted open/event
state is published. Later event/close requests and completion are absent after a
fatal event. Executable, native objects, packaging policy, envelope and lifecycle
identity remain unchanged. Each image and the input restoration container are
unchanged by their use; fresh/cache diagnostic-bearing container bytes are not
required to match.

Existing i32-ancestor, five-child-type event failure/budget and helper-entry
regressions remain included. Reference fault cleanup does not turn a native
process trap into a recoverable exception.

The rebuilt ordinary CLI checks 2513 clean registered drift checks / 17470
required patterns, clean coverage/hierarchy/task-card lineage, six source-equal
fields and all six complete predecessor field hashes. The selected cell remains
`active/99`; broader persistent application-session closure is separate.

Reproduce from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_i32_ancestor_private_bool return_i64_ancestor_private_bool private_event_budget private_event_failure helper_entries_share_exact_budget helper_entries_reject_before_body helper_entries_skip_unselected_paths helper_entries_use_unsigned_limits helper_entries_count_loop_free_roots helper_entries_include_scoped_iterations --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_i32_ancestor_private_bool return_i64_ancestor_private_bool native_effectful_selected_calls_build_cache_restore_and_execute_only_selected_arguments --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
```

The tensor preserves the entire predecessor fields, not just a truncated summary:

| Field | Complete 288-test predecessor SHA-256 |
| --- | --- |
| `evidence` | `01b39a54ac01596e0d23c4d62859cdc9a5654f911bf450b75b12e1eebcfc74ed` |
| `next_step` | `ce28e8868de38abed9a1aafdcad14166a5e0f281597c527c0f19a9692efeafe3` |
| `blocker` | `1a10c81a26ec3867658b6f7ce5db354453fb522863d143fad5ab7e64002f8160` |
| `next_action` | `a96e8e249c3a34835c9675b9ca37cc3cc8b055a05ab97785c7244f4641205896` |
| `validation_command` | `eec34875125293b9408048bbdb4c815d15c922f173ff7b04a1154b0b29b6cdf4` |
| `expected_artifact` | `347b1a530c98562012bb9ad9db3047318ad008e5276d985dd945bead6c4cae7b` |

## Remaining Boundary

The next selected task is one exact f32 or f64 outward ancestor beside a private
bool, preserving IEEE bits through typed owned fields. Floating owners,
resources, extra public results, arbitrary effects and partial exits remain
separate.

This receipt is macOS CPU execution, not new GPU/Linux/Windows acceptance,
a performance benchmark, formal memory safety or self-hosting acceptance.
Formatting, whitespace, UTF-8, relative local links and source/test/document caps
are separate checks. No full-workspace or strict-Clippy acceptance is claimed.
ABI stabilization remains independent.
