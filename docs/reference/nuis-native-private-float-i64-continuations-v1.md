# Native Private Float I64 Continuations V1

The persistent application-session CPU path carries one exact f32 or f64 outward
ancestor beside one exact private i64 child through bounded terminal mixed
selections and sequential continuations. This extends the
[i64/float checkpoint](nuis-native-private-i64-float-continuations-v1.md)
using independently typed owned fields, not bit packing or numeric conversion.

The source helper remains **bool-returning**. Only the floating owner is exported
to its enclosing return prefix. Neither the child nor the compiler-private
carrier is a public result or a new public tuple ABI.

## Single Outward Target

The separate numeric-pair admission predicate now also accepts exact unqualified
f32/f64 owners with an exact i64 child. Existing exact i64-owner numeric-child
and bool/numeric routes retain their prior authority and layouts. Floating
owners with floating children and other numeric/numeric orientations remain
unproved; generic carrier installation does not grant their admission.

A child is already prefix-defined, unsealed and absent from the entering scope.
The owner is prefix-owned or the exact delegated target. One outward target,
readonly entering values, both source write orders, current/retained versions,
private scope and seals remain mandatory. Third/unrelated entering writes,
qualifier/resource fields, hidden ordinary logical work, child leaks and
partial exits veto the complete candidate atomically.

## Exact Private Transport

The compiler-private record has actual semantic fields:

```text
owner: f32 or f64
child: i64
```

Inactive helpers return width-specific positive floating zero and i64 zero.
Both typed projections read the same selected owned record. No sign/payload
bit is borrowed, numeric narrowing/packing or source replay is added.
The observed bridge adds no owned aggregate allocation/drop calls, common
effect authority, computed-print grants or backend-specific transport table.

Whole-forest limits stay 64 nodes, 256 expressions, 32 logical edges,
16 statements per body/suffix, depth eight and 31 captures plus a gate.
Both widths prove the seven nodes / seven expressions / zero logical edges
boundary, adjacent rejection, 16/17 statements, 31/32 captures and depth 7/8.

## Source And Probes

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_float_i64_private_pair.ns)
uses identity float initialization and a private integer initializer that chooses
the signed extrema or marker 96 from source comparisons. Current integer signs
control floating sign negation, while private integer updates use complement
arithmetic. Both write orders, nested selections, a shared continuation, seals,
unused work and caller continuation remain in the real source.

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_float_i64_oracle.rs)
replays IEEE owner words and full-width integer updates without compiler carrier
logic. Twelve raw words per width cross both gates: signed zeros, both smallest
and largest subnormals, finite values, infinities and quiet/signaling NaN payloads.
Pair snapshots deliberately differ from some final owners; a correct final bool
alone cannot prove current-value transport.

The native harness observes both actual after-work arguments and the actual
final answer-work owner. Floating observations bitcast the real callee argument,
not an oracle-manufactured result; no production hook is added. Original work,
joins, real traps and actual callback-state forwarding remain intact, with
output canaries. Late unused/answer faults carry private integer marker 96.
Failure prefixes use source-event positions, not searches through mixed field
and marker outputs, and failing arguments trap before callee entry.

Reference and CLI inputs are four exactly representable finite values per width.
NaN payload and raw subnormal evidence belongs to native probes, not decimal
parsing or reference-state display.

## Executed Acceptance

The 2026-10-12 serial scoped cohort contains **343 distinct tests**:
264 compiler unit/artifact tests, 48 native bridge tests, one CLI test spanning
both widths, three host failure-policy tests and 27 tensor tests. Sixteen tests
are new. Repaired/repeated runs and the complete 328-test predecessor are
not added to the current count. The current CLI receipt runs the new reverse
two-width matrix; the predecessor's forward i64/float CLI test is not counted
as rerun.

The new matrix runs 192 successful native processes and
36 fatal native processes across both widths and normal/reversed inventories.
It proves 648 accepted callbacks, actual IEEE owner/pair words, full-width
private integer extrema, final owners, source order and canaries.
Nine selected original proof stages per width trap before callee entry,
including marker-valued late failures. Reference fault cleanup does not
make native process traps recoverable.

Eight CLI images, four per width, cover fresh build, cache hit and two successive
source-free restores. They prove 72 CLI event traps and 744 accepted callbacks,
including healthy reentry after each failure. Only the last accepted open/event
state is published. Later event/close requests and completion are absent after
fatal events. Executable, native objects, packaging, envelope and lifecycle
identity stay unchanged. Each image and its restoration input container remain
byte-identical after use; diagnostic-bearing fresh/cache container bytes need
not match.

Existing i32/i64/float-ancestor, dual-i64 and forward i64/float native,
five-child-type event failure/budget and helper-entry regressions remain
included. The rebuilt ordinary CLI reports 2593 clean drift checks / 18034
required patterns and clean coverage/hierarchy/task-card lineage.
It checks six source-equal fields and six complete predecessor hashes.
The selected cell remains
`active/99`; this is not broad persistent application-session closure.

Reproduce from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_i32_ancestor_private_bool return_i64_ancestor_private_bool private_event_budget private_event_failure helper_entries_share_exact_budget helper_entries_reject_before_body helper_entries_skip_unselected_paths helper_entries_use_unsigned_limits helper_entries_count_loop_free_roots helper_entries_include_scoped_iterations return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_float_i64_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
```

The tensor retains all six complete 328-test predecessor fields after their
i64-float markers:

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `6732b74f72e628ce0a0692b5bbc82eca2cb6b51269b0fea29f3c64ff6b1cc8f0` |
| `next_step` | `79f50ee15bc8a3b91e6a28cf95a70940b2b03c962e5b334286a9caa01429e232` |
| `blocker` | `a2f1ef6807901b6ffa192032f5a8583aa91e61361710346b01770629ffd4c8dc` |
| `next_action` | `756fd6fd827f5201a70de284c53fb5239c0b6cf6dd73f52b0f0d15e790ff524d` |
| `validation_command` | `363a616b46f0f6430053322e70c64863f39173c6043a15defad1bed16cb9edaa` |
| `expected_artifact` | `8c62e3385ff7dfea04fe7addfe7eff8191bcab25297dead6f554d23448bf733c` |

## Remaining Boundary

Other numeric/numeric orientations and resource transport remain separate.
The next bounded proof is a same-width floating owner/private child pair:
exact f32/f32 and f64/f64. Keep one outward target, readonly entering values,
seals and original budgets. Prove both independently carried IEEE fields,
current reads, source-order traps and source-free identity before admitting
cross-width floating pairs or generalized resource fields.

This is a macOS CPU receipt, not new GPU/Linux/Windows execution, performance,
formal memory safety, self-hosting, full-workspace or strict-Clippy acceptance.
Broader application-session closure and public ABI stabilization remain open.
