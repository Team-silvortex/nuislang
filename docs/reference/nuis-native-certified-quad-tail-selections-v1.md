# Native Certified Quad Tail Selections V1

This bounded CPU checkpoint follows the
[complete 438-test predecessor](nuis-native-certified-triple-tail-selections-v1.md).
It permits four certified child bindings to advance in nested selections
within the single common continuing tail of one in-field source bool exit.

## Certificate And Source Work

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_quad_tail_in_fields.ns)
keeps an i64 owner and private i32/f32/f64/two-bool fields. A nested selected
leaf negates f64 c, updates bool d from the new c's sign, updates i32 a from
the new d, and updates f32 b from the new a's sign. It then complements the
owner. Empty leaves retain all fields; the other tail branch updates only
the owner. Later after_work/parent_work consume completed current fields.

The original PrivateFields certificate remains the sole child-write authority.
A bounded scan of the entire shared tail returns at most four distinct
certified children, excluding the exact owner. Every child-writing selection
must be a subset of that same whole-tail set. Existing field staging and
transport retain current/unchanged values; owner-only selections still use
the exact-owner adapter. No entering/root/bool authority is rediscovered.

Five distinct children within one selection or across separate tail choices,
entering writes, flat child writes, seals/type changes, additional returns,
lexical leaks and future-name collisions reject atomically. Unit alternatives
cover i32/f32/bool bindings and a separate fourth-child choice under the same
whole-tail set. Fresh execution covers f64/bool/i32/f32 current-value interaction.
Earlier source fixtures remain intact; obsolete four-child veto examples
are narrowed to real five-child violations. Historical receipts remain frozen.

Seven data-only six-field records and one guarded bool continuation remain.
No control flags, copied tails or duplicate source calls are introduced.
The complete fixture charges thirty-six original nodes and three logical
edges. Thirty-five nodes, one fewer required expression or two logical edges
reject. Inherited depth four passes and five rejects; a deeper sibling
choice needs thirty-seven nodes. Whole-source, expression/depth/logical budgets,
seals, exact types and the 32-parameter capture limit remain unchanged.

## Interaction Witness And Proof Operands

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_quad_tail_in_fields_oracle.rs)
retains before_tail and completed-field tuples. A finite witness with
first=true, second=false, bool_seed=true and peer_seed=false changes c from
positive to negative, d from true to false, a from i32::MIN to i32::MAX,
and b from -2.5f32 to +2.5f32. Its correct final owner differs from seed and
the final bool is false. Reading old negative a in the fourth update instead
keeps b negative. With the other completed fields held fixed, the final
parity then retains owner at seed and yields true. The peer is false and
cannot mask the error. Actual compiled after_work field arguments and
answer_work owner arguments are independently observed.

The fourth update can cancel the final-result effect of an earlier stale-d
error in the third update. Therefore this family's final bool is not claimed
as an independently unmasked witness for every earlier dependency:
the raw a/d/b observations distinguish those intermediate errors, and the
freshly replayed triple predecessor retains its own unmasked stale-third-input
result. The pair predecessor's independent stale-second-input result is
earlier historical evidence, not newly replayed by this selected cohort.
Oracle/reference assertions keep these dependency checks separate.

The fixture keeps thirty-two open inputs, thirty-three state words and
twenty-six event inputs. The fourth-child helper consumes existing b/a values
without another proof operand. Its marker 60 proves once-only order and
execution, but cannot independently poison that call.
There are twenty-four poisonable source boundaries and twenty-seven distinct
helper markers, plus entry/publication markers:
markers 60, 61 and 65 are not independent traps.
Independent owner-work trap coverage remains in the earlier accepted one-child
receipt, historical rather than freshly replayed by this selected cohort.

Sixteen finite gate/bool cases and sixty-four paired IEEE profiles cover signed
zeros, subnormals, infinities and quiet/signaling NaNs, not a Cartesian product.
Both source-arm polarities and normal/reversed inventories consume actual
previous outputs, guard canaries and source-position failure cuts, including
field words equal to stage markers. Prefix exit_work and completed after_work
modes are **not simultaneous observations** in a process.
Inactive operands remain zero. A source exit, including false, skips every
common-tail condition, update and effect. Raw f32/f64 sign and payload words
are checked independently of comparisons that can mask zero/NaN signs.

## Execution And Delivery

The new family contributes **640 successful native processes**,
**192 fatal native processes** and 2304 accepted native callbacks per full
replay: two sources * two inventories * two independent observer modes *
eighty healthy / twenty-four selected failure cases.
Focused repeats do not increase these totals. The direct triple predecessor
is freshly replayed separately.

Eight fresh/cache/two successive source-free images contribute
**192 CLI event traps** and **1344 accepted callbacks**.
Each image runs sixteen finite healthy cases, twenty-four selected failures
and healthy reentry. Source, manifest and project cache are removed before
both restores. Executable, native objects, policy/envelope/lifecycle,
container bytes, complete state and healthy stdout remain exact.
Fatal CLI stdout may be buffered; exact partial effects belong to the
separately flushed native observer, not inference from CLI stdout.
AOT retains transitive guard-first Effect order, explicit print grants and
no owned aggregate allocator/drop. This is not formal memory-safety acceptance.

## Scoped Acceptance

The 2026-10-13 serial macOS CPU cohort passed **444 distinct tests**:
402 compiler, eight native/reference, four CLI, three host and 27 tensor.
Twelve are new: five units, one AOT, two reference, two native and two CLI.
The direct triple predecessor's native/reference and CLI families are freshly
replayed; older runtime histories remain historical, not fresh execution.

Run from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields return_in_fields return_nested_in_fields return_prefix_in_fields return_sibling_in_fields return_tail_in_fields return_child_tail_in_fields return_pair_tail_in_fields return_triple_tail_in_fields return_quad_tail_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_triple_tail_in_fields return_quad_tail_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_triple_tail_in_fields return_quad_tail_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

The selected cell remains
`standard-library/ns-nova/persistent-application-session`, `active/99`.
All six complete predecessor strings remain after three-child common-tail
history markers, with fresh SHA-256, not hash-only summaries.
The rebuilt ordinary CLI exposes six source-equal fields and six matching
complete predecessor hashes. All 3076 drift checks / 21521 patterns and
coverage, hierarchy and task-card lineage are clean.
Changed Rust formatting passes for 35 files. UTF-8, source/test/document
caps, relative links and diff whitespace pass. Whole-workspace formatting
still reports four untouched historical files; this is not whole-workspace
formatting or strict-Clippy acceptance.

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `59854511a7fd7fd6f4c183b91da419bb2c5f761272cd1590d0cb1d46cbf77e78` |
| `next_step` | `a7d98779a53ac3c6b3f7b7d6e606832ed36feb35abe36ad4a0c3be80a6542af7` |
| `blocker` | `5ba0d941b617f3a2fdf685f14e194b47b22c2a899b6a59bd50a98ab7c375cacc` |
| `next_action` | `db77f75ccc773934de758e6ddbcc5656cc4c27f3c729348f73d48666e9c0cd76` |
| `validation_command` | `6942c9a07e6a3c7dfa3de04631edf7e23d733f8617513176eed9d5c836fcd91e` |
| `expected_artifact` | `dcdb9c68fa7c07bc0f441d4daa93792d4934562e0069729c6c4372726a7c9e39` |

## Remaining Boundary

Four distinct certified children are supported; five distinct children and
multiple exits remain separate. Next prove five independently updated certified
children with genuine current-value interaction and retained empty leaves
under the same original certificate and single continuing tail.
Do not widen entering writes, expose branch-local names, copy a continuation,
increase captures incidentally or manufacture source-exit authority.
Resources and deeper/multiple source exits remain separate.
This local CPU receipt does not certify GPU/Linux/Windows execution,
heterogeneous application closure, performance, completed self-hosting or
whole-workspace formatting/strict-Clippy acceptance.
