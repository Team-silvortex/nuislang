# Native Certified Quint Tail Selections V1

This bounded CPU checkpoint follows the
[complete 444-test predecessor](nuis-native-certified-quad-tail-selections-v1.md).
It permits five certified child bindings to advance in nested selections
within the single common continuing tail of one in-field source bool exit.

## Certificate And Source Work

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_quint_tail_in_fields.ns)
keeps an i64 owner and private i32/f32/f64/two-bool fields. A nested selected
leaf negates f64 c, updates bool d from the new c's sign, updates i32 a from
the new d, updates f32 b from the new a's sign, and updates bool e from the
new b's comparison. It then complements the owner. Empty leaves retain all
fields; the other tail branch updates only the owner. Later after_work and
parent_work consume completed current fields.

The original PrivateFields certificate remains the sole child-write authority.
A bounded scan of the entire shared tail returns at most five distinct
certified children, excluding the exact owner. Every child-writing selection
must be a subset of that same whole-tail set. Existing field staging and
transport retain current/unchanged values; owner-only selections still use
the exact-owner adapter. No entering/root/bool authority is rediscovered.

[The independent capacity test](../../tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_tail_child_bound_tests.rs)
obtains a real six-child certificate through classify, inside the certificate
module's private test scope. Five authorized writes pass, while six authorized
writes in one sequence or across a separate choice reject. Adding the exact
owner does not consume another child slot; an entering seed write rejects
independently. No production visibility is widened for testing, and no
uncertified root write is passed off as an authorized capacity counterexample.

The live fixture has exactly five certified children. Old five-child veto
examples are narrowed to genuine entering-root violations, while the
independent six-child certificate test proves the capacity-only boundary.
Flat child writes, seals/type changes, additional returns, lexical leaks and
future-name collisions still reject atomically. Unit alternatives cover
i32/f32/bool bindings and a separate fifth-child choice under the same whole-tail
set. Fresh execution covers f64/bool/i32/f32/bool current-value interaction.
Earlier source fixtures and historical receipts remain intact.

Seven data-only six-field records and one guarded bool continuation remain.
No control flags, copied tails or duplicate source calls are introduced.
The complete fixture charges thirty-seven original nodes and three logical
edges. Thirty-six nodes, one fewer required expression or two logical edges
reject. Inherited depth four passes and five rejects; a deeper sibling choice
needs thirty-eight nodes. Whole-source, expression/depth/logical budgets,
seals, exact types and the 32-parameter capture limit remain unchanged.

## Interaction Witness And Proof Operands

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_quint_tail_in_fields_oracle.rs)
retains before_tail and completed-field tuples. A finite witness with
first=true, second=false, bool_seed=true and peer_seed=false changes c from
positive to negative, d from true to false, a from i32::MIN to i32::MAX,
b from -2.5f32 to +2.5f32, and e from false to true.
The correct final owner equals seed and the final bool is true.

This fixture uses answer_work(...) && e rather than the predecessor's || e:
the final answer needs both the current owner and the current peer, so a true
peer cannot conceal an incorrect owner answer. Reading old negative b in the
fifth update instead leaves e false. With the other completed fields fixed,
the final parity complements owner away from seed; the final conjunction is
false. This is an independently distinguishing stale-fifth-input result,
not merely an updated field with no contribution to the answer.
Actual compiled after_work fields and answer_work owner arguments are observed.

Earlier dependency errors can still cancel in a final outcome. This family
retains raw d/a/b assertions rather than claiming one final bool independently
distinguishes every stale intermediate input. The freshly replayed quad
predecessor retains its own unmasked stale-fourth-input result.
Earlier triple/pair results and independent one-child owner traps are
historical evidence, not freshly replayed by this selected cohort.

The fixture keeps thirty-two open inputs, thirty-three state words and
twenty-six event inputs. The fifth-child helper consumes existing e/b values
without another proof operand. Marker 59 proves once-only order and execution,
but cannot independently poison that call. There are twenty-four poisonable
source boundaries and twenty-eight distinct helper markers, plus
entry/publication markers:
markers 59, 60, 61 and 65 are not independent traps.

Sixteen finite gate/bool cases and sixty-four paired IEEE profiles cover signed
zeros, subnormals, infinities and quiet/signaling NaNs, not a Cartesian product.
Both source-arm polarities and normal/reversed inventories consume actual
previous outputs, guard canaries and source-position failure cuts, including
field words equal to stage markers. Prefix exit_work and completed after_work
modes are **not simultaneous observations** in a process.
Inactive operands remain zero. A source exit, including false, skips every
common-tail condition, update and effect. Raw f32/f64 sign and payload words
remain separate from comparisons that can mask zero/NaN signs. The fifth helper
toggles on not-negative, including zero/NaN, not on an ordered >= comparison.

## Execution And Delivery

The new family contributes **640 successful native processes**,
**192 fatal native processes** and 2304 accepted native callbacks per full
replay: two sources * two inventories * two independent observer modes *
eighty healthy / twenty-four selected failure cases.
Focused repeats do not increase these totals. The direct quad predecessor
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

The 2026-10-13 serial macOS CPU cohort passed **451 distinct tests**:
409 compiler, eight native/reference, four CLI, three host and 27 tensor.
Thirteen are new: six units, one AOT, two reference, two native and two CLI.
The direct quad predecessor's native/reference and CLI families are freshly
replayed; older runtime histories remain historical, not fresh execution.

Run from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields return_in_fields return_nested_in_fields return_prefix_in_fields return_sibling_in_fields return_tail_in_fields return_child_tail_in_fields return_pair_tail_in_fields return_triple_tail_in_fields return_quad_tail_in_fields return_quint_tail_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_quad_tail_in_fields return_quint_tail_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_quad_tail_in_fields return_quint_tail_in_fields --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

The selected cell remains
`standard-library/ns-nova/persistent-application-session`, `active/99`.
All six complete predecessor strings remain after four-child common-tail
history markers, with fresh SHA-256, not hash-only summaries.
The rebuilt ordinary CLI exposes six source-equal fields and six matching
complete predecessor hashes. All 3093 drift checks / 21665 patterns and
coverage, hierarchy and task-card lineage are clean.
Changed Rust formatting passes for 28 files. UTF-8, source/test/document
caps, relative links and diff whitespace pass. Whole-workspace formatting
still reports four untouched historical files; this is not whole-workspace
formatting or strict-Clippy acceptance.

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `5f9fef71290ddf7268efdcdd748274b88b0c27a43d1fa1395724f266b37c9f5a` |
| `next_step` | `66c2ae922a8e92d24b7eb51b6818abf8c469a2f5e9b33f5497e1e141bdb8362e` |
| `blocker` | `265b2876db112f234039d602a76cf5699364f15039626e5debc1d00612ea0f75` |
| `next_action` | `84dcd800ce4a8a9016a6e3dcf47d9ee2fbd7b760f93a305694003bf59a744d67` |
| `validation_command` | `fbb85102f31f8ab621a81edf082559bd705bb6978cff71114160f6d1d3a418ad` |
| `expected_artifact` | `33c19ebf951779c52940f1a8ba738ed5ead017fbd7b57a946fbb0e7bd0d0a5f0` |

## Remaining Boundary

Five distinct certified children are supported; six distinct children and
multiple exits remain separate. The next application-led boundary is a second
explicit source bool exit in the other field arm. Prove both exit predicates
from actual branch-current fields, preserve the last accepted publication,
and retain one common continuing tail without field-record control flags,
entering write authority, copied work or incidental capture/budget increases.
Six-child live coverage, resources and deeper/general multiple exits remain
separate. This local CPU receipt does not certify GPU/Linux/Windows execution,
heterogeneous application closure, performance, completed self-hosting or
whole-workspace formatting/strict-Clippy acceptance.
