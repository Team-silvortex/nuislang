# Native Bool Field Vector Continuations V1

Historical one-bool checkpoint. The subsequent
[two-bool sibling proof](nuis-native-two-bool-field-vector-continuations-v1.md)
executes independently seeded boolean siblings and current-value reads.
This receipt retains its original cohort, boundaries and predecessor hashes.

This bounded CPU proof follows the
[complete 463-test predecessor](nuis-native-numeric-field-vector-continuations-v1.md).
The field list now admits exact private bool children beside numeric children
without changing the authorized outward owner or adding a backend recipe.

## Admission And Authority

The child predicate is exact bool or the existing exact i32/i64/f32/f64
predicate. Both lexical certification and transport preparation use it.
The owner independently remains numeric, mutable and already delegated by
the opaque Target. Arity, a boolean seed or a child write cannot grant owner
authority. The old two-write pair route and root discovery are unchanged.

Every child must still be prefix-defined, absent from entering scope and
unsealed. References, optional/generic types, unsigned/resource fields, type
changes, unrelated entering writes, hidden leaves, scope leaks and partial
exits remain atomic vetoes. Previous numeric tests now distinguish admissible
exact bool children from the still-rejected bool owner; qualifier and seal
vetoes are not removed. Older numeric-owner/bool-child count-only sibling
negatives now use sealed siblings instead. A two-bool structural source case
also verifies updated-sibling logical reads; independently seeded two-bool
native/CLI execution remains the next separate proof.

The existing record assembler already supplies false for inactive bool
fields and retains each typed current value in active arms. Distinct
owner/child_N projections are used, not a numeric boolean codec. No public
ABI, allocator/drop, general function permission or resource transport was
added. The original forest and shared suffix are still walked once.

## Logical And Original Budgets

Four unit families cover selected logical updates, reversed/split/nested
writes, child retention, exact bool projection and independent false
neutrals, collision handling and owner/scope/type/seal rejection. Original
budgets remain shared across the numeric and boolean computations.

A balanced logical RHS tests 32/33 logical edges. Exact work is five statement
nodes and 69 expression visits for the 32-edge case; one fewer node,
expression or edge rejects. Complete 64/65 forests, 16/17 arm bodies,
depth 7/8 and 31/32 captures are separately tested. Qualified bools are not
admitted by name alone. The preceding pair, two-child and numeric-list
families remain in the selected compiler/native cohort.

The AOT checker uses the supplied exact layout
`{owner:i64;child_0:i32;child_1:FLOAT;child_2:bool}`, not a self-matched
layout. Guard-first transitive Effect order, removed-order vetoes, default/
empty print grants, no owned allocation/drop and width-safe original floating
values remain required. This is not formal memory safety.

## Independent Mixed Computation

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_bool_field_vector.ns)
initializes the i64 owner, i32 child, original-width f32/f64 child and bool
child from four independent inputs. Initial arms retain the owner and use
different write orders. Boolean AND/OR rebindings read their preceding
version; nested selections retain or toggle children. A shared after_work
receives all four current fields before a parent calculation consumes them.
A later boolean toggle feeds the next calculation, not the initial seed.

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_bool_field_vector_oracle.rs)
uses signed complements, IEEE sign XORs and boolean operations, not the
compiler carrier. Both true and false bool seeds, all gate pairs, independent
integer extrema and twelve IEEE payloads form 1152 cases per width/inventory.
Raw native coverage includes signed zero, subnormals, infinities and quiet/
signaling NaNs; finite-decimal CLI coverage is separate.

Reference checks change only the bool seed and require a different computed
owner. They require both retained and toggled boolean snapshots. Inactive
AND and OR RHSs carry zero divisors and must not execute; selected RHSs use
the existing guarded predicate lowering and original edge budget.

Native observers read actual callee parameters, including an i1-to-i64
observation of the boolean, plus the final owner. They do not inject oracle
values or replace source computations. The late raw snapshot is
`(70001, -70002, 96, 1)`; the final owner is `-70002`.
Source-event positions, not matching numeric sentinels, identify failure
prefixes. Actual accepted state, including the bool seed, is forwarded to
the next callback.

## Executed Acceptance

The 2026-10-12 serial macOS CPU cohort has **474 distinct tests**:
335 compiler, 108 native bridge, one two-width CLI workflow, three host
boundary and 27 tensor tests. Twelve tests are new: four unit, one AOT,
two reference/oracle, four actual native and one CLI. Focused reruns are
not added to this count. The previous numeric-list CLI is historical,
not rerun in this cohort; its compiler/reference/native tests are included.

The new fixture contributes 4608 successful native processes and
52 fatal native processes, with 13928 accepted callbacks. Normal and
reversed inventories check exact field/owner words, current/retained
booleans, skipped poisoned RHSs, thirteen selected event failure prefixes,
actual-state forwarding, output canaries and absence of owned allocation/drop.

Eight fresh/cache/two successive source-free images contribute
104 CLI event traps and 1096 accepted callbacks, including healthy reentry.
Before both restorations, source, project manifest and local cache are
absent. Public state/stdout, unchanged executable/native objects, packaging,
envelope and lifecycle identity are checked. Fatal events publish only last
accepted open/event state, with no later event, close or completion.
Used images and restoration containers remain byte-identical.

Reproduce from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_i32_ancestor_private_bool return_i64_ancestor_private_bool private_event_budget private_event_failure helper_entries_share_exact_budget helper_entries_reject_before_body helper_entries_skip_unselected_paths helper_entries_use_unsigned_limits helper_entries_count_loop_free_roots helper_entries_include_scoped_iterations return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_bool_field_vector --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
```

The ordinary CLI is rebuilt separately. It exposes six source-equal fields
and six complete predecessor hashes, with 2828 clean drift checks / 19858
required patterns, clean coverage, hierarchy and task-card lineage.
The selected cell stays `active/99`, not broad
application closure.

The complete six 463-test predecessor fields remain verbatim after the
numeric field-vector history markers:

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `8abf398dfe6f93fd3cb7b6add9c1fa38fa970d0028130bf1fa07bed06464429d` |
| `next_step` | `615e1646e300e2edb7b741dfca3c925ae81cd7f9d8dfa561dea85b0468c65f0f` |
| `blocker` | `84ab3790a09157e8dcf8e7baef31893db5ff2131c3f708f1736809ee2c3bd6fd` |
| `next_action` | `7a0c884ef3b86a5dc149c08da00caa2615e7edafdcd753e43dc71249e70cd114` |
| `validation_command` | `698947771f9f9fa148c346743de25e5ce9871be06e5eb9ec7046773c1a4c4931` |
| `expected_artifact` | `89617b23dcb2ed337d9013097f7b6b306e35ca21210ddc2312780af6f42a1fab` |

## Remaining Boundary

The next narrow proof should execute two independently seeded private bool
children beside numeric fields. Test a later bool RHS reading the already
updated sibling, opposite logical polarities, nested retention, poisoned
skipped RHSs and exact raw projections without adding bit packing or owner
permission. Keep all original work/capture bounds and source-free publication
checks. Floating-owner field-list live coverage remains a separate gap.

Resources, broader roots/function eligibility, mixed exits and whole-
application closure remain separate. This receipt is not fresh GPU/Linux/
Windows execution, measured performance, full-workspace/strict-Clippy
acceptance or completed self-hosting.
