# Native Private I32 Float Continuations V1

This is the complete 415-test checkpoint. The subsequent
[float/i32 reverse proof](nuis-native-private-float-i32-continuations-v1.md)
extends the opposite orientation without rewriting this scoped acceptance.

The persistent application-session CPU path carries one exact i32 outward owner
beside an independently initialized private f32 or f64 child through bounded
terminal mixed selections and sequential continuations. This follows the
[same-width integer checkpoint](nuis-native-private-i32-i32-continuations-v1.md),
without a public tuple ABI or new common effect/backend permission.

## Single Outward Target

A separate exact unqualified i32-owner/float-child predicate admits these two
widths. Generic owned-record installation remains separate from admission.
The source helper still returns **bool**; only the authorized i32 owner reaches
its enclosing return prefix.

One delegated/prefix-owned outward target, readonly entering values, current/
retained versions, both source write orders, private lexical scope and seals
remain mandatory. Third/unrelated entering writes, qualified/resource fields,
hidden ordinary logical work, child leaks and partial exits reject atomically.
Other i32 orientations with a floating outward owner and private i32 child
remain separate. Existing rejection lists retain unsupported unsigned/resource
fields and reverse float/i32 pairs instead of rejecting the newly proved route.

## Exact Private Transport

The closed compiler-private record retains independent original-width fields:

```text
owner: i32 / child: f32 or f64
```

The owner neutral is the existing checked i32 cast of literal i64 zero.
The child neutral has its original f32/f64 zero type. Both typed projections
read the same selected record but retain distinct names and current values.
The early owned-field route does not select the old bool/i32 pair codec.
There is no live float/integer conversion, field bit borrowing, source replay,
new owned allocation/drop or private-child export.

The original whole-forest limits remain 64 nodes, 256 expressions, 32 logical
edges, 16 statements per body/suffix, depth eight and 31 captures plus a gate.
Both width-specific four-test families retain the exact seven-node/seven-
expression/zero-logical-edge boundary, adjacent rejection, 16/17 statements,
31/32 captures, depth 7/8 and whole-input/name atomicity.

AOT checks exact i32/f32 and i32/f64 layouts, i32 helper results, guard-first
transitive Effect paths and default/empty-print/removed-Effect policy vetoes.
YIR floating/integer casts remain absent. The integer narrowing checker accepts
bounded literal zero/negative-one construction, width-safe complement of an
already typed i32 and exact existing canonical bool/i32 restoration with a
proved producer. Constant/producer depth is bounded at eight; the actual i64
proof parameter is rejected as an arbitrary narrowing source.

The LLVM check permits legacy auxiliary f32 widening only when its sole consumer
is a dead integer conversion. Synthetic live floating/integer uses and f64-to-
f32 narrowing reject. These test-only structural checks grant no new compiler
cast permission. Existing surrounding bool/i32 routes and nonparticipating
bookkeeping remain valid; the whole LLVM unit is not claimed cast/codec-free.

## Source And Probes

[The Nuis source](../../tools/nuisc/tests/native_application_bridge/return_i32_float_private_pair.ns)
takes independent i32 and floating seeds, retained separately in a fourteen-word
state. It does not derive the integer from the float. Current child signs drive
i32 complement work; typed negative-one minus the value avoids intermediate
negation of the signed minimum. Child work changes only the original IEEE sign
bit by floating negation.

Both write orders, nested private zero versions, shared continuation, seals,
unused work and caller continuation remain real source computations. Pair
snapshots may differ from the final owner. A final bool alone cannot prove
integer width, IEEE payload preservation or correct current-value reads.

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_i32_float_oracle.rs)
crosses eleven i32 seeds with twelve IEEE child words and both gates: 528 cases
per width/inventory. Integer extrema and diagnostic marker values are independent
of floating signed zeros, smallest/largest subnormals, finite values, infinities
and quiet/signaling NaN payloads. Imperative replay uses an i32 complement and
raw IEEE sign-bit XOR, never the compiler carrier or numeric conversion.
Reference healthy execution covers the full raw matrix; selected reference
failure/cleanup inputs are finite. The public f32 state-text expectation follows
its existing promoted f64 formatter, separately from the shortest f32 input
text. Raw state-word checks stay unchanged; display precision is not numeric
field widening. A smallest-subnormal audit distinguishes both text forms while
requiring the original state word to remain exactly one.

Native-only entry probes observe the actual sign-extended i32 owner and original-
width IEEE child bits separately at after-work, and the actual final i32 owner
at answer-work. f32 child observations zero-extend its original 32-bit word;
f64 observations preserve all 64 bits. Neither source computations nor callback
state are replaced with oracle values. Both normal/reversed function inventories
retain real calls, joins, traps, actual accepted-state forwarding and canaries.

The shared full-trace comparison distinguishes a diagnostic sentinel from a
matching field word. Native late faults carry legal owner -70002 beside raw
child word 96, using independently initialized negative subnormal inputs.
Event-position prefixes, real SIGILL/SIGTRAP and absence of actual post-failure
status/canary publication remain mandatory. Reference cleanup is not native
process recovery.

CLI acceptance instead uses six independently paired integer/finite-float
inputs crossed with both gates: 24 healthy cases per image. Typed signed-zero
and decimal values are checked, not arbitrary IEEE NaN payload text roundtrips.
Finite CLI late faults retain legal owner -70002 but do not claim the native
raw-word-96 seed. The two receipts are deliberately separate.

## Executed Acceptance

The 2026-10-12 serial scoped cohort has **415 distinct tests**:
306 compiler unit/artifact tests, 78 native bridge tests, one two-width CLI test,
three host failure-policy tests and 27 tensor tests. Sixteen tests are new.
Focused/repeated runs and the complete 400-test predecessor are
not added to the current count. The new two-width mixed CLI replaces the
predecessor's same-width-integer CLI in this receipt; that old CLI is not claimed
as rerun.

The new matrix runs 2112 successful native processes and
36 fatal native processes across both widths and normal/reversed inventories,
proving 6408 accepted callbacks. Actual pair/final-owner words, independently
initialized extrema/IEEE payloads, current reads, source order, accepted-state
forwarding and width-driven canaries remain checked. Nine original proof stages
per width fail before callee entry, including marker-valued late failures.

Eight CLI images cover fresh build, cache reuse and two successive source-free
restores for each width. They prove 72 CLI event traps and 936 accepted callbacks,
including healthy reentry after each failure. Only the last accepted open/event
state is published; later event/close requests and completion remain absent
after fatal events. Executable, native objects, packaging, envelope and lifecycle
identity stay unchanged. Images and restoration input containers remain byte-
identical after use; diagnostic-bearing fresh/cache container bytes need not match.

The prior ancestor, dual-i64, both integer/float orientations, same-/cross-width
float, all four bounded i32/i64 integer pairs, five-child-type event failure/
budget and helper-entry native regressions remain included. The rebuilt ordinary
CLI checks six source-equal fields and six complete predecessor hashes, with
2716 clean drift checks / 19037 required patterns and clean coverage/
hierarchy/task-card lineage. The selected cell stays `active/99`, not broad
application-session closure.

Reproduce from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_i32_ancestor_private_bool return_i64_ancestor_private_bool private_event_budget private_event_failure helper_entries_share_exact_budget helper_entries_reject_before_body helper_entries_skip_unselected_paths helper_entries_use_unsigned_limits helper_entries_count_loop_free_roots helper_entries_include_scoped_iterations return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_i32_float_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
```

The tensor retains all six complete 400-test predecessor fields after their
i32-i32 history markers:

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `ef46624b436d8ccbf8a071db91907d89a72cbe663e842f7a7193a320e570ab44` |
| `next_step` | `e82ff3d56ab1a46e2167a7fbb4db4d792bdaf5bd8e50e65a2cabfdbb9e7c42dd` |
| `blocker` | `0e56472ab7b4ca6cf7d8dcd69d615bbb00dab6835bcac276bb70bc8fbc1bb7db` |
| `next_action` | `49779ad3b0b225267fb1024cbfa1c6698a34a17fd5c21cb6c208f3aeceae19a7` |
| `validation_command` | `4cf327683a861bbb40f5ee1d5a8b9b55fd21e0f8600ecb9b0ede45334ba2ef65` |
| `expected_artifact` | `1b9ce043b863d9da228984b0f12c344b4b637d5ff116fe9fd6b28effa3377a32` |

## Remaining Boundary

The next separate proof is one exact f32 or f64 outward owner beside a private
i32 child. Preserve independently initialized original-width IEEE/integer
fields, typed neutrals and projections without numeric conversion or bool/i32
pair encoding. Retain original authority/budgets, current/retained reads, both
inventories, actual raw IEEE/integer extrema, marker-safe source-order traps,
actual-state forwarding/canaries and fresh/cache/two source-free identity/
publication. Do not infer reverse permission from the generic record mechanism.

This is a macOS CPU receipt, not new GPU/Linux/Windows execution, measured
performance, formal memory safety, self-hosting, full-workspace or strict-Clippy
acceptance. Broader application-session closure and public ABI stabilization
remain open.
