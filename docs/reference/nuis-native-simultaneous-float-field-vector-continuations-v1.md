# Native Simultaneous Float Field Vector Continuations V1

This bounded CPU proof follows the
[complete 396-test predecessor](nuis-native-cross-width-field-vector-continuations-v1.md).
One numeric owner now has actual live evidence beside simultaneous,
independent i32, f32, f64 and two private bool children. Production field
admission/transport, the shared AOT checker and native driver are unchanged.

## Current Data Across Widths

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_multi_float_field_vector.ns)
has an i64 owner and five private children. Its public state has 26 words,
open has 25 arguments and event has 19 arguments. Six public seeds are
inputs, not exported versions of the private children.

Both selected arms update the f32 child before wide_work consumes that
current value and a bool sibling to select an original-width f64 sign
toggle. Nested stages update/reuse the wide value; later peer/wide updates
feed another owner computation. after_work observes all six current fields
before parent_work consumes them. Initial child-only selections retain the
already authorized owner. AND/OR retain poisoned inactive RHS proofs.

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_multi_float_field_vector_oracle.rs)
uses signed integer complements, separate IEEE sign XORs and boolean
logic. Per inventory it runs 12 f32 words * two companion f64 profiles *
two i64 owner extrema * two i32 extrema * four bool pairs * four gate
pairs, plus eight masking and four nested witnesses: 1548 cases.
Companion f64 words use offsets 1 and 5 in their own width's dictionary;
this is not a full IEEE Cartesian product. Both floats include signed
zero, subnormal, finite, infinity and quiet/signaling NaN words.

A deliberate source mutation replaces current b with entering narrow_seed
in wide_work. Finite witnesses change the computed owner and public
result while preserving selected calls. NaN/zero wide witnesses instead
have identical owner, result and calls but different private f64 sign
bits. Both original and stale source versions are compiled and executed
under normal/reversed inventories. Observers dump actual callee fields,
not expected values or a replaced source computation.

The late raw snapshot is `(70001, -70002, 93, 96, 1, 1)` and final
owner is `-70002`. Floating words collide with source print markers;
failure cuts use source-event positions. The native driver forwards
actual previous callback output, requires canaries, and checks all
seventeen selected failure prefixes before the failing callee.

## Authority And Shared Limits

Two new unit families require five separately prefix-defined, mutable,
non-entering exact scalar children and one numeric Target. Qualified,
sealed, undefined, unsigned/resource children and a bool owner remain
atomic vetoes; removing genuine local owner authority remains rejected.
Higher child count is representation capacity, not source permission.

Six neutral fields and projections remain independently typed:
`{owner:i64;child_0:i32;child_1:f32;child_2:f64;child_3:bool;child_4:bool}`.
The f32 and f64 zero expressions are different NIR kinds; two bool fields
are distinct false entries. The six-field 31/32 capture boundary is
checked; original forest/statement/expression/depth/logical limits are
replayed through the shared compiler cohort without reset or expansion.

The supplied exact AOT layout, guard-first transitive Effects,
removed-order/default/empty-grant vetoes, safe narrowing, no live float-
width conversion and no owned allocator/drop remain asserted by the
unchanged checker/assembler. This is not formal memory safety.

## Executed Acceptance

The 2026-10-13 serial macOS CPU cohort has **386 distinct tests**:
349 compiler, five new native/reference, two CLI, three host boundary
and 27 tensor tests. Ten are new: two units, one AOT, two reference,
three actual native and two CLI. Focused reruns are not double-counted.

Only test registration, fixtures, oracles, verification and documentation
change in this checkpoint. Shared production admission/transport,
AOT checker and native driver are unchanged, so older native families and
CLI workflows remain historical rather than freshly replayed/countable.
Their structural/AOT/compiler checks remain in the selected cohort.
All six complete predecessor fields/hashes remain preserved.

All thirty changed Rust files pass scoped formatting checks. Whole-workspace
formatting still reports four unchanged historical files; it is not green.

The new family contributes 3112 successful native processes and
34 fatal native processes, with 9404 accepted callbacks. This includes
3096 original and sixteen stale-source successes under both inventories.
Actual independent f32/f64 and bool field words, final owner, real-state
forwarding and complete flushed failure prefixes remain exact.

Four fresh/cache/two successive source-free images contribute
68 CLI event traps and 556 accepted callbacks, including healthy reentry.
Each image runs eighteen finite cases plus seventeen failure/reentry
cases. Raw IEEE/NaN claims belong to reference/native tests, not the
finite CLI parser. Source, project manifest and cache are removed before
both restores. Public state/stdout, binary/native-object bytes,
packaging/envelope/lifecycle and input container identity remain exact.
Failed events expose only last accepted open/event state, with no later
event, close or completion; a fresh invocation remains healthy.

Reproduce from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_multi_float_field_vector --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_multi_float_field_vector --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
```

The ordinary CLI is rebuilt separately. It exposes six source-equal fields
and six complete predecessor hashes, 2905 clean drift checks / 20345
required patterns, clean coverage/hierarchy/task-card lineage and the
same mainline coordinate. The selected cell stays `active/99`.

The complete six 396-test predecessor fields remain verbatim after the
cross-width field-vector history markers:

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `05ef9422c066e39a8003d7e55178d49a95433f8b3a5addf8a6ea068893daebd0` |
| `next_step` | `57f6fd8c3979e9c2b6cd95b7130a739e97b752e4d974f5011269e3f5cb3991a1` |
| `blocker` | `f21455355f3474aa4f284592c677179a2230f05ec731d10f1e1fcf0a743b4aa4` |
| `next_action` | `c25ebfc3c9476797b8ae2d62614d9c89f7f52405abda78de44f7e6ea1ed36aa7` |
| `validation_command` | `87ca32789669577e71cdb8621ec6cbd46bc06ec5aa6c15e5a4fcfbc74a352b30` |
| `expected_artifact` | `5f6be6fc5569bfb8473d8aa04b9be2e10331e6dfa2218b22fd5aad0dd4de9013` |

## Remaining Boundary

Next separately prove an actual source-selected early return within a
mixed private-field update. Exited paths must suppress later effects,
proofs and neutral publications; continuing paths must preserve their
authorized owner and current/retained children. Start with a bounded
exact bool source exit, not a general child export or new numeric root.
Reuse the existing controller/field interfaces and original shared limits.

Resources, broader roots and application-wide closure remain separate.
This receipt is not fresh GPU/Linux/Windows execution, measured
performance, full-workspace/strict-Clippy acceptance or completed self-hosting.
