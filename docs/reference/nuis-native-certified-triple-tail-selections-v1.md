# Native Certified Triple Tail Selections V1

This bounded CPU checkpoint follows the
[complete 432-test predecessor](nuis-native-certified-pair-tail-selections-v1.md).
It permits three certified child bindings to advance in nested selections
within the single common continuing tail of one in-field source bool exit.

## Certificate And Source Work

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_triple_tail_in_fields.ns)
keeps an i64 owner and private i32/f32/f64/two-bool fields. A nested selected
leaf negates f64 c, updates bool d from its preceding d and the new c's sign,
then updates i32 a from the new d. It finally complements the owner.
Empty leaves retain every field; the other tail branch updates only the owner.
Later after_work/parent_work consume the completed current fields.

The original PrivateFields certificate remains the sole child-write authority.
A bounded scan of the entire shared tail returns at most three distinct
certified children, excluding the exact owner. Every child-writing selection
must be a subset of that same whole-tail set. Existing field staging and
transport preserve current/retained values, and owner-only selections keep
the exact-owner adapter. No entering/root/bool authority is rediscovered.

Four distinct children within one selection or across separate tail choices,
entering writes, flat child writes, seals/type changes, additional returns,
lexical leaks and future-name collisions reject atomically. Unit alternatives
cover i32/f32/bool bindings and a separate third-child choice under the same
whole-tail set. The new live fixture covers f64/bool/i32 interaction.
Earlier source fixtures remain intact; obsolete three-child veto examples
are narrowed to real four-child violations. Historical receipts remain frozen.

Seven data-only six-field records and one guarded bool continuation remain.
There are no copied tail calls or control flags in field records.
The complete fixture charges thirty-five original nodes and three logical
edges. Thirty-four nodes, one fewer required expression or two logical edges
reject. Inherited depth four passes and five rejects; a deeper sibling
choice needs thirty-six nodes. Whole-source, expression/depth/logical budgets,
seals, exact types and the 32-parameter capture limit are unchanged.

## Interaction Witness And Proof Operands

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_triple_tail_in_fields_oracle.rs)
retains before_tail and completed-field tuples. A finite witness with
first=true, second=false, bool_seed=true and peer_seed=false changes c from
positive to negative, d from true to false, and a from i32::MIN to i32::MAX.
The correct final owner equals seed and the final bool is true.
Reading old d in the third update instead keeps a negative; the final parity
then complements the owner and yields false. The peer is false and cannot
mask this error. Actual compiled after_work field arguments and answer_work
owner arguments are independently observed.

The final triple result can cancel an earlier stale-c error in the second
update. Therefore it is not claimed as a final-result counterexample for
both dependencies: the triple's raw d observation distinguishes that error,
and the freshly replayed pair predecessor has its own unmasked stale-c bool
witness. Oracle/reference assertions check both dependency boundaries.

The fixture keeps thirty-two open inputs, thirty-three state words and
twenty-six event inputs. The new third-child helper consumes the existing a/d
values without another proof operand. Its marker 61 proves once-only order
and execution, but cannot independently poison that new call.
There are twenty-four poisonable source boundaries and twenty-six distinct
helper markers, plus entry/publication markers: markers 61 and 65 are not independent traps.
The pair predecessor is freshly replayed. Independent owner-work trap
coverage remains in the earlier accepted one-child receipt, historical
rather than newly replayed by this selected cohort.

Sixteen finite gate/bool cases and sixty-four paired IEEE profiles cover
signed zeros, subnormals, infinities and quiet/signaling NaNs, not a Cartesian
product. Both source-arm polarities and normal/reversed inventories consume
real previous outputs, guard canaries and source-position failure cuts,
including field words equal to stage markers. Prefix exit_work and completed
after_work modes are **not simultaneous observations** in a process.
Inactive operands remain zero. A source exit, including false, skips every
common-tail condition, update and effect.

## Execution And Delivery

The new family contributes **640 successful native processes**,
**192 fatal native processes** and 2304 accepted native callbacks per full
replay: two sources * two inventories * two independent observer modes *
eighty healthy / twenty-four selected failure cases.
Focused repeats do not increase these totals. The direct pair predecessor
is freshly replayed separately.

Eight fresh/cache/two successive source-free images contribute
**192 CLI event traps** and **1344 accepted callbacks**.
Each image runs sixteen finite healthy cases, twenty-four selected failures
and healthy reentry. Source, manifest and project cache are removed before
both restores. Executable, native objects, policy/envelope/lifecycle,
container bytes, complete state and healthy stdout remain exact.
Fatal CLI stdout may be buffered; exact partial effects are checked by the
separately flushed native observer, not inferred from CLI stdout.
AOT retains transitive guard-first Effect order, explicit print grants,
and no owned aggregate allocator/drop. This is not formal memory-safety acceptance.

## Scoped Acceptance

The 2026-10-13 serial macOS CPU cohort passed **438 distinct tests**:
396 compiler, eight native/reference, four CLI, three host and 27 tensor.
Twelve are new: five units, one AOT, two reference, two native and two CLI.
The direct pair predecessor's native/reference and CLI families are freshly
replayed; older runtime histories remain historical, not fresh execution.

Run from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields return_in_fields return_nested_in_fields return_prefix_in_fields return_sibling_in_fields return_tail_in_fields return_child_tail_in_fields return_pair_tail_in_fields return_triple_tail_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_pair_tail_in_fields return_triple_tail_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_pair_tail_in_fields return_triple_tail_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

The selected cell remains
`standard-library/ns-nova/persistent-application-session`, `active/99`.
All six complete predecessor strings remain after two-child common-tail
history markers, with fresh SHA-256, not hash-only summaries.
The rebuilt ordinary CLI exposes six source-equal fields and six matching
complete predecessor hashes. All 3060 drift checks / 21401 patterns and
coverage, hierarchy and task-card lineage are clean.
Changed Rust formatting passes for 28 files. UTF-8, source/test/document
caps, relative links and diff whitespace pass. Whole-workspace formatting
still reports four untouched historical files; this is not whole-workspace
formatting or strict-Clippy acceptance.

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `75a7152e635058f31176b06d2b6d92fbd01b5a51b15f791f60a5a3bb8118c133` |
| `next_step` | `1f6c64d286c930de3f5d08f2d51bb87f50d4a072764826f304cce2c092ef77d3` |
| `blocker` | `8a8e5139480d3a229cf83d0f3b06d28229a2bc793d6800e1f8d85a71c944c8af` |
| `next_action` | `0b167daf5b665130812a26b4367d41c6f9baf11b4aa54ed5454c6f0f95afc62b` |
| `validation_command` | `8b3a773bafdba3131c4bea21d923d91a6d7856fb4de38681e0e14f4c67f2cc5f` |
| `expected_artifact` | `735f7865e74497b899f8b46c1bbce4e98d6b393468ee7cba28404e2e982e5f4f` |

## Remaining Boundary

Three distinct certified children are supported; four distinct children and
multiple exits remain separate. Next prove four independently updated
certified children with current-value interaction and retained empty leaves
under the same original certificate and single continuing tail.
Do not widen entering writes, expose branch-local names, copy a continuation,
increase captures incidentally or manufacture source-exit authority.
Resources and deeper/multiple source exits remain separate.
This local CPU receipt does not certify GPU/Linux/Windows execution,
heterogeneous application closure, performance, completed self-hosting or
whole-workspace formatting/strict-Clippy acceptance.
