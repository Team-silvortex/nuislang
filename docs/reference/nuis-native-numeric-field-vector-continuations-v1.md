# Native Numeric Field Vector Continuations V1

Historical numeric-only checkpoint. The subsequent
[mixed bool field-list proof](nuis-native-bool-field-vector-continuations-v1.md)
admits exact private bool children under the unchanged numeric Target.
This receipt retains its original cohort, boundaries and predecessor hashes.

This bounded CPU proof follows the
[complete 452-test predecessor](nuis-native-retained-owner-three-field-continuations-v1.md).
The private certificate and carrier now use a numeric field list rather than
an exact two-child recipe. A new Nuis fixture executes one authorized owner
beside three independently initialized private children.

## Field List And Authority

An existing opaque Target remains the only outward owner permission. Exactly
two known writes including that owner still take the unchanged old pair route.
For the field-list route, remove that already authorized owner from the write
evidence and individually certify every remaining child: defined in the current
prefix, absent from the entering scope, mutable, unsealed and exact unqualified
i32/i64/f32/f64. Owner and child types are not inferred from arity.

The original 64-node write scan bounds distinct evidence; it no longer rejects
a third child by count. The representation separately admits 2..63 children,
keeping owner plus children within 64 scalar fields. This is a conservative
capacity bound, not source permission or a promise that every such arity can
execute. Real source admission retains the original 16-statement, 64-node,
256-expression, 32-logical-edge, depth-eight and 31-plus-gate capture budgets.
In particular, high-arity all-defined structural preparation is not a live
application or backend acceptance claim.

The same field-driven assembler produces typed neutrals and distinct
owner/child_N projections. No new codec, native backend recipe, public ABI,
allocation or drop was added. The original forest is walked once, with no
continuation copied into both arms. Unrelated entering writes, undefined or
sealed children, qualified/unsigned/resource/bool fields, hidden leaves,
leaks, partial exits and missing root owner authority remain atomic vetoes.

Older count-only sibling negatives now use a sealed extra binding. Their
entering-scope, qualifier, hidden-work, leak and module/name-atomicity checks
remain; an otherwise eligible third numeric child is no longer rejected just
for being third.

## Independent Four Field Computation

[The fixture](../../tools/nuisc/tests/native_application_bridge/return_numeric_field_vector.ns)
uses independent i64 owner, i32 child, original-width f32/f64 child and another
full-width i64 child. The initial selection updates children only and retains
the owner. Its arms use different write orders; nested selections retain one
child while advancing others. A shared after_work receives all four current
fields before parent_work computes with all three children. A later child
update and owner computation consume the new version, not its initial seed.

[The oracle](../../tools/nuisc/tests/native_application_bridge/return_numeric_field_vector_oracle.rs)
uses independent signed complements and IEEE sign XORs, not a compiler carrier.
Its raw matrix is 3 owner extrema * 4 i32 values * 12 IEEE payloads * 2 other
i64 extrema * 4 gate pairs: 1152 cases per width/inventory. Source-event
positions, not word values, identify failures. The late marker snapshot is
`(70001, -70002, 96, -70002)`; its final owner is `-70002`, colliding with a
diagnostic sentinel without being confused with a successful return.

[Reference acceptance](../../tools/nuisc/tests/native_application_bridge/return_numeric_field_vector.rs)
also changes only the third input and requires a different computed owner.
Native observers read all four actual callee parameters and the final owner;
they do not inject expected outputs or replace source calculations. Actual
accepted state is forwarded into the next callback, including the other seed.

Raw reference/native proof covers signed zero, subnormals, infinities and
quiet/signaling NaNs. Finite-decimal CLI proof remains separate. The original observe
function still returns bool and exports no private child.

## Unit And AOT Boundaries

[Four unit families](../../tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_numeric_field_vector_tests.rs)
test owner updates and retention, reversed/split/nested writes, exact field
types and neutrals, distinct projections, collision handling and atomic
authority vetoes. Structural preparation checks all 256 exact ordered numeric
quadruples, not 256 live backend recipes. Separate all-defined arities
0/1/2/3/16/31/63/64 check representation capacity independently of incoming
capture count.

Original work accepts exactly five nodes/expressions with zero logical edges;
either adjacent smaller budget rejects. Complete 64/65 forests, 16/17 bodies,
31/32 captures and depth 7/8 keep their boundaries. The preceding pair and
two-child tests remain in the selected cohort.

The shared AOT policy checker requires the supplied expected four-field layout,
not a layout matched against itself. Guard-first transitive Effect order,
default/empty print grants, removed-order vetoes, no owned allocator/drop and
no live floating conversion or arbitrary i64 narrowing remain required.
This is not formal memory safety or a new source-function eligibility grant.

## Executed Acceptance

The 2026-10-12 serial macOS CPU cohort has **463 distinct tests**:
330 compiler, 102 native bridge, one two-width CLI workflow, three host
boundary and 27 tensor tests. Twelve tests are new: four unit, one AOT, two
reference/oracle, four actual native and one CLI. Focused reruns are not added.
The previous retained-owner CLI remains historical rather than rerun; its
compiler/reference/native families remain included.

The new fixture contributes 4608 successful native processes and
52 fatal native processes, with 13928 accepted callbacks. Normal and reversed
inventories check current/retained fields, raw IEEE words, final owner,
all thirteen selected failure prefixes, actual-state forwarding, canaries and
the absence of owned allocation/drop.

Eight fresh/cache/two successive source-free images contribute
104 CLI event traps and 1096 accepted callbacks, including healthy reentry.
Sources, project manifest and local cache are absent before both restorations.
Public state/stdout, exact executable/native objects and packaging/envelope/
lifecycle identity are checked. Fatal events publish only last accepted
open/event state, with no later event, close or completion. Used images and
restoration containers remain byte-identical.

Reproduce from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_i32_ancestor_private_bool return_i64_ancestor_private_bool private_event_budget private_event_failure helper_entries_share_exact_budget helper_entries_reject_before_body helper_entries_skip_unselected_paths helper_entries_use_unsigned_limits helper_entries_count_loop_free_roots helper_entries_include_scoped_iterations return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_numeric_field_vector --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
```

The ordinary CLI is rebuilt separately. It must expose six source-equal fields,
six complete predecessor hashes, 2805 clean drift checks / 19729 patterns and
clean coverage/hierarchy/task-card lineage. The selected cell stays `active/99`,
not broad application closure.

The six complete 452-test predecessor fields remain verbatim after the
retained-owner history markers:

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `b6b97796636d25af01f7968ff004e9fc419481ebac921158901140fb2caa35fa` |
| `next_step` | `c8dd6823c07dc7ecea5e7f69ed97c171c279686525a16677343bf392016f0de9` |
| `blocker` | `ff71634cf11743f0d1a5c26bf1299bc30219ab0878dff55865a59f1c437ce555` |
| `next_action` | `9892ab2dbe683957a072cc83de35d7b424fe264e520eed24e1a9785c4d804cc7` |
| `validation_command` | `4923adecc94af52fc8fa877d1906852fd6eed6db06b99c419b99ffcc3a6ff1fc` |
| `expected_artifact` | `c29c5cc7b59d08263b19a82d038df50896c5056b9e3dc6e965ed40d45f063daf` |

## Remaining Boundary

A separate next proof should admit exact private bool fields beside numeric
children without broadening numeric outward-owner authority, the old pair
route or resource eligibility. Reuse the field list and typed neutral/projection
mechanism; preserve original logical-edge/work budgets and test selected bool
updates, retention, source order, raw words and source-free publication.

Other field types, resources, broader roots/function eligibility, mixed exits
and whole-application closure remain separate. This receipt is not fresh
GPU/Linux/Windows execution, measured performance, full-workspace/strict-Clippy
acceptance or completed self-hosting.
