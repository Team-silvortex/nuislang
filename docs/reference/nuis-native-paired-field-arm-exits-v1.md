# Native Paired Field Arm Exits V1

This bounded CPU checkpoint follows the
[complete 451-test predecessor](nuis-native-certified-quint-tail-selections-v1.md).
One field selection may now have one explicit direct source bool exit in each
arm, while retaining a single continuing common tail.

## Source Authority And Staging

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_paired_in_fields.ns)
retains an i64 owner and private i32/f32/f64/two-bool fields. The original arm
has its existing prefix work, exit, nested remainder and owner update.
The sibling arm now checks a second source exit after its own current-field
prefix and before sibling_owner_work. It continues to the same five-child
common tail only when its exit is not taken.

[The separate paired adapter](../../tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_paired_in_fields.rs)
is dispatched only after whole-source preflight and one valid direct exit
is identified in each arm. Exit removal is for write classification only;
the original before/after slices supply staged work and inherited budgets.
The original PrivateFields certificate is the sole field-write authority.

The source selector is proved and bound once to an immutable fresh gate.
One field split transports the selected arm's prefix fields. Each source exit
predicate is lazy behind its own gate polarity, so an unselected exit's calls
and argument divisions never run. The paired plans are constructed in reverse
to keep the physical yes check outermost; mirror-source coverage swaps both
physical arms and inverts the original selector.
After both exit checks fail, one field split runs the selected arm's remaining
work and one common continuing tail. The adapter does not copy that tail,
rediscover entering/bool/root write authority, or put control flags in records.

Seven data-only six-field records remain, and only one guarded bool
continuation contains after_work. There are now two explicit exit decisions,
not a claim of one total bool helper. A false early value is still a source
return, not permission to continue. Each tail helper and source selector
occurs once. Generated parameters remain within the original 32-input limit.

## Bounds And Rejections

The complete fixture charges thirty-nine original nodes and four logical
edges. Thirty-eight nodes, one fewer required expression or three logical
edges reject. Inherited depth four passes and five rejects; a deeper sibling
choice needs forty nodes. Budget charges belong to original work, not each
generated gate/plan, and no adapter creates a new budget or resets depth.

More than one exit per arm, deeper/multi-statement exit bodies, non-bool/void
returns, common-tail exits, entering writes, seals/type changes, flat child
writes, ordinary-local leaks and future-name collisions reject atomically.
Unit tests also remove either source exit in both source polarities and
reverify the resulting one-sided adapter route. The child-write capacity
remains five; six-child live coverage and generalized resources are not added.

## Current Fields And Exit Evidence

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_paired_in_fields_oracle.rs)
retains the actual selected prefix tuple, completed tail tuple and final
owner. Both source arms have true and false early values. For a finite sibling
witness with first=true, second=false, bool_seed=false and peer_seed=true,
the prefix updates i32 a from MIN to MAX and d from false to true.
Its current a/d predicate takes the exit and returns false, skipping both the
sibling owner update and every common-tail effect.

Using stale negative a would wrongly miss that exit. A separate arithmetic
counterfactual with healthy continuing proof operands returns true, opposite
to the correct false result; it is not claimed as a mutated-compiler execution.
Actual exited cases additionally keep inactive continuing operands at zero,
so erroneous execution of those paths would also trap.
Both the false published result and absent suffix effects are checked in real
compiled execution, alongside the actual prefix arguments supplied to each
exit callee. The direct predecessor's five-child interaction witness is
freshly replayed rather than replaced by this new exit witness.

The fixture still has thirty-two open inputs, thirty-three state words and
twenty-six event inputs. The new sibling_exit operand replaces the old sibling
owner proof at zero-based proof index 17 (event word 19 / open word 25);
sibling_owner_work now receives constant 1.
There remain twenty-four poisonable source boundaries. New marker 58 is an
independent selected source-exit trap, while marker 67 is not an independent new trap.
The freshly replayed predecessor retains independent marker-67 trap coverage.
Markers 59, 60, 61 and 65 remain observed effects without independent new traps.
There are twenty-nine helper markers, plus entry/publication markers.

## Native And Delivery Checks

The generic native observer accepts a list of callee observations; the old
single-observer API delegates to it without changing older callers.
Prefix mode installs observers at both exit callees, but only the selected
one executes in a callback. Prefix and completed-tail modes are
**not simultaneous observations** in a process.
Raw i32/f32/f64/bool field words and final-owner callee arguments are observed;
real source computation and argument traps are not replaced by expected data.

Sixteen finite gate/bool cases and sixty-four paired IEEE profiles cover signed
zeros, subnormals, infinities and quiet/signaling NaNs, not a Cartesian product.
Both source polarities and normal/reversed inventories consume real previous
outputs, canaries and source-position failure cuts, including field words equal
to stage markers. Source-position cuts, not matching raw data words, identify
the failing source call. The original selected prefix work survives a later
failure; unselected/dead proofs remain zero and do not trap.

The new family contributes **640 successful native processes**,
**192 fatal native processes** and 2304 accepted native callbacks per full
replay: two sources * two inventories * two independent observer modes *
eighty healthy / twenty-four selected failure cases.
Focused repeats do not add to those totals. A freshly rebuilt CLI-only audit
repeat checks the added zero-tail/profile-count assertions without replaying
unchanged workflow bodies or suppressing unused-field/function warnings. The direct quint predecessor is
freshly replayed separately.

Eight fresh/cache/two successive source-free images contribute
**192 CLI event traps** and **1344 accepted callbacks**. Each image runs sixteen
finite healthy cases, twenty-four selected failures and healthy reentry.
Source, manifest and project cache are absent before both restores.
Executable, native objects, envelope/policy/lifecycle, container bytes, complete
state and healthy stdout remain exact. Fatal CLI stdout may be buffered;
exact partial effects belong to the separately flushed native observer.
AOT retains transitive guard-first Effect order, explicit print grants and
no owned aggregate allocator/drop. This is not formal memory-safety acceptance.

## Scoped Acceptance

The 2026-10-15 serial macOS CPU cohort passed **458 distinct tests**:
416 compiler, eight native/reference, four CLI, three host and 27 tensor.
Thirteen are new: six units, one AOT, two reference, two native and two CLI.
The direct quint predecessor's scoped native/reference and CLI families are
freshly replayed; older runtime histories remain historical, not fresh execution.

Run from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields return_in_fields return_nested_in_fields return_prefix_in_fields return_sibling_in_fields return_tail_in_fields return_child_tail_in_fields return_pair_tail_in_fields return_triple_tail_in_fields return_quad_tail_in_fields return_quint_tail_in_fields return_paired_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_quint_tail_in_fields return_paired_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_quint_tail_in_fields return_paired_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_paired_in_fields_cli_both_source_exits --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

The selected cell remains
`standard-library/ns-nova/persistent-application-session`, `active/99`.
Six complete predecessor fields remain after five-child common-tail history
markers, with fresh SHA-256, not hash-only summaries. The rebuilt ordinary CLI
exposes six source-equal fields and six matching complete predecessor hashes.
All 3111 drift checks / 21783 patterns and coverage, hierarchy and task-card
lineage are clean. Changed Rust formatting passes for 15 files; UTF-8,
source/test/document caps, relative links and diff whitespace pass.
Whole-workspace formatting still reports four untouched historical files;
this is not whole-workspace formatting or strict-Clippy acceptance.

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `ee794e1db54fd5e2ee309a1a6d7ab9676a811c4334894d7b3430a4807e280398` |
| `next_step` | `e4d3d30f662bbb42869c88eb4d11e19ccfec32ef6b818a9abab1f9bc4200791b` |
| `blocker` | `4f2a67e49ab7467f41cba3dba933080de39f423f3d844a694f6ed5d022b5fe71` |
| `next_action` | `f6654ebfc8094166ba82ab1d2229b846fc3f10c2c04580842e40f7da8edd3578` |
| `validation_command` | `b2bf362cba2b619654db91f822f56774dc2f6c7ecd41baf85e474ede04634bf1` |
| `expected_artifact` | `88824b3c2d2407be2250a8e189c68c131feeb7433b8ce667d8bf4766a52f9a73` |

## Remaining Boundary

The accepted source shape is one direct bool exit per field arm and one
common continuing tail. Next prove a bounded continuing selection/prefix inside the returning branch
of a source-exit if before its bool return, retaining current fields and
explicit source authority rather than manufacturing deeper return discovery.
The remaining exclusions include more than one exit per arm, deeper/general
exits, six-child live coverage, ordinary-local exports and generalized resources.
This local CPU receipt does not certify GPU/Linux/Windows execution,
heterogeneous application closure, performance, completed self-hosting or
whole-workspace formatting/strict-Clippy acceptance.
