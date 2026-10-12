# Native Two Bool Field Vector Continuations V1

This bounded CPU proof follows the
[complete 474-test predecessor](nuis-native-bool-field-vector-continuations-v1.md).
The existing exact-scalar field list now has executed evidence for two
independently seeded private bool children beside numeric children. No
production admission, transport, backend recipe or public ABI was changed.

## Current Sibling Computation

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_two_bool_field_vector.ns)
takes independent i64 owner, i32 child, original-width f32/f64 child and two
boolean seeds. Its state has 23 words, open has 22 arguments and event has
17 arguments. These public boolean seeds are inputs, not exported private
child versions.

Initial arms retain the authorized numeric owner. One arm flips c before
`d && peer_work(c, ...)`; the other flips d before
`c || bool_work(d, ...)`. The RHS must read the new sibling version.
Selected calls keep source order; skipped RHS arguments contain zero
divisors and must not execute. Nested stages retain one boolean while
toggling the other. A shared after_work observes all five fields before
parent_work computes with them; a later d update feeds a second computation.

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_two_bool_field_vector_oracle.rs)
uses signed complements, IEEE sign XORs and boolean operations, not the
compiler carrier. Its raw matrix is 3 owner extrema * 4 i32 values * 12
IEEE payloads * 4 independent boolean pairs * 4 gate pairs: 2304 cases per
width/inventory. It includes all four current boolean snapshots. Raw
reference/native signed zero, subnormal, infinity and quiet/signaling NaN
coverage is separate from finite-decimal CLI evidence.

Changing either boolean seed independently changes a selected computed
owner. Two deliberate stale-sibling source mutations read the entering
seed instead of the updated private value. Reference execution requires
different fields, owner and public result while preserving the selected
call sequence, under normal and reversed inventories. This verifies that
the current-value checks can detect stale reads, not just missing calls.

## Authority And Shared Bounds

Four unit families cover both dependency directions, reversed/split/nested
updates, retention, distinct false neutrals and child_2/child_3 projections.
Each bool must remain prefix-defined, mutable, absent from entering scope
and exact unqualified bool. Entering or sealed siblings, qualifiers,
unsigned/resources, type changes, hidden work, partial exits and absent
owner authority remain atomic vetoes. The outward Target stays numeric;
the existing two-write pair route and root discovery are unchanged.

Two logical roots share one original budget. The 16-edge plus 16-edge case
consumes exactly six statement nodes, 70 expressions and 32 edges; reducing
any one budget rejects. The 17-plus-16 case rejects, covering
32/33 shared logical edges without resetting the budget between updates. Original
16/17 bodies, depth 7/8 and 31/32 captures are tested. The older complete
forest and generic capacity proofs remain included through shared field
families; representation capacity is not source permission.

The exact AOT layout is
`{owner:i64;child_0:i32;child_1:FLOAT;child_2:bool;child_3:bool}`.
The shared checker verifies that supplied layout, guard-first transitive
Effects and removed-order vetoes, default/empty print grants, no owned
allocator/drop and no live float-width conversion or arbitrary narrowing.
Boolean fields are independently typed; no bit packing or codec is added.
This is not formal memory safety or whole-application acceptance.

## Executed Acceptance

The 2026-10-12 serial macOS CPU cohort has **402 distinct tests**:
340 compiler, 30 related native bridge, one two-width CLI workflow, one
finite-input sensitivity audit, three host boundary and 27 tensor tests. Thirteen tests are new: four unit, one
AOT, two reference/oracle, four actual native and two CLI. Focused reruns
are not added to the count.

Production admission and transport are unchanged, so native replay is
focused on the private-fields, retained-owner, numeric-list, one-bool and
two-bool field families. Unrelated older pair/event-budget native families
and the prior one-bool CLI receipt remain historical, not rerun or counted
as fresh acceptance. The complete predecessor tensor fields are preserved,
not reduced to this narrower replay selection.

The new fixture contributes 9216 successful native processes and
60 fatal native processes, with 27768 accepted callbacks. Normal and
reversed inventories observe all five actual callee parameters, including
separate i1-to-i64 observations of both booleans, plus the final owner.
They do not inject expected values or replace source computations.
Fifteen selected failure prefixes, actual accepted-state forwarding,
canaries and absence of owned allocation/drop are required.

The late raw snapshot is `(70001, -70002, 96, 1, 1)` and its final owner
is `-70002`. Source-event positions identify failures despite marker
collisions. Fifteen selected traps stop before the failing callee and
exclude subsequent calls and publication.

Eight fresh/cache/two successive source-free images contribute
120 CLI event traps and 1416 accepted callbacks, including healthy reentry.
Each image runs 34 finite cases covering all boolean pairs, both dependency
directions, all outer gates and extra nested/current-value updates.
Source, project manifest and local cache are absent before both restores.
Public state/stdout, executable/native-object bytes and packaging/envelope/
lifecycle identity remain exact. Failed events publish only last accepted
open/event state, with no later event, close or completion. Used images
and restoration containers remain byte-identical.

Reproduce from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_two_bool_field_vector --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
```

The ordinary CLI is rebuilt separately. It exposes six source-equal fields
and six complete predecessor hashes, with 2849 clean drift checks / 19992
required patterns, clean coverage, hierarchy and task-card lineage.
The selected cell stays `active/99`, not broad application closure.

The complete six 474-test predecessor fields remain verbatim after the
bool field-vector history markers:

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `d0185167ac0f8f286c8a7d85ec6008c15032a3cfb0bb441ee080a5d5fec40f38` |
| `next_step` | `8f8239a751ee641b58cf7080c480d857f7b94e07d593a51c0888b6960c5cd4bf` |
| `blocker` | `8646ee1a57dad37f30f9efbb2d40cdea5c0f844688aae8ed77c7ffefc66db721` |
| `next_action` | `c7d0d8dc104348858ff597e6e93996c15bd669cfe57c805d928795577e4bded5` |
| `validation_command` | `abdebde03c68a6b36d699bf02deff8775fef0800049b6a694f1d2e4e68be97da` |
| `expected_artifact` | `c71da706184dea9d1ad7fe9f09a2a1295d45746e58e4d116eeb0f2b86a97cae9` |

## Remaining Boundary

Next execute an original-width floating outward owner with the mixed
numeric/boolean private field list. Observe real owner IEEE words and
current/retained children rather than using the final bool result alone:
NaN equality can conceal an incorrect owner. Reuse the same exact numeric
Target authority and field assembler without integer transport, widening,
finite backend/type recipes or new source permission.

Resources, broader roots/function eligibility, mixed exits and application-
wide closure remain separate. This receipt is not fresh GPU/Linux/Windows
execution, measured performance, full-workspace/strict-Clippy acceptance
or completed self-hosting.
