# Native Private I64 Float Continuations V1

This is the frozen 328-test i64/float checkpoint. The subsequent
[float/i64 proof](nuis-native-private-float-i64-continuations-v1.md) also admits
exact f32/f64 outward owners beside a private i64, retaining one outward target.

The persistent application-session CPU path carries an exact i64 outward
ancestor beside an exact private f32 or f64 child through bounded terminal mixed
selections and sequential continuations. This extends the
[dual-i64 checkpoint](nuis-native-private-dual-i64-continuations-v1.md)
with independently typed fields, not floating-to-integer numeric conversion.

The source helper remains **bool-returning**. Only the i64 owner is exported
to its enclosing return prefix. The child and compiler-private carrier are not
public results, a new public tuple ABI or extra outward write permission.

## Single Outward Target

Numeric/numeric admission requires an exact unqualified i64 owner and an exact
i64, f32 or f64 child. The predicate is separate from generic carrier installation.
Float outward owners still admit only private bool children. Existing bool/numeric
and numeric/bool routes retain their prior layouts and authority.

The child must already be prefix-defined, unsealed and absent from the entering
scope. The owner must be prefix-owned or the exact delegated target. One target,
readonly entering values, both source write orders, retention/current reads,
private scope and seals remain mandatory. Third or unrelated entering writes,
qualified/resource fields, hidden ordinary logical leaves, child leaks and
partial exits veto the whole candidate atomically.

## Exact Private Transport

The existing owned record now admits:

```text
owner: i64
child: f32 or f64
```

Inactive helpers return independent i64 zero and width-specific positive
floating zero. Both typed projections read the same actual selected record.
No payload/sign bit is borrowed, numeric narrowing/packing or replay is added.
The observed bridge adds no owned allocation/drop calls, extra common effect
authority, literal-print grants or backend-specific transport table.

Original whole-forest limits remain 64 nodes, 256 expressions, 32 logical edges,
16 statements per body/suffix, depth eight and 31 captures plus one gate.
Both widths prove the seven nodes / seven expressions / zero logical edges
boundary, adjacent rejection, 16/17 statements, 31/32 captures and depth 7/8.

## Source And Probes

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_i64_float_private_pair.ns)
initializes an i64 owner to 96 for zero seeds or the signed maximum otherwise.
Current private floating comparisons control integer complement updates.
Both source write orders, nested zero selections, a shared continuation,
seals, unused work and caller continuation are retained. Floating updates use
unary sign negation, not payload-changing arithmetic.

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_i64_float_oracle.rs)
replays integer updates and IEEE sign-bit changes without compiler transport logic.
Twelve raw seeds per width cross both gates: signed zeros, both smallest/largest
subnormals, finite values, both infinities and quiet/signaling NaN payloads.
The source distinguishes branch-current pair values from the final owner:
source order can carry the integer minimum with a floating value while the
later continuation restores the integer maximum.

The native test observes both actual after-work arguments and the actual final
answer-work owner. Float observation is a bitcast of the real callee argument;
there is no oracle-injected value or production observation hook. The shared
harness keeps original work, joins and traps, forwards actual callback state
and checks output canaries. Real unused/answer faults carry an integer field
equal to marker 96; failure prefixes use source-event positions, not searches
through mixed field/marker outputs.

Reference and CLI inputs use four exactly representable finite values per width.
NaN payload and raw subnormal acceptance belong to native word probes, not
decimal parsing or reference-state display.

## Executed Acceptance

The 2026-10-12 serial scoped cohort contains **328 distinct tests**:
255 compiler unit/artifact tests, 42 native bridge tests, one CLI test spanning
both widths, three host failure-policy tests and 27 tensor tests. Sixteen
tests are new. Repeated/repaired runs and the complete 315-test predecessor
are not added to the current count. The current CLI receipt runs only the new
two-width matrix; the predecessor's three other CLI tests are not counted as
rerun.

The new matrix runs 192 successful native processes and
36 fatal native processes across both widths and normal/reversed inventories. It proves
648 accepted callbacks, actual integer/IEEE pair values and final owners,
source effect order and canaries. Nine selected original proof stages per
width trap before callee entry, including marker-valued late failures.
Reference fault cleanup does not make native process traps recoverable.

The new CLI matrix checks eight images, four per width: fresh build, cache hit
and two successive source-free restores. It proves 72 CLI event traps and
744 accepted callbacks, including healthy reentry after each failure. Only the
last accepted open/event state is published. Later event/close requests and
completion are absent after fatal events. Executable, native objects, packaging,
envelope and lifecycle identity stay unchanged. Each image and its restoration
input container remain byte-identical after use; diagnostic-bearing
fresh/cache container bytes need not match.

Existing i32/i64/float-ancestor and dual-i64 native, five-child-type event
failure/budget and helper-entry regressions remain included. The rebuilt
ordinary CLI reports 2572 clean drift checks / 17876 required patterns, clean
coverage/hierarchy/task-card lineage, six source-equal fields and six complete
predecessor hashes. The selected cell remains `active/99`; this is not broad
persistent application-session closure.

Reproduce from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_i32_ancestor_private_bool return_i64_ancestor_private_bool private_event_budget private_event_failure helper_entries_share_exact_budget helper_entries_reject_before_body helper_entries_skip_unselected_paths helper_entries_use_unsigned_limits helper_entries_count_loop_free_roots helper_entries_include_scoped_iterations return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_i64_float_private_pair --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
```

The tensor retains every complete 315-test predecessor field after its
dual-i64 marker:

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `446a04af096fda2bed2b6ee853a5d37a4422be6feb8199765f80e00c53d42a04` |
| `next_step` | `cd081643202ca7075e55ef12cc4b0b56c9cdfb2ba27523cc9c479a1538450bc5` |
| `blocker` | `9ff281d69490bbaa8782fae69535ae763080b65c73b32ac2ec6ba587fbb2c80a` |
| `next_action` | `a61693916bf2189df7300564b7af583de78704a35a8d89b7670e5fdb137f41d7` |
| `validation_command` | `8afefc04a5a9b96b3ec7a14f269c32b2b8da09a13c11e5f30d315ec78eee2858` |
| `expected_artifact` | `5975600647a01068424324242c8c32e14359c18feafdd2204d7fabfe366962c4` |

## Remaining Boundary

Other numeric/numeric orientations and resource transport remain separate.
The next bounded proof is an exact f32 or f64 outward owner beside an exact
private i64 child, retaining one outward result, readonly entering values,
seals and original budgets. Prove actual IEEE owner bits and full-width
private integer values through native order/trap and source-free identity
checks before broadening the admission predicate.

This is a macOS CPU receipt, not new GPU/Linux/Windows execution, performance,
formal memory safety, self-hosting, full-workspace or strict-Clippy acceptance.
Broader application-session closure and public ABI stabilization remain open.
