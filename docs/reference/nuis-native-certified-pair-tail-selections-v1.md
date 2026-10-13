# Native Certified Pair Tail Selections V1

This bounded CPU checkpoint follows the
[complete 426-test predecessor](nuis-native-certified-child-tail-selections-v1.md).
It permits two certified child bindings to advance in nested selections
within the single common continuing tail of one in-field source bool exit.

## Certificate And Source Work

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_pair_tail_in_fields.ns)
keeps an i64 owner and private i32/f32/f64/two-bool fields. A nested selected
leaf first negates f64 c, then updates bool d from its preceding d and the
new c's sign comparison. The same leaf updates the numeric owner. Empty
leaves retain all values, and the sibling tail branch updates only the owner.
Later after_work/parent_work consume completed current fields.

The original PrivateFields certificate remains the authority source.
A bounded scan of the entire shared tail returns a set of at most two
distinct certified children, rejecting every other bound write except the
owner. Every child-writing selection must be a subset of that same set.
Existing field staging/transport handles current/retained values; selections
without child writes retain the exact-owner adapter. No root/bool permission
is rediscovered and flat child writes remain rejected.

Three distinct children across the same or separate tail choices, entering
writes, seals/type changes, flat child writes, additional returns, lexical
leakage and future-name conflicts reject atomically. Typed unit alternatives
cover i32/f32/bool variants; this family's fresh execution covers f64/bool
interaction. Earlier one-child and owner-only fixtures remain intact, with
obsolete two-child veto examples narrowed to genuine three-child violations.

Seven data-only six-field records and one guarded bool continuation remain.
No control flags, copied tails or duplicate calls are introduced. The complete
fixture uses thirty-four original nodes and three logical edges. Thirty-three
nodes, one fewer required expression or two logical edges reject. Inherited
depth four passes and five rejects; a deeper sibling choice needs thirty-five
nodes. Whole-source, expression/depth/logical budgets, seals, lexical types and
the 32-parameter capture bound are unchanged.

## Interaction Witness And Proof Operands

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_pair_tail_in_fields_oracle.rs)
keeps before_tail and completed-field tuples. A finite witness with first=true,
second=false, bool_seed=true and peer_seed=false changes c from positive to
negative, then changes d from true to false using the new c. Its correct
final owner differs from seed and the final bool is false. Feeding old c to
the second update instead leaves d true, wrongly retains the final owner at
seed and returns true; e is false and cannot mask the error. Actual compiled
after_work fields and answer_work owner arguments are observed independently.

The fixture already reaches thirty-two open arguments, thirty-three state
words and twenty-six event arguments. It keeps that bound instead of adding
another proof parameter: the new tail_pair operand replaces tail_yes, and
the owner call uses constant proof 1. Owner execution, once-only order and
arguments are still observed, but independent
owner-work trap coverage belongs to the freshly replayed predecessor.
This new family has twenty-four poisonable source boundaries and twenty-five
distinct helper effect markers, plus entry/publication markers. Marker 65
is not claimed as an independent new trap.

Sixteen finite gate/bool cases plus sixty-four paired IEEE profiles cover signed
zeros, subnormals, infinities and quiet/signaling NaNs, not a Cartesian product.
Both source-arm polarities and normal/reversed function inventories use actual
previous outputs, canaries and source-position failure cuts, including words
equal to stage markers. Prefix exit_work and completed after_work modes are
**not simultaneous observations** in a process. Inactive operands stay zero.
The original source exit, including a false bool, skips the entire common tail.

## Execution And Delivery

The new family contributes **640 successful native processes**,
**192 fatal native processes** and 2304 accepted native callbacks per full
replay: two sources * two inventories * two independent modes * eighty healthy
/ twenty-four selected failure cases. Focused repeats do not increase totals.
The direct predecessor is freshly replayed separately.

Eight fresh/cache/two successive source-free images contribute
**192 CLI event traps** and **1344 accepted callbacks**. Each image runs
sixteen finite healthy cases, twenty-four selected failures and healthy reentry.
Source, manifest and project cache are absent before both restores. Executable,
native objects, policy/envelope/lifecycle, container bytes, complete state and
healthy stdout remain exact. Fatal CLI stdout can be buffered; exact partial
effects belong to the separately flushed native observer.
AOT retains transitive guard-first Effect order, explicit print grants and
no owned aggregate allocator/drop. This is not formal memory-safety acceptance.

## Scoped Acceptance

The 2026-10-13 serial macOS CPU cohort passed **432 distinct tests**: 390 compiler, eight
native/reference, four CLI, three host and 27 tensor. Twelve are new:
five units, one AOT, two reference, two native and two CLI.
The direct predecessor's native/reference and CLI families are freshly replayed;
older runtime histories remain historical, not fresh execution.

Run from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields return_in_fields return_nested_in_fields return_prefix_in_fields return_sibling_in_fields return_tail_in_fields return_child_tail_in_fields return_pair_tail_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_child_tail_in_fields return_pair_tail_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_child_tail_in_fields return_pair_tail_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

The selected cell remains
`standard-library/ns-nova/persistent-application-session`, `active/99`.
All six complete predecessor strings remain after one-child common-tail
history markers, with fresh SHA-256, not hash-only summaries.
The rebuilt ordinary CLI exposes six source-equal fields and six matching
complete predecessor hashes. All 3044 drift checks / 21292 patterns and
coverage, hierarchy and task-card lineage are clean. Changed Rust formatting
passes for 21 files; UTF-8, file caps, relative links and diff whitespace pass.
Whole-workspace formatting still reports four untouched historical files,
so this is not whole-workspace formatting or strict-Clippy acceptance.

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `5128d718b3688f88d27eeedd1b76886260671919c31324112c1308d8063e116b` |
| `next_step` | `c2d230183ab06f63259cacb3b1db0045fb33fd9e69f69d1b9c27af09090203e8` |
| `blocker` | `82b958b59a633e80de0e76531b30cad4a8c91d3b62bf8ae38934f141fd0c3481` |
| `next_action` | `2e648c4904a2b98b33426a1e21767c3aaf6798a23e85fa3737a66cca08a0b524` |
| `validation_command` | `8ac6bfb0aee11e49c0fc89834e057d3e1dc86a066bdf1f5c7d43242c06563a57` |
| `expected_artifact` | `05cc98daa8781f173a24e5897e452bc994487a74f24ca5147601510fe5c9271b` |

## Remaining Boundary

Two certified child bindings are supported; three distinct tail children and
multiple exits remain separate. Next prove three independently updated
certified children with real current-value interaction and empty-arm retention,
under the same certificate and single tail, without widening entering writes,
exporting branch-local names or manufacturing exit authority.
Resources and deeper/multiple source exits remain separate. This local CPU
receipt does not certify GPU/Linux/Windows execution, heterogeneous application
closure, performance, completed self-hosting or whole-workspace formatting/
strict-Clippy acceptance.
