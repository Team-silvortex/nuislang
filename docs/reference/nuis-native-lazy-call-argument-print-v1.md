# Native Lazy Call Argument Print V1

This bounded CPU worktree follow-up continues the
[computed argument proof](nuis-native-computed-call-argument-print-v1.md) on Git
beta-0.17.9 at `2275b6af`. It changes no release version, ABI, source intrinsic,
runtime wrapper or backend-specific producer policy.

## Scalar RHS Captures

The [shared typed plan](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs)
now admits fallible integer division/remainder in bool RHS arguments of completed
pure float calls. Every leaf still needs its original owned-field certificate
or independent ordinary readiness. A capture-path union accompanies each type:
non-total logical right trees share a limit of 31 scalar captures, plus the
existing gate slot. Repeated paths deduplicate; nested edges do not reset the
union. Depth/work, exact parameter positions, aggregate print logical-edge limits
and the sealed generic fallback remain independent checks.

The [capture builder](../../tools/nuisc/src/lowering/buffer_loop_outline/logical_field_captures.rs)
materializes only exact scalar variables and Var-rooted field paths. Qualified,
unknown, non-scalar and computed-root paths fail transactionally. It supports all
five primitive scalar kinds, deterministic first-use order and hygienic names,
including when every field of a record is used. Calls, division and remainder
remain in the rewritten RHS body, never in eager capture arguments.

The [existing logical outliner](../../tools/nuisc/src/lowering/buffer_loop_outline/logical_conditions.rs)
retains its leading guarded helper and neutral-false/complement protocol for both
logical operators. Loop print values now pass through this same normalization;
previously only local bindings did, so an unselected checked RHS could execute.
Private float word/sign wrappers and supported i32 word casts are traversed too,
so wrapping a selector in negation cannot hide its logical arguments.

Projected captures use the already-certified scalar read paths. They do not mark
the whole record available, authorize a user-visible record argument, inherit an
ancestor epoch or move a checked computation across its guard. Independently
ready source routes keep their own authority. Legacy non-projectable helper
captures retain their separate existing fallback, not a new source read grant.

Three [capture controls](../../tools/nuisc/src/lowering/buffer_loop_outline/logical_field_captures_tests.rs)
cover exact kinds/order/dedup/hygiene, complete union/depth/work bounds, rejection
and rollback, all-fields-used scalar signatures and checks after the guard.
Three [epoch controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/lazy_call_argument_print_epoch_tests.rs)
cover the 31/32 capture boundary, original and skipped invalid reads, raw IEEE
execution for both short-circuit edges, nested poison, both induction orders,
sign wrappers and selected zero/overflow failures. Earlier unsupported-RHS
controls now use genuinely unsupported integral-return calls; ordinary read and
generic fallback restrictions are not removed merely because checks are lazy.

## Execution And Delivery

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/lazy_call_argument_print_epoch.ns)
uses parent AND, child OR and nested OR predicates, versioned bool fields,
original parent/child integer fields and outer float negation. Carry updates and
checked final State publication remain separate from printed selector values.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/lazy_call_argument_print_epoch_cases.rs)
evaluates only the reached checked integer operands. It tracks the original and
updated bool field versions, wrapping integer carries, exact-width floating
operations and the final sign wrapper. Native probes inspect actual ready/guard
inputs, selector float/bool/i64 arguments and final raw State. Arithmetic NaNs
normalize only in test observers; non-NaN results and input/carry bits remain exact.

[Native/reference execution](../../tools/nuisc/tests/native_application_bridge/lazy_call_argument_print_epoch.rs)
uses 19 raw healthy sequences per width and eighteen failure rows. Healthy rows
include skipped zero poison, skipped nested remainder poison and zero trips.
Failures distinguish parent/child/later-parent checks, selector/tail failures,
MIN/-1 division/remainder and a later updated-field remainder overflow. Exact
reached prefixes, canaries, normal/reversed YIR, scalar-call counts and last
accepted State remain checked. Owned-record work entries are pinned in the full
native word stream, not counted as nonexistent scalar reference events.

Success uses 12 loop work, 512 helper entries and at most 32 functions. Separate
128-entry NaN controls retain atomic exhaustion rejection without widening a
producer budget. The [finite CLI matrix](../../tools/nuis/tests/native_session_workflow/lazy_call_argument_print_epoch.rs)
uses six healthy sequences per width before and after all eighteen failures on
fresh/cache and two independent source-free images. Binary/container, host-object,
lifecycle, envelope and packaging identities remain pinned. Decimal CLI input
is not arbitrary NaN-payload evidence.

## Validation

The locked serial validation chain passed 675 distinct tests: 303 targeted
compiler, 114 native/reference, one CLI delivery, all 178 LLVM library tests,
three packaging, 17 buffer-loop, three host-failure, 26 maintenance-script,
one absolute-path policy, two file-limit and 27 tensor tests. Ten new controls
are included; focused repetitions count only once. The expanded compiler gate
also runs adjacent ready-value and capture-projection controls. The LLVM suite
includes the normally ignored host-clang record round trip at O0 and O2; this
does not claim that every workspace suite or physical backend ran.

Before the print-normalization repair, unselected divide poison reached an actual
divide-by-zero failure. After the repair, raw source/NIR and native/reference
matrices passed, including the sign-wrapped and nested cases. Both widths passed
fresh/cache and two independent source-free deliveries, with six healthy sequences
before and after all eighteen failures. Formatting, whitespace, delivery-test
compilation, the ordinary CLI build, locked offline metadata, 4,436 local
documentation links and 5,863 UTF-8 files passed.

The rebuilt ordinary CLI reports all 3,925 drift checks clean, together with
coverage, manifest, milestone, hierarchy and task-card lineage validation. Three
format-sensitive locator patterns were updated without weakening their semantic
checks; all 27 tensor tests then passed. All six live selected fields exactly
match source and retain complete predecessor values and SHA-256 digests.
Historical command syntax and inertness checks passed through stdin without
executing prior chains. The selected task stays ready and active/99; bounded CPU
acceptance is not GPU/Linux parity, stable ABI, a performance target or self-hosting.

## Remaining Frontier

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99. The next
selected gate is completed pure bool/i64-return helper operands in lazy RHSs,
with exact scalar captures and transitive purity. They do not gain field-read
permission from a return type alone. Float comparisons, i32 arithmetic, casts,
resource/I/O/async capture, full native ns-nova, stable ABI, backend parity and
self-hosting remain separate gates.
The later [helper call-argument proof](nuis-native-helper-call-argument-print-v1.md)
advances completed pure auxiliary operands without changing this checkpoint's
historical totals or source limits.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- lazy_call_argument_print_epoch lazy_field_capture ready_values:: --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- lazy_call_argument_print_epoch computed_call_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- lazy_call_argument_print_epoch --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete preceding fields remain losslessly embedded in the tensor.
Historical command chains are quoted here-document data consumed by a no-op,
checked for syntax/inertness through stdin without executing prior chains.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `089bfda99bd55cae127a3f9cc61a675104a3834235805be800387976f4252be2` |
| `next_step` | `3f83139e04ade37acf561c63f122eb4a18f819dcf151676b048d24ec94db6bd9` |
| `blocker` | `2b0c0986bc4020679c5a869f23fd7f5e36220463acb381297b9e0df37dfb7f96` |
| `next_action` | `27edfb60b5c5ff4e7d93919355a1cf7dd64308647af1c5111a5305cd60422078` |
| `validation_command` | `0e9f5620109374e9587a3b0dcaf5100e10b4243785e13c9f417007fca0397608` |
| `expected_artifact` | `6d2286990b87a7a88dd2899bdfd3369145c43f54439bb096d608e56994bbad36` |

