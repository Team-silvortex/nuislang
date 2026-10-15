# Native Returning Exit Prefixes V1

This bounded CPU checkpoint follows the
[paired direct-exit receipt](nuis-native-paired-field-arm-exits-v1.md).
It admits continuing computation inside a selected source exit before the
explicit final bool return. One direct source exit per field arm and one
common non-exit tail remain the source boundary.

## Lowering Contract

[The exit parser](../../tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_in_fields.rs)
splits the returning branch into a continuing prefix and final return.
Whole-source preflight still bounds recursion before shape discovery.
A prefix may not introduce another return, a loop or an exit else branch.

[The shared prefix prover](../../tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_exit_prefix.rs)
uses one inherited certificate from the original continuing field arms.
Exit-local writes do not participate in certificate discovery.
The same helper serves paired and one-sided exits. Its staged computation
is installed inside the selected early-return plan, not before the exit test.
Empty prefixes retain the direct predicate representation.

Owner work and selections over already certified children use existing staging.
Flat child writes, entering writes, sealed/type-changing writes and exit-only
field authority still reject. Ordinary exit-local bindings do not become
continuation inputs; the final predicate consumes their current local versions.
The non-exit continuation and source selector are not copied. No field record
acquires a control flag, exit result or function-return authority.

Six unit tests cover both selector polarities, a one-sided exit, original
five-child structural transport, exact budgets and rejection cases.
The small fixture charges 24 original nodes and one logical edge; one fewer
node, expression or edge rejects. Inherited depth four passes and five
rejects. The 32-input cap and existing source/node/expression/depth limits
are unchanged. One AOT test verifies guard-first transitive Effect order,
print-policy rejection and absence of owned aggregate allocation/drop.

## Fresh Execution

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_exit_prefix.ns)
has an i64 owner and two private children, i32 and bool.
After a source exit is selected it advances the owner, evaluates an inner
choice, optionally updates both children and then evaluates its bool return.
The other path executes the common tail exactly once. The inner child operand
is zero when its selection is inactive; all exit-local operands are zero on
non-exit paths, and tail operands are zero on exiting paths.

[The independent case arithmetic](../../tools/nuisc/tests/native_application_bridge/return_exit_prefix_cases.rs)
covers 24 finite combinations of three seeds, both source arms, taken/untaken
exits and inner choices. At seed seven the updated owner permits a true exit;
updating the children produces a false exit. A false source return still skips
the non-exit tail. These witnesses are not formal verification or exhaustive
integer/IEEE coverage.

[Reference and native execution](../../tools/nuisc/tests/native_application_bridge/return_exit_prefix.rs)
check both selector-source variants and normal/reversed YIR inventories.
Native probes observe actual compiled answer-callee owner/i32/bool arguments
without replacing source computation. Healthy runs pass the real open result
into event input, then close the actual event result, checking output canaries,
full effect order and results. The reference route also preserves the last
accepted publication after an event fault and rejects reentry.

There are seven selected argument boundaries: source selection 11, exit test
14, exit owner 20, inner choice 21, child update 22, answer 28 and tail owner 25.
The shared child operand first traps at 22, not independently at marker 23.
The shared tail operand first traps at 25, not independently at marker 26.
Argument traps occur before callee entry. Flushed native partial traces are
compared exactly, including absence of post-failure callback output.

The new fixture contributes four healthy native processes, 28 fatal native
processes and 316 accepted native callbacks: four images each run 24 healthy
open/event/close triples and seven healthy opens before failing events.
The direct paired and quint-tail native/reference predecessors are freshly
replayed separately. Their previously recorded CLI fresh/cache/two source-free
acceptance remains historical: this is not a fresh CLI delivery replay of the
new prefix fixture.

## Scoped Acceptance

The 2026-10-15 serial macOS CPU cohort passes 463 distinct tests:
423 compiler, ten native/reference, three host and 27 tensor.
Nine are new: six units, one AOT, one reference and one native test.
Focused repeats are not additional distinct tests.

Run from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields return_in_fields return_nested_in_fields return_prefix_in_fields return_sibling_in_fields return_tail_in_fields return_child_tail_in_fields return_pair_tail_in_fields return_triple_tail_in_fields return_quad_tail_in_fields return_quint_tail_in_fields return_paired_in_fields return_exit_prefix --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_quint_tail_in_fields return_paired_in_fields return_exit_prefix --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, `active/99`.
Six complete preceding fields are retained after paired source-arm exit history
markers, not replaced by hash-only summaries. Their fresh SHA-256 values are:

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `3cc1b9fea69095419e2e3f2ea4c6f8eb001d77d18d6216e253065b6e7ffb8abe` |
| `next_step` | `f9cd07cbb884b419b730c75dc1fb878504205d9b0a8c5668ae9d8820a04ee7bf` |
| `blocker` | `dccd66cf5698344ec5da46091b729ba1d0a127276938cf66098d91d0eb8eeddd` |
| `next_action` | `540b28a6927e2e6104a86c53c2162eecfd2808702c727200c0dec36fe66963c4` |
| `validation_command` | `2ba01532fb46a336084634abb2de28a8af22ad354cb228db1d78b0fab80c64b0` |
| `expected_artifact` | `51645f14c757228341c6f13abbf17ef58961a046e95433f4ff7923626bc8ea08` |

The rebuilt ordinary CLI exposes six source-equal fields and matching complete
predecessor hashes. All 3124 drift checks / 21858 patterns, coverage, hierarchy
and task-card lineage pass. Changed Rust formatting, UTF-8, source/test/document
caps, relative links and diff whitespace pass. Whole-workspace formatting
still reports four untouched historical files; strict Clippy was not run.

## Next Boundary

The new prefix fixture has two private children. Its five-child proof is
structural, not five-child native prefix execution. Next add an asymmetric
five-child typed prefix fixture with raw current-field/owner evidence, true and
false exits, lazy independent argument traps, CLI fresh/cache and two successive
source-free delivery checks. Keep the paired direct-exit history independent.

More than one exit per arm, deeper/general exit discovery, flat child writes,
ordinary-local exports and generalized resource transport remain separate.
This receipt does not certify GPU/Linux/Windows execution, heterogeneous
application closure, ABI stability, formal memory safety, performance,
completed self-hosting or whole-workspace formatting.
