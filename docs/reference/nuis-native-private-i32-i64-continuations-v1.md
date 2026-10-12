# Native Private I32 I64 Continuations V1

This complete 382-test checkpoint remains historical. The separate
[i64/i32 reverse follow-up](nuis-native-private-i64-i32-continuations-v1.md)
extends the next orientation without reinterpreting this receipt's scope.

The persistent application-session CPU path carries one exact i32 outward owner
beside an independently initialized private i64 child through bounded terminal
mixed selections and sequential continuations. This extends the
[cross-width floating checkpoint](nuis-native-private-cross-float-continuations-v1.md)
without exporting a child or adding a public tuple ABI.

## Single Outward Target

The separate numeric-pair predicate admits only this new unqualified i32/i64
orientation. Previously proved integer/float/bool routes retain their authority.
Generic owned-field installation is not admission. The source helper remains
**bool-returning**, and only the i32 owner reaches its enclosing return prefix.

One delegated/prefix-owned outward target, readonly entering values, unsealed
prefix-defined private children, current/retained versions, both write orders
and lexical child scope remain mandatory. Third/unrelated entering writes,
seals, qualifier/resource fields, hidden ordinary logical work, child leaks
and partial exits reject the candidate atomically. Other i32 orientations
remain separate; this proof does not admit the reverse i64/i32 case.

## Exact Private Transport

The closed compiler-private record has exact independent fields:

```text
owner: i32 / child: i64
```

The i32 neutral uses the existing checked cast of literal i64 zero, not a
conversion of a live field. The i64 neutral remains zero. Both typed projections
read the same selected record. Neither field borrows bits from the other;
the new pair never uses bool/i32 packing or narrows its private i64 child.
No source replay, new owned allocation/drop, public result or common
effect/backend permission is introduced.

The original whole-forest limits remain 64 nodes, 256 expressions, 32 logical
edges, 16 statements per body/suffix, depth eight and 31 captures plus a gate.
The exact seven-node/seven-expression/zero-logical-edge boundary, adjacent
rejection, 16/17 statements, 31/32 captures and depth 7/8 remain tested.

The enclosing source bool/i32 routes may still use their existing canonical
codec. Existing i32 arithmetic may widen operands and range-check the result
before restoring i32. This is not an entirely codec-free or cast-free LLVM
unit. The AOT fixture check separately recognizes zero/negative-one constant
construction, width-safe complement of an i32, and the exact legacy canonical
bool/i32 restoration shape and producer. Constant/producer proofs have an
eight-level bound. It explicitly rejects the real i64 child parameter as a
narrowing source; no new compiler cast permission follows from this test.

## Source And Probes

[The Nuis source](../../tools/nuisc/tests/native_application_bridge/return_i32_i64_private_pair.ns)
takes independently typed i32 and i64 seeds and retains them in a fourteen-word
state. Current child signs control i32 complement work. The source uses
negative-one minus the value at each width, avoiding intermediate negation of
the signed minimum. The i32 negative-one type constant is established before
its callee-local selection.

Both write orders, nested selections for private zero/negative-one versions,
a shared continuation, seals, unused work and caller continuation remain real
source computations. Pair snapshots may differ from the final owner; a final
bool alone cannot prove correct current-value transport.

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_i32_i64_oracle.rs)
crosses eleven i32 seeds with thirteen i64 seeds and both gates: 572 cases per
function inventory. The matrix includes both signed extrema, values outside
i32 range in the child, zero/negative-one and source/diagnostic marker values.
Imperative replay uses native-width complements, never the compiler carrier.

Native-only entry probes observe the actual sign-extended i32 argument and
actual full-width i64 argument at after-work, plus the actual final i32 owner
at answer-work. They replace neither source computations nor callback state
with oracle values. Normal/reversed function inventories retain real calls,
joins, traps, actual accepted-state forwarding and output canaries.

The shared harness previously rejected any observed word equal to its AFTER
diagnostic sentinel, -70002. That incorrectly rejected legal i32/i64 field
values. It now compares the complete actual and independent expected traces;
event-position failure prefixes and real SIGILL/SIGTRAP remain mandatory.
Late faults deliberately carry owner -70002 beside child 96, so both a
diagnostic sentinel and a source stage marker occur as valid data before the
real failure boundary. This is a test-harness repair, not a runtime trap policy
change or permission to continue after failure.

Reference execution covers the full Cartesian matrix. CLI acceptance uses
four extreme/marker seed pairs crossed with both gates, retaining both nested
zero/negative-one paths and i32/i64 extrema without running a huge CLI corpus.

## Executed Acceptance

The 2026-10-12 serial scoped cohort has **382 distinct tests**:
287 compiler unit/artifact tests, 64 native bridge tests, one CLI test,
three host failure-policy tests and 27 tensor tests. Ten tests are new.
Repaired/repeated runs and the complete 373-test predecessor are
not added to the current count. The new integer CLI matrix replaces, rather
than counts as rerunning, the predecessor's cross-width floating CLI receipt.

The new matrix runs 1144 successful native processes and
18 fatal native processes across normal/reversed function inventories.
It proves 3468 accepted callbacks, independently initialized full-width pair
and final-owner values, current reads, source order and width-driven canaries.
Nine original proof stages trap before callee entry, including marker-valued
late failures. Reference cleanup does not make native process traps recoverable.

Four CLI images cover fresh build, cache reuse and two successive source-free
restores. They prove 36 CLI event traps and 372 accepted callbacks, including
healthy reentry after each failure. Only the last accepted open/event state is
published; later event/close requests and completion remain absent after fatal
events. Executable, native objects, packaging, envelope and lifecycle identity
stay unchanged. Images and restoration input containers remain byte-identical
after use; diagnostic-bearing fresh/cache container bytes need not match.

The prior i32/i64/float-ancestor, dual-i64, both integer/float orientations,
same-/cross-width float, five-child-type event failure/budget and helper-entry
native regressions remain included. The rebuilt ordinary CLI reports
2655 clean drift checks / 18520 required patterns and clean coverage/hierarchy/
task-card lineage. It checks six source-equal fields and six complete
predecessor hashes. The selected cell stays `active/99`, not broad
application-session closure.

Reproduce from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_i32_ancestor_private_bool return_i64_ancestor_private_bool private_event_budget private_event_failure helper_entries_share_exact_budget helper_entries_reject_before_body helper_entries_skip_unselected_paths helper_entries_use_unsigned_limits helper_entries_count_loop_free_roots helper_entries_include_scoped_iterations return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_i32_i64_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
```

The tensor retains all six complete 373-test predecessor fields after their
cross-float history markers:

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `20da373cb8d03e2f2cbf20761d22684c659e05b82928bacf02d655295a9832df` |
| `next_step` | `2d7458a781ca17bbbf2d60b6e8b566451462da13c793b9dbf7188b8428ffe7d2` |
| `blocker` | `64593e94d5b1f27be00820d04d2eba9e85f084465f72930d71ba4b22d220443b` |
| `next_action` | `3fbea244746400586254d4f39c67bc6bfd0e1914bb87105ad18ec557421b788e` |
| `validation_command` | `fa652efe17b6d3f371d81c20d05f3bde4571d7683dc1571005144f065bcc2b06` |
| `expected_artifact` | `62ded328a3a6118c967a5baa2edbf5cff3783406d68ac612525c505666e11012` |

## Remaining Boundary

The next independent proof is one exact i64 outward owner beside a private i32
child. Preserve original signed widths through independently typed neutral
fields and projections, not bool/i32 packing or truncation of a live i64.
Prove independent extrema/current reads, both inventories, source-order traps,
marker-safe full traces, actual-state forwarding/canaries and source-free
identity/publication before granting other i32 numeric combinations.

This is a macOS CPU receipt, not new GPU/Linux/Windows execution, measured
performance, formal memory safety, self-hosting, full-workspace or strict-Clippy
acceptance. Broader application-session closure and public ABI stabilization
remain open.
