# Native Private I64 I32 Continuations V1

This complete 391-test checkpoint remains historical. The separate
[same-width i32 follow-up](nuis-native-private-i32-i32-continuations-v1.md)
extends the next orientation without reinterpreting this receipt's scope.

The persistent application-session CPU path carries one exact i64 outward owner
beside an independently initialized private i32 child through bounded terminal
mixed selections and sequential continuations. This is the reverse of the
[i32/i64 checkpoint](nuis-native-private-i32-i64-continuations-v1.md), not a
public tuple ABI or new common backend/effect permission.

## Single Outward Target

The exact unqualified i64-owner predicate now admits an i32 child alongside its
previously proved i64/f32/f64 children. Generic record installation remains
separate from admission. The source helper still returns **bool**; only the
authorized i64 owner reaches its enclosing return prefix.

One delegated/prefix-owned outward target, readonly entering values, current/
retained versions, both source write orders, private lexical scope and seals
remain mandatory. Third/unrelated entering writes, qualified/resource fields,
hidden ordinary logical work, child leaks and partial exits reject atomically.
Other i32 orientations, including i32/i32 and i32/float or float/i32, remain
separate proofs rather than a general numeric cross-product.

Three historical ancestor/dual-i64/integer-float test files previously rejected
the entire i64/i32 orientation. Their rejection cases now use a sealed i32 child
that cannot be rewritten, and their direct unsupported-type checks retain
String/resource rejection. The four new proof families retain whole-input/name
atomicity, reference/optional/generic vetoes and the original budgets. This is
not deletion of safety assertions or a broader write grant.

## Exact Private Transport

The closed compiler-private record preserves independently typed fields:

```text
owner: i64 / child: i32
```

Installation reuses the existing generic record and early owned-field route,
even though the child is i32. It does not fall through to the old bool/i32 codec.
The owner neutral is i64 zero; the child neutral is the existing checked i32
cast of literal i64 zero. Both projections read the same selected record.
There is no live i64 owner narrowing, field bit sharing, source replay, new
owned allocation/drop or exported child.

The original whole-forest limits remain 64 nodes, 256 expressions, 32 logical
edges, 16 statements per body/suffix, depth eight and 31 captures plus a gate.
The exact seven-node/seven-expression/zero-logical-edge boundary and adjacent
rejections, 16/17 statements, 31/32 captures and depth 7/8 remain tested.

The AOT fixture's narrowing checks accept only bounded literal zero/negative-one
construction and the width-safe complement of an already typed i32. The latter
uses the existing widened i32 arithmetic before restoring i32, not an arbitrary
i64 payload. Constant proof has an eight-level bound and explicitly rejects the
real i64 initialize parameter as a narrowing source. This test-only checker
grants no compiler cast permission. This fixture needs no legacy bool/i32
restoration; existing routes elsewhere are not removed or declared invalid.

## Source And Probes

[The Nuis source](../../tools/nuisc/tests/native_application_bridge/return_i64_i32_private_pair.ns)
uses independently typed i64 and i32 seeds, retained in a fourteen-word state.
Current child signs drive full-width owner complement work. Negative-one minus
the value avoids intermediate negation of the signed minimum at both widths.
The i32 child establishes its typed negative-one constant inside child-work;
child sign/zero/negative-one comparisons use typed literal casts.

Both write orders, nested private zero/negative-one versions, shared continuation,
seals, unused work and caller continuation remain real source computations.
Pair snapshots may differ from the final owner, so a final bool is insufficient
evidence for correct field/current-value transport.

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_i64_i32_oracle.rs)
crosses thirteen i64 seeds with eleven i32 seeds and both gates: 572 cases per
function inventory. Full signed extrema, owners outside i32 range, source
markers and diagnostic sentinels occur as legal data. Imperative replay uses
native-width complements, never the compiler carrier.

Native-only entry probes observe the actual full-width i64 owner and actual
sign-extended i32 child at after-work, and the actual final i64 owner at
answer-work. They replace neither source computations nor accepted state with
oracle values. Both normal/reversed function inventories retain real calls,
joins, source-order traps, actual state forwarding and output canaries.

The shared full-trace comparison distinguishes a diagnostic sentinel from a
matching field word. Event-position prefixes, real SIGILL/SIGTRAP and the
absence of actual post-failure status/canary publication remain mandatory.
Late faults carry legal owner -70002 beside child 96 before the failure boundary.
Reference fault cleanup does not make native process traps recoverable.

Reference and native acceptance use the full Cartesian inventory. CLI acceptance
uses four independently typed extreme/marker pairs crossed with both gates,
retaining both nested private zero/negative-one paths without a huge CLI corpus.

## Executed Acceptance

The 2026-10-12 serial scoped cohort has **391 distinct tests**:
292 compiler unit/artifact tests, 68 native bridge tests, one CLI test,
three host failure-policy tests and 27 tensor tests. Ten tests are new.
Repaired/repeated runs and the complete 382-test predecessor are
not added to the current count. The new reverse integer CLI replaces the
predecessor's forward integer CLI in this receipt; that old CLI is not claimed
as rerun.

The new matrix runs 1144 successful native processes and
18 fatal native processes across normal/reversed function inventories,
proving 3468 accepted callbacks. Actual pair/final-owner values, independent
extrema, wide owners, current reads, source order, width-driven accepted-state
forwarding and canaries remain checked. Nine original proof stages fail before
callee entry, including diagnostic/source marker-valued late failures.

Four CLI images cover fresh build, cache reuse and two successive source-free
restores. They prove 36 CLI event traps and 372 accepted callbacks, including
healthy reentry after each failure. Only the last accepted open/event state is
published; later event/close requests and completion remain absent after fatal
events. Executable, native objects, packaging, envelope and lifecycle identity
stay unchanged. Images and restoration input containers remain byte-identical
after use; diagnostic-bearing fresh/cache container bytes need not match.

The prior i32/i64/float-ancestor, dual-i64, both integer/float orientations,
same-/cross-width float, forward integer pair, five-child-type event failure/
budget and helper-entry native regressions remain included. The rebuilt
ordinary CLI checks six source-equal fields and six complete predecessor hashes,
with 2675 clean drift checks / 18683 required patterns and clean coverage/
hierarchy/task-card lineage. The selected cell stays `active/99`, not broad
application-session closure.

Reproduce from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_i32_ancestor_private_bool return_i64_ancestor_private_bool private_event_budget private_event_failure helper_entries_share_exact_budget helper_entries_reject_before_body helper_entries_skip_unselected_paths helper_entries_use_unsigned_limits helper_entries_count_loop_free_roots helper_entries_include_scoped_iterations return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_i64_i32_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
```

The tensor retains all six complete 382-test predecessor fields after their
i32-i64 history markers:

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `db0ed6766a566f0bcd79ce2b6319a2eaa5adab8042df3151b7463ef9f1fd4002` |
| `next_step` | `16293f5b25cc3d7ae1d0cf627b1d9b4dcbbea4250f6af24e78c60581176a114d` |
| `blocker` | `8be42c796840b7bdade3e8c9ea9d8d4a90a7c81ecd49fb7c48e4dc955cfc6a70` |
| `next_action` | `7cc2dfd28c44756ce65a611c42888bf0f391dab35cba1785f8175ed96cf4fa01` |
| `validation_command` | `78f138197f6e6016e947be3954d14e701eaeb02359c59c296eb923d4a5b0cbb0` |
| `expected_artifact` | `9637fc1e471f82782d28e03604a28dc94613dbce88aaa4de3062d53f5a32ff36` |

## Remaining Boundary

The next independent proof is one exact i32 outward owner beside a private i32
child. Preserve both independently initialized original-width values with
typed neutral fields and projections, not bool/i32 packing or aliasing them
into a single value. Retain original authority/budgets, current/retained reads,
both inventories, marker-safe source-order traps, actual-state forwarding/
canaries and fresh/cache/two source-free identity/publication before granting
further i32 numeric combinations.

This is a macOS CPU receipt, not new GPU/Linux/Windows execution, measured
performance, formal memory safety, self-hosting, full-workspace or strict-Clippy
acceptance. Broader application-session closure and public ABI stabilization
remain open.
