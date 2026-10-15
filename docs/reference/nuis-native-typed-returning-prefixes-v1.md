# Native Typed Returning Prefixes V1

This bounded CPU receipt follows the
[exit-local prefix proof](nuis-native-returning-exit-prefixes-v1.md).
The earlier five-child proof was structural. This follow-up executes
asymmetric five-child prefixes and their delivered images without changing
production field authority, source discovery or backend admission.

## Contract And Source

[The Nuis application](../../tools/nuisc/tests/native_application_bridge/return_typed_exit_prefix.ns)
carries an i64 owner plus private i32/f32/f64/bool/bool children.
Its continuing arms establish one inherited certificate before either exit.
A selected returning prefix may update that owner and those certified children;
it cannot use its own writes to discover new authority.

The yes prefix updates i32, f32, f64, bool and peer-bool in that order.
The sibling prefix updates f64 and bool before i32/f32/peer-bool.
The f32 operation consumes current i32, f64 consumes the current f32 available
at its actual source position, and peer-bool consumes the current bool.
The two owner helpers also have different increments and distinct markers.
Mirrored source swaps both physical arms and inverts the once-only selector.

A non-exit path retains one common tail. Records contain six data fields only.
[Four unit tests](../../tools/nuisc/src/lowering/buffer_loop_outline/effectful_selections_return_typed_exit_prefix_tests.rs)
check this layout, once-only selection/tail, capture isolation, the unchanged
32-input bound and atomic proof rejection. Type-invalid argument propagation
may reject in the frontend before outlining; the test checks that diagnostic.
The fixture has 21 open/event inputs and one bool State word.
Its bounded proof charges 39 original nodes and one logical edge.
One fewer required node/expression/edge rejects; inherited depth four passes
and five rejects. No new per-prefix budget or field-write token is created.

## Current Values And Execution

[The independent arithmetic](../../tools/nuisc/tests/native_application_bridge/return_typed_exit_prefix_cases.rs)
provides six independent stale-operand witnesses: replacing only the current
owner, i32, f32, f64, bool or peer-bool with its pre-prefix value changes the
return result. These are arithmetic counterfactuals, not mutated-compiler
executions. All witness inputs also run through the real compiled source;
native probes observe the complete current tuple at the answer callee.

The 160 input rows comprise 64 finite control/seed/bool rows, 64 finite
current-value witness rows and 32 paired IEEE rows. They are not a Cartesian
product or an exhaustive input-space claim. Native IEEE rows cover both
signed zeros, subnormals, infinities and quiet/signaling NaNs for f32/f64.
Probes preserve raw f32 words by zero extension and raw f64 words by bitcast;
comparison results alone are not used as evidence of bit preservation.

[Reference/native tests](../../tools/nuisc/tests/native_application_bridge/return_typed_exit_prefix.rs)
check original/mirrored source and normal/reversed YIR inventories.
Both arms produce true and false early returns. Non-exit, unselected arm,
and inactive inner-selection proof operands remain zero and do not execute.
Twelve source argument boundaries are independently poisoned:
selection 11, exit condition 14, yes owner 20, sibling owner 30, inner choice 21,
i32 22, f32 23, f64 24, bool 27, peer-bool 29, answer 28 and tail owner 25.
The tail-work marker 26 shares operand 25; baseline markers 12/13/15/16/17
do not have independent proof operands.

[The word-session probe](../../tools/nuisc/tests/native_application_bridge/word_session_probe.rs)
accepts oracle input/state/trace rows independently of the language field types.
It inserts observations into the real compiled callee, never replaces source
computation, copies actual previous outputs to event/close inputs and checks
output canaries. Exact flushed failure prefixes are cut by source positions,
before the poisoned callee is entered, not by raw field words equal to markers.
The reference route additionally checks last accepted publication, faulted
session state, denied reentry and failed completion after close.

The new native family contributes four healthy processes, 48 fatal processes
and 1968 accepted callbacks: four images each execute 160 open/event/close
triples and twelve healthy opens before failing events.
Two-child prefix and direct paired-exit native/reference predecessors are
freshly replayed separately. Older CLI histories are not relabeled as fresh.

## Delivered Images

[The CLI workflow](../../tools/nuis/tests/native_session_workflow/effect_call_return_typed_exit_prefix.rs)
checks eight delivery images: original/mirrored source, each built fresh,
loaded from a cache hit and restored twice without source.
Each healthy session batches 16 finite control rows and six stale-operand
witness rows as open plus 21 events and close. It is run before and after
three selected fatal event routes (owner 20, sibling owner 30 and answer 28).
A true open result and a false accepted event distinguish the last publication.
Fatal CLI stdout may be buffered; exact partial effects belong to native probes.

The eight images contribute 24 CLI event traps and 416 accepted callbacks.
Executable bytes, native objects, packaging policy/envelope and lifecycle
match across fresh/cache builds. Execution preserves payload/container bytes.
Source, manifest and project cache are removed before both source-free
restores; restoration preserves the input artifact bytes. The CLI uses
finite values, not raw NaN payload transport, for this delivery checkpoint.

## Scoped Acceptance

The 2026-10-15 serial macOS CPU cohort passes 468 distinct tests:
428 compiler, eight native/reference, two CLI, three host and 27 tensor.
Nine are new: four units, one AOT, two reference/native and two CLI.
A fresh rebuilt CLI audit clears the removed unused import; the accepted
workflow body is unchanged. Focused repeats do not add to the distinct count.

Run from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool return_i64_ancestor_private_bool return_float_ancestor_private_bool return_dual_i64_private_pair return_i64_float_private_pair return_float_i64_private_pair return_same_float_private_pair return_cross_float_private_pair return_i32_i64_private_pair return_i64_i32_private_pair return_i32_i32_private_pair return_i32_float_private_pair return_float_i32_private_pair return_private_fields return_retained_private_fields return_numeric_field_vector return_bool_field_vector return_two_bool_field_vector return_float_owner_field_vector return_cross_width_field_vector return_multi_float_field_vector return_early_fields return_in_fields return_nested_in_fields return_prefix_in_fields return_sibling_in_fields return_tail_in_fields return_child_tail_in_fields return_pair_tail_in_fields return_triple_tail_in_fields return_quad_tail_in_fields return_quint_tail_in_fields return_paired_in_fields return_exit_prefix return_typed_exit_prefix --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_paired_in_fields return_exit_prefix return_typed_exit_prefix --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_typed_exit_prefix --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_typed_exit_prefix_cli_audits_lazy_proofs_and_current_value_witnesses --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, `active/99`.
Six complete preceding fields are retained after returning exit-prefix history
markers, with these fresh SHA-256 values:

| Field | Complete Predecessor SHA-256 |
| --- | --- |
| `evidence` | `2b226be1bd858c328b491283fb2806dbf5b4b78ecb6534e1576a21abf124d3d7` |
| `next_step` | `02b29e33c38449f20f561537379259c8b94e30f7518f3b1d7edbc44735153c4b` |
| `blocker` | `c73e71b767ff3dd684ea2a302615135bcc428862ff1a8f62fab9a2d38ae3ef52` |
| `next_action` | `5a31a6b0fcb7131897bf045d1b912bc5295895194870471e561063b5ec432b94` |
| `validation_command` | `2730e712809c8162ce3f4a2396261e6f0ae1792e502a5ffd686e8463d7209c48` |
| `expected_artifact` | `2798479ac0eacc3cec5c230073776616444f6a3e54b5752311de2652a6e71f6f` |

The rebuilt ordinary CLI exposes six source-equal fields and six matching
complete preceding hashes. All 3137 drift checks / 21934 patterns, coverage,
hierarchy and task-card lineage pass. Changed Rust formatting, UTF-8,
source/test/document caps, relative links and diff whitespace pass.
Whole-workspace formatting still reports four untouched historical files;
this is not strict-Clippy acceptance.

## Next Boundary

The accepted fixture still uses a logical common suffix ending in
`answer_work(...) && true`. Next prove an ordinary bool-call suffix through
the bounded prefix route without relying on that syntactic logical wrapper.
This is an eligibility/coverage target, not a claim that every ordinary bool
function in the compiler is unsupported. Keep inherited certificate authority,
selected-only effects, true/false exits, one tail and exact bounds.

More than one exit per arm, deeper/general exit discovery, six-child live
coverage, ordinary-local exports and generalized resources remain separate.
This local receipt does not certify GPU/Linux/Windows, heterogeneous
application closure, ABI stability, formal memory safety, performance or
completed self-hosting or whole-workspace formatting.
