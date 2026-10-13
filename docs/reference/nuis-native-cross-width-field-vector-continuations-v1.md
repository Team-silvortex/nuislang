# Native Cross Width Field Vector Continuations V1

This bounded CPU proof follows the
[complete 385-test predecessor](nuis-native-floating-owner-field-vector-continuations-v1.md).
The existing exact-scalar field list now has live f32-owner/f64-child and
f64-owner/f32-child evidence beside i32 and two private bool children.
Production admission, field transport and the shared native driver are unchanged.

## Independent Widths

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_cross_width_field_vector.ns)
defaults to f32 owner/f64 child. Its source adapter swaps owner and child
tokens independently, never by cascading replacements. The public state
has 23 words, open has 22 arguments and event has 17 arguments.

The shared reference model now takes separate owner and child type inputs.
Owner negation uses its sign bit; child comparison/negation and decimal
state formatting use the child's width. previous() and stale_sibling()
preserve both types. Legacy same-width constructors and matrices delegate
with equal widths, retaining their original input/state formats.

Per combination/inventory the matrix has 12 owner IEEE words * two
companion child profiles * four i32 values * four bool pairs * four gate
pairs, plus eight NaN/zero equality-masking witnesses: 1544 cases.
Child words come from their own width's dictionary at offsets 1 and 5;
this is not a full IEEE Cartesian product. Signed zero, subnormal, finite,
infinity and quiet/signaling NaN words remain original-width data.

[The reference tests](../../tools/nuisc/tests/native_application_bridge/return_cross_width_field_vector.rs)
check typed inputs, actual accepted-state words, current sibling data,
last publication on failure and width retention under normal/reversed
inventories. A separate model test requires different child raw words for
same-width/cross-width finite values despite equal owner results and calls.
It also checks narrow words, state type suffixes and retained width tags.

Both original and deliberately stale source variants are compiled and
executed. NaN/zero witnesses retain identical bool results and calls but
different real owner bits; finite reference mutants change the public
result as well. Observers dump actual owner/child callee arguments through
their independently correct bitcasts, plus i32 and two bool fields.
They do not inject expected values or replace source computations.

The late raw snapshot is `(owner_sign | 95, -70002, 96, 1, 1)`,
with final owner word `95`. Its child input uses `child_sign | 96`,
not the owner's sign mask. Source-event positions disambiguate marker
collisions. Actual prior callback output is forwarded into the next
callback, with boundary canaries and selected failure prefixes.

## Shared Authority And Layout

Two new unit families retain exact numeric Target authority and atomic
qualified/sealed/resource/root rejection. Owner and child zero neutrals
must have different NIR float kinds; five projections retain their exact
types and separate bool values. Existing lexical child rules and shared
work/logical/capture bounds remain replayed, not widened.

The shared AOT checker now has a source-already-typed entry point.
Legacy wrappers still substitute FLOAT and require the same i64 or
same-width float owner expectation. Cross-width callers supply independent
source, owner and full expected layout:
`{owner:f32;child_0:i32;child_1:f64;child_2:bool;child_3:bool}`
or the reverse float widths. Initialization owner parameter/result,
guard-first transitive Effects, removed-order/default/empty-grant vetoes,
safe narrowing, no live float-width conversion and no owned allocator/drop
remain asserted. This is not formal memory safety.

## Executed Acceptance

The 2026-10-12 serial macOS CPU cohort has **396 distinct tests**:
346 compiler, 17 native/reference, three CLI, three host boundary and
27 tensor tests. Fourteen are new: two units, one AOT, three reference,
six actual native and two CLI. Focused reruns are not added to the count.

Shared model/checker changes require replay of the complete same-width
floating-owner native/reference family alongside the new cross-width
family. The old finite CLI input audit also verifies legacy constructor/
state/decimal argument identity; the new audit checks raw profile width
tags and all fifteen selected stages. Both audits are replayed. The old full
build/cache/restoration workflow, other historical native families and
older CLI receipts are not rerun or counted as fresh execution here.
The new CLI matrix exercises both owner widths and both child decimal
formats. All six complete predecessor tensor fields remain preserved.

The new cross-width family contributes 6208 successful native processes
and 60 fatal native processes, with 18744 accepted callbacks. This includes
6176 original and 32 stale-source successes, under both inventories.
Including the replayed same-width family, the scoped native total is
12416 successes, 120 traps and 37488 accepted callbacks.

Eight new fresh/cache/two successive source-free images contribute
120 CLI event traps and 1032 accepted callbacks, including healthy reentry.
Each image runs 18 finite-decimal cases plus fifteen selected failure/
reentry cases. Raw NaN/payload claims belong to native/reference tests,
not the finite CLI parser. Source, manifest and project cache are absent
before both restores; failed events publish only last accepted open/event
state, with no later event/close/completion. Public state/stdout, binary/
native-object bytes, packaging/envelope/lifecycle and input container
identity remain exact after cache, execution and repeated restoration.

Reproduce from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_cross_width_field_vector return_float_owner_field_vector --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_cross_width_field_vector return_float_owner_field_vector_cli_stale_sensitive_inputs --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
```

The ordinary CLI is rebuilt separately. It exposes six source-equal fields
and six complete predecessor hashes, 2888 clean drift checks / 20240
required patterns, clean coverage/hierarchy/task-card lineage and the
same mainline coordinate. The selected cell stays `active/99`.

The complete six 385-test predecessor fields remain verbatim after the
floating-owner field-vector history markers:

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `1d403dc918e00ba0227fae772c5ccbe0b6da5ac2d75bfedbaeae38a494cb7399` |
| `next_step` | `dc956d5c56cfcc6a1fcb8cf7fffd448d237d0208c5f74fcb6b0a7b2a15a1e889` |
| `blocker` | `f9cf88beef48c8e4b78734ad7900c9281dd53e27532a19db927f38134d24dec2` |
| `next_action` | `04a1ec6444bdce062a52f55d55eaca6c85845947a8dfc869646823b077b5d730` |
| `validation_command` | `1c56decc6e18ceb6ec1e2af393fa19913394ad601c093d54c3e7289b7f0a0e8e` |
| `expected_artifact` | `4aa32936919ba934943db0bd858ec8fdaaaf4b3d4582710d09f951c3d584c365` |

## Remaining Boundary

Next execute simultaneous independently seeded f32 and f64 private
children in one mixed field list beside i32 and two bool siblings under
one existing numeric Target. Keep actual current/retained field-word and
stale-read observations, original shared budgets and source-free identity;
do not infer new authority from higher field count or add backend recipes.

Resources, broader roots, mixed exits and application-wide closure remain
separate. This receipt is not fresh GPU/Linux/Windows execution, measured
performance, full-workspace/strict-Clippy acceptance or completed self-hosting.
