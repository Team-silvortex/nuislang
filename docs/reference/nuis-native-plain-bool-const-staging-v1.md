# Native Plain Bool Const Staging V1

This bounded CPU checkpoint follows the
[ordinary complete-tree receipt](nuis-native-plain-bool-return-trees-v1.md).
A fresh bool const no longer needs a syntactic logical initializer to enter
the existing staged bool/effect proof.

## Admission And Seals

[Region staging](../../tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_regions.rs)
now routes every const initializer through the existing bool predicate prover.
Ordinary leaves use Predicate::Scalar; logical initializers retain their
existing guarded path. Exact declared/inferred bool types, fresh names,
current lexical reads and synchronous scalar signatures are still required.
This does not admit numeric/ref/optional/generic const values or introduce a
general compile-time constant evaluator. These are local immutable bindings
with existing runtime initializer/effect semantics.

Const nodes remain NirStmt::Const on installation. A direct four-constant
stage installs unchanged, creates no helpers/names/structs, consumes exactly
four source nodes/seven expressions and zero original logical edges.
A node or expression shortfall rejects; 16/17 prefix, 31/32 capture,
31/32 expression-depth and 8/9 selection-depth tests retain their bounds.
The shared tree node/expression/logical budgets and guarded parent ABI do not
reset or expand because an initializer is ordinary.

[Four new unit tests](../../tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_plain_bool_const_tests.rs)
cover literals, entering names, comparisons, ordinary bool-returning calls,
initializer-only effects, fresh seals in ancestor/leaf scopes and exact bounds.
One previous syntax-only rejection is now an explicit positive. Pure-only
selections, same-name/inherited writes, sealed rewrites, unsupported non-bool
initializers, hidden logical call arguments, late leaks/missing values,
invalid declared types and async signatures still reject atomically.
Both the module and generated-name inventory remain unchanged on rejection.

This is bounded staging authority, not a global immutability proof.
The outer writes_logical_constants preflight is unchanged; its ordinary-RHS
sealed-root coverage is the next audit, not silently treated as accepted.

## Real Execution

[The Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_plain_const.ns)
uses seven ordinary bool const declarations in ancestor and leaf scopes.
Literal enabled/input bindings feed an effectful sealing call, a call result
feeds a leaf constant, and copied/literal bools feed the opposite branch.
The ancestor i64 transforms by +1; sealing flips the input flag.
All four leaf paths can produce the expected true/false outcomes.

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_plain_const_cases.rs)
enumerates 16 finite inputs. Two stale-i64-input and two unflipped-bool
counterfactuals change the expected answer; these are arithmetic witnesses,
not mutated-compiler execution. Native observations additionally read real
answer-callee owner/proof arguments. Boolean call-result consumption is
proved by the unmasked result cases, not claimed as raw seal-callee probing.

Original/mirrored ordinary initializer sources and their logical-wrapper
counterparts provide four distinct source variants with the same semantics.
Every inactive proof operand is zero. There are eight independently poisoned argument boundaries,
including the new sealing initializer. Unused selected work remains observable.

[Reference/native checks](../../tools/nuisc/tests/native_application_bridge/return_plain_const.rs)
exercise normal/reversed YIR inventories. The new family contributes eight
healthy native processes, 64 fatal processes and 448 accepted callbacks.
Accepted-call canaries, actual prior-output transport, real callee arguments
and exact flushed failure prefixes are checked. Reference events retain the
last state, enter Faulted and refuse further events before close.
The previous State is transported, not consumed in the business computation.

[AOT checks](../../tools/nuisc/src/aot_application_effect_call_return_plain_const_tests.rs)
keep nonempty helper coverage, guard-first transitive Effect dependencies,
explicit print grants and no owned aggregate allocator/drop. Removing effect
edges or print permission rejects.

## Delivery And Acceptance

[The CLI workflow](../../tools/nuis/tests/native_session_workflow/effect_call_return_plain_const.rs)
checks sixteen delivery images: four sources, each fresh/cache/two successive
source-free restores. Each image runs a 16-input healthy session before and
after four selected event traps, including the sealing initializer. The new
family contributes 672 accepted callbacks and 64 fatal CLI events.
Failed events publish neither a state nor completion; the last accepted event
differs from open. Flushed native probes supply exact failure-prefix evidence,
because fatal CLI stdout can be buffered.

Executable/native-object/envelope/packaging-policy/lifecycle identities remain
exact within each variant across fresh/cache builds. Execution preserves
payload/container bytes, restores preserve the input artifact, and source,
manifest and project cache are absent before both restores.
Different source variants are not required to share identical binaries.

The 2026-10-15 serial macOS CPU cohort passes 488 distinct tests:
442 compiler, twelve native/reference, four CLI, three host and 27 tensor.
Eight tests are new: four units, one AOT, two native/reference and one CLI.
The initializer-only syntax regression is migrated rather than counted as
another test. The new native probe is explicitly gated to 64-bit macOS/Linux;
the two affected native/reference tests are replayed after this test-only gate.
That focused replay is not additional distinct evidence or Linux acceptance.

Run serially from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields return_in_fields return_nested_in_fields return_prefix_in_fields return_sibling_in_fields return_tail_in_fields return_child_tail_in_fields return_pair_tail_in_fields return_triple_tail_in_fields return_quad_tail_in_fields return_quint_tail_in_fields return_paired_in_fields return_exit_prefix return_typed_exit_prefix return_plain_prefix return_plain_tree return_plain_const --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_paired_in_fields return_exit_prefix return_typed_exit_prefix return_plain_tree return_plain_const --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_typed_exit_prefix return_plain_tree return_plain_const --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, `active/99`.
Six complete preceding ordinary-tree fields are retained, with fresh hashes.
The validation command again frames its entire previous payload in a quoted
no-op heredoc; nested historical commands remain literal, nonexecuting data.
Hashing excludes only this new framing, not any predecessor content.

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `bd05cc0d4f37c7258972a64065d24a2c7dab2e4e16adb412f172d660cbd87e7b` |
| `next_step` | `c93fafdadc75722b56d947a6804cd6ddb2efee3fcf28b80d907f214c41ac0975` |
| `blocker` | `a6bb39d14bc0a3edf2219dbb3dd2c337e42e7284c08e971503aac3ba3432a188` |
| `next_action` | `dccad95d79816a4b82a7506a541fddbc20d0b8be4add606b46f12728303d0543` |
| `validation_command` | `c74b548df1db5bc1400730d0d0089b1b2eeae073c616109f7b33e21e038c1ffe` |
| `expected_artifact` | `0266b34d7e595a375ee60d842411c572d7de8a0d10f848886472e198d1c6e8fd` |

The rebuilt ordinary CLI exposes six source-equal fields with matching complete
preceding hashes. All 3172 drift checks / 22121 patterns, coverage, hierarchy,
task-card lineage, UTF-8, changed caps/relative links, changed Rust formatting
and diff checks pass. Shell syntax and a harmless-front replay prove literal
command-history isolation. Four untouched historical files still fail
whole-workspace formatting; strict Clippy was not run. Active/99 is not global
completion or whole-workspace acceptance.

## Next Audit

Audit sealed top-level binding writes for ordinary RHS forms as well as
logical forms in the outer effectful-selection preflight. Require atomic
vetoes, nested/one-sided/two-sided cases, no false vetoes of unrelated fresh
consts and the same bounded traversal before widening any stateful proof.
Actual state-dependent bool computation remains a later proof.

General const evaluation, resources, new write authority, multiple/deeper
field-arm exits, six-child live coverage, full stateful/heterogeneous closure,
GPU/Linux/Windows acceptance, stable ABI, formal memory safety, performance and
completed self-hosting remain outside this local CPU receipt.
