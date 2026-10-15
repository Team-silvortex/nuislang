# Native Plain Bool Return Trees V1

This bounded CPU checkpoint follows the
[ordinary prefix-return receipt](nuis-native-plain-bool-prefix-returns-v1.md).
Bounded complete bool-return trees no longer need a syntactic logical leaf
to enter their existing typed/effect proof.

## Proof And Authority

[The tree entry](../../tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_trees.rs)
still requires a nested final if and explicit final bool returns throughout
both arms. It now directly composes ReturnedValue proofs instead of carrying
a Tree.logical_return syntax flag. Discovery of an impure call belongs to
the whole tree, including conditions and unused selected prefix work;
independently pure children may compose, but an entirely pure tree is not admitted by this effectful entry.

The parent exact bool type, scalar signatures, current lexical scope,
read-only entering bindings, fresh locals, seals, guard-first installation,
31 captures plus one guard and shared source budgets remain unchanged.
The bounds are still 64 nodes, 256 expressions, 32 original logical edges,
eight selection levels and 16 prefix statements. No budget resets occur
at child boundaries, and no generated gating edge is charged to the source.

[Four new unit tests](../../tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_plain_tree_tests.rs)
cover call/literal/name leaves; effects only in root/descendant conditions,
unused ancestor/leaf prefixes; private ancestor captures and logical seals.
Two prior syntax-only negative cases are now explicit positives. Partial
returns, mixed fallthrough, inherited/private-ancestor writes, seal rewrites,
cross-leaf leakage, hidden logical call arguments, unsupported statements,
late invalid NIR types/values and async effects still reject atomically,
preserving both the module and generated-name inventory.

An exact six-node/seven-expression tree uses zero original logical edges.
One fewer node or expression rejects. Separate 31/32 capture, 8/9 selection
depth and 64/65 original-node cases check their respective boundaries;
the existing logical-tree and logical-ancestor bounds remain freshly replayed.
The runtime fixture keeps a logical const initializer because ordinary
nonlogical const staging is a separate existing restriction, not widened here.

## Execution And Delivery

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_plain_tree.ns)
has four explicit leaves: an effectful call, a sealed variable, true and false.
The call observes a current ancestor-local i64 after a real +1 transformation.
For both seeds, independently using the original input instead changes the
expected result; native callee observations also check the transformed value.
This counterfactual is not mutated-compiler execution. Unused selected work is also
retained. [The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_plain_tree_cases.rs)
enumerates 16 inputs over seed, root choice, child choice and bool value.
Each unused proof operand is zero; seven argument boundaries are independently
poisoned. Original/mirrored plain sources and logical-leaf wrapped predecessors
provide four distinct source variants under the same oracle.

[Reference/native checks](../../tools/nuisc/tests/native_application_bridge/return_plain_tree.rs)
run all four with normal/reversed YIR inventories. The native probe observes
actual answer-callee owner/proof arguments without replacing source work.
Exact flushed effects stop before each failed call, accepted-call canaries remain intact,
and actual previous outputs feed event/close callbacks. This family contributes
eight healthy native processes, 56 fatal processes and 440 accepted callbacks.
The reference host retains its last published state, enters Faulted after a
bad event and refuses subsequent events. The source step does not compute
from its previous State, so this is not general state-dependent closure.

[The CLI workflow](../../tools/nuis/tests/native_session_workflow/effect_call_return_plain_tree.rs)
checks sixteen delivery images: four sources, each fresh/cache/two successive
source-free restores. Each image runs the 16-input healthy session before and
after three fatal event routes, contributing 640 accepted callbacks and
48 fatal CLI events. Failed events publish neither a new state nor a completion;
the last accepted event is deliberately different from the accepted open state.
The flushed native probe supplies exact failure-prefix evidence; fatal CLI
stdout may be buffered.

Payload, native objects, envelope, packaging policy and lifecycle are identical
within each variant across fresh/cache builds. Execution preserves payload and
container bytes. Both successive restores occur without source, manifest or
project cache, and preserve the original input artifact bytes.
Different source variants need not have identical binaries.
[AOT checks](../../tools/nuisc/src/aot_application_effect_call_return_plain_tree_tests.rs)
require nonempty generated-helper coverage, guard-first transitive Effect order,
explicit print grants and no owned aggregate allocator/drop. Removing Effect
edges or print policy rejects rather than silently accepting work.

## Scoped Acceptance

The 2026-10-15 serial macOS CPU cohort passes 480 distinct tests:
437 compiler, ten native/reference, three CLI, three host and 27 tensor.
Eight tests are new: four units, one AOT, two native/reference and one delivery.
After strengthening the new ancestor value oracle to +1, the affected five
compiler, two native/reference and one delivery tests are replayed. These
focused repeats and expanded rows are not additional distinct tests; unchanged
predecessor fixtures retain their full-cohort results.

Run serially from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields return_in_fields return_nested_in_fields return_prefix_in_fields return_sibling_in_fields return_tail_in_fields return_child_tail_in_fields return_pair_tail_in_fields return_triple_tail_in_fields return_quad_tail_in_fields return_quint_tail_in_fields return_paired_in_fields return_exit_prefix return_typed_exit_prefix return_plain_prefix return_plain_tree --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_paired_in_fields return_exit_prefix return_typed_exit_prefix return_plain_tree --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_typed_exit_prefix return_plain_tree --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, `active/99`.
The current validation command places its complete historical payload in a
quoted no-op heredoc. Historical notes and old commands remain literal data,
not extra arguments or executable shell text. The delimiter uses the preceding
command hash and is checked for collision; history hashing excludes only the
new heredoc framing. Shell syntax and a harmless-front replay verify isolation
without rerunning historical commands. Other five fields retain their original
complete-predecessor suffix convention.

All six complete preceding ordinary-prefix fields are retained, not summarized,
with these SHA-256 values:

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `dbedd181193b4bb37cd3573fc45ccb302add069eed1e8242d2b7f79b3ee66eb9` |
| `next_step` | `f0d3c903c54f31fa95087870a7c0b83a62e1eca50a24ef4b08f66f6388a15c0e` |
| `blocker` | `9176011bba578b7e5077c7eb26d34e88144780a01131f0526087f58406766b54` |
| `next_action` | `76880de047346bc00a7429dc424566751ada4bd9e57b33fd724695b82f86e8df` |
| `validation_command` | `95f68d6f991e463e869133976e70ae062ce6aba0780af4db373617e764e14f06` |
| `expected_artifact` | `1cfe3ba400f561441d17f3a6bff63c7c2e4e02e876c7fd019657507f64559b16` |

The rebuilt ordinary CLI exposes six source-equal fields and matching complete
preceding hashes. All 3159 drift checks / 22049 patterns, coverage, hierarchy,
task-card lineage, UTF-8, changed file caps/relative links, changed Rust
formatting and diff checks pass. Four untouched historical files still fail
whole-workspace formatting; strict Clippy was not run. The active/99 coordinate
is not a global completion or whole-workspace acceptance claim.

## Next Boundary

Ordinary bool const initializers in staged regions still have a separate
nonlogical-initializer veto. Next reuse the existing typed scalar/predicate
proof for fresh bool const values without broadening entering writes, sealed
rewrites, hidden logical call arguments or non-bool const admission.

No generalized field-arm exit discovery is added. Multiple/deeper field-arm
exits, resources, six-child live coverage, stateful/heterogeneous application
closure, GPU/Linux/Windows acceptance, stable ABI, formal memory safety,
performance and completed self-hosting remain outside this CPU receipt.

