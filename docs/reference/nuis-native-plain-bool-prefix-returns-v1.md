# Native Plain Bool Prefix Returns V1

This bounded CPU checkpoint follows the
[typed-prefix delivery receipt](nuis-native-typed-returning-prefixes-v1.md).
A nonempty prefix no longer requires a syntactic logical return suffix to
enter the existing bool/effect proof.

## Eligibility And Authority

[The prefix entry](../../tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_prefixes.rs)
still requires an explicit final return in both arms and a nonempty prefix in
at least one arm. It now delegates bool admission to the existing predicate
prover instead of first requiring a Binary And/Or return expression.
The parent exact-bool return-type check, signature/type inspection, impure-call
requirement and capture bounds remain unchanged. Pure selections and the
separate no-prefix/direct-return route are not broadened by this patch.

The change reuses existing prefix/field staging and plan installation.
It does not discover entering/root/bool write authority, export ordinary
locals, reset source budgets, add record control flags or change backend
effect/print admission. The original certificate remains the only authority
for owner/private child updates.

[Four new unit tests](../../tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_plain_prefix_tests.rs)
cover ordinary calls, bool literals, local-variable returns, invalid non-bool
NIR returns/calls and atomic root/seal/flat-child/extra-exit vetoes.
The 31-capture-plus-guard case passes; 32 captures reject. Pure-call-only
prefixes and two empty prefixes remain outside this entry.
Two old rejections based solely on logical-return syntax are now explicit
positive regressions: both an effectful logical initializer and an effectful
private-bool update may feed a plain local-variable return. Other rejection
checks remain, and unexpected update admission now prints the source body.

The five-field plain-call body charges 39 source nodes and
zero original logical edges. One fewer node/expression rejects, inherited depth four passes
and five rejects, and generated parameters remain at most 32.
The wrapped predecessor continues to check its original one-edge budget.
No artificial edge is charged for generated gating.

## Plain And Wrapped Sources

[The new Nuis fixture](../../tools/nuisc/tests/native_application_bridge/return_plain_exit_prefix.ns)
returns answer_work directly after its common tail, without an && true wrapper.
[The wrapped fixture](../../tools/nuisc/tests/native_application_bridge/return_typed_exit_prefix.ns)
is retained unchanged. Each has an arm-swapped, selector-inverted source form;
four distinct source variants are freshly exercised.

The same independent case arithmetic is shared across variants, not changed
to fit a new expected outcome. It retains 160 rows: 64 finite controls,
64 finite stale-operand witnesses and 32 paired IEEE rows.
Six independent stale-operand arithmetic counterfactuals change the result;
the real compiled source and actual raw answer-callee tuples are observed
separately. This is not mutated-compiler execution or exhaustive input proof.
Native IEEE rows cover signed zeros, subnormals, infinities and quiet/signaling
NaNs, with i32 sign extension and exact f32/f64 words.

## Native And Delivered Execution

[The expanded reference/native family](../../tools/nuisc/tests/native_application_bridge/return_typed_exit_prefix.rs)
runs all four sources with normal/reversed YIR inventories. Both true and false
source exits, inactive zero proof operands, current fields, exact flushed
source-position failure cuts, actual prior-output transport and canaries pass.
The same twelve argument boundaries are independently poisoned.
Tail-work marker 26 still shares operand 25; baseline markers do not acquire
new independent operands.

The typed family now contributes eight healthy native processes, 96 fatal
processes and 3936 accepted callbacks. Exactly half belong to plain sources
and half to freshly replayed wrapped sources.
Two-child prefix and direct paired-exit native/reference tests are also
freshly replayed, without relabeling older CLI histories.

[The expanded CLI workflow](../../tools/nuis/tests/native_session_workflow/effect_call_return_typed_exit_prefix.rs)
checks sixteen delivery images: four sources, each fresh/cache/two successive
source-free restores. Each image runs its 22-row finite/witness healthy session
before and after three fatal event routes. The typed family contributes
48 CLI event traps and 832 accepted callbacks; half belong to plain sources.
CLI delivery checks finite inputs, not raw NaN payload serialization.
Exact partial effects belong to the native probe because fatal CLI stdout
may be buffered.

Executable/native-object/envelope/packaging-policy/lifecycle identities remain
exact within each source variant across fresh/cache builds. Execution preserves
payload/container bytes; source-free restores preserve the input artifact.
Source, manifest and project cache are absent before both restores.
Different source variants are not required to have byte-identical binaries.
AOT checks keep guard-first transitive Effect order, explicit print grants,
and no owned aggregate allocator/drop.

## Scoped Acceptance

The 2026-10-15 serial macOS CPU cohort passes 472 distinct tests:
432 compiler, eight native/reference, two CLI, three host and 27 tensor.
Four units are new; the existing typed AOT/reference/native/CLI bodies are
expanded to cover the two new plain source variants. Expanded rows and focused
repeats are not additional distinct tests.

Run from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields return_in_fields return_nested_in_fields return_prefix_in_fields return_sibling_in_fields return_tail_in_fields return_child_tail_in_fields return_pair_tail_in_fields return_triple_tail_in_fields return_quad_tail_in_fields return_quint_tail_in_fields return_paired_in_fields return_exit_prefix return_typed_exit_prefix return_plain_prefix --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_paired_in_fields return_exit_prefix return_typed_exit_prefix --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_typed_exit_prefix --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, `active/99`.
Six complete preceding fields remain after typed returning-prefix history
markers, with fresh SHA-256:

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `31357cf568d2d13dfb03f8b011bf1ebb5a8112cea7b2ef5e3d2b56dab6bf4b1f` |
| `next_step` | `29781349ae3a145ef843b4b41a15678ab9ee1ba02e9e8f3a84ae80718e694cc3` |
| `blocker` | `e8b986439914fe4730746abfadd519d4c05bf73340bb15036eb62dc20c984f26` |
| `next_action` | `cb33b62c322c856f3b9acafa2f37d917deb4bea681c03400de5ef041c9ceca02` |
| `validation_command` | `2a8af43a7cadc45fe98d5cb4aba341a1f8b0b6daf09e3311bdac39d624a6c5fc` |
| `expected_artifact` | `65b15f5e9c151c878d642fb3d251aa1505a4afb91d308b50d39f0890afff39e2` |

The rebuilt ordinary CLI exposes six source-equal fields and matching complete
preceding hashes. All 3149 drift checks / 21991 patterns and coverage, hierarchy,
task-card lineage, UTF-8/caps/relative links, changed Rust formatting and diff
checks pass. Whole-workspace formatting still reports four untouched historical
files; strict Clippy was not run.

## Next Boundary

The separate full-return-tree route still has its own logical-leaf eligibility
condition. Next prove bounded complete bool-return trees with ordinary
call/literal/name leaves using the existing type/effect proof, without treating
a logical leaf as source authority. That target is not arbitrary deeper exit
discovery inside a field arm, nor a claim that all ordinary bool functions or
direct returns are unsupported elsewhere in the compiler.

More than one exit per field arm, generalized resource transport, six-child
live coverage, stateful/heterogeneous application closure, formal memory safety,
GPU/Linux/Windows, stable ABI, performance and completed self-hosting remain
outside this local receipt and its whole-workspace formatting scope.
