# Native Computed Call Argument Print V1

This bounded CPU worktree follow-up continues the
[pure float-call proof](nuis-native-float-call-field-epoch-print-v1.md) on Git
beta-0.17.9 at `2275b6af`. No release version, ABI, source intrinsic, runtime
wrapper or backend-specific producer rule changes.

## Typed Argument Proof

The [shared plan](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs)
now derives exact operand types in one bounded postorder traversal. Completed
pure f32/f64-return calls retain exact parameter positions; arguments may compose
existing i64 arithmetic, integer comparisons and bool logic over original
certified fields and independently ready leaves. Mixed kinds, casts, i32
arithmetic, integral-return calls, float comparisons and whole-record arguments
receive no new permission. Float arithmetic/sign and nested pure-call support
remain compositional rather than enumerated expression templates.

The [field query](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_field_reads.rs)
returns the original nominal leaf type only after the same entry, ownership and
layout/path certificate succeeds. Existing exact projection queries filter this
shared result. The [epoch](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch.rs)
never changes ambient availability, ordinary initializer/condition authority or
ancestor grants.

A separate totalness fact accompanies each type. Fallible logical RHS work remains
closed to this epoch proof: division, remainder and calls cannot become speculative right operands
of `&&` or `||`. The left operand is always reached; total typed right trees
may use certified fields and non-failing arithmetic/comparisons. Fallible logical RHS
support still requires its own lazy capture/read proof. Boolean equality keeps
its existing atom rule; boolean XOR is not newly admitted.

Rejected float-call plans cannot recover epoch reads through the generic call
fallback, even with an already-ready float first argument or private word packing.
Fallback uses independently ready ordinary values only; their separate authority
is not suppressed. A red-before-fix regression pins this read-boundary repair.

The [shared shape guard](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_prefix.rs)
opens logical roots only at exact bool parameters of completed pure float calls
in the explicit print profile. Ordinary prefix, return-tree and record-field
routes keep their default closed argument policy. The same aggregate work/depth
and 32-logical-edge limits span print roots, including branches/children; the
argument proof shares work 4096 and depth 64 across all operand kinds and calls.
The source tree is rendered without replay or reassociation.

Six [compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/computed_call_argument_print_epoch_tests.rs)
cover arithmetic/comparison kinds, exact i32/i64/bool parameters, root/child
permissions, ordinary reads, totalness rejection, scoped shape/edge budgets,
source-order leaf checks and raw NIR/IEEE execution in both induction orders.
Legacy default shape tests remain independent controls.

## Execution And Delivery

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/computed_call_argument_print_epoch.ns)
calculates integer and bool arguments from parent/child record fields and actual
session inputs, then calls a pure float selector. Separate carry sign updates
and checked final State publication are retained.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/computed_call_argument_print_epoch_cases.rs)
uses checked integer division/remainder and wrapping arithmetic, actual-width
float operations and source-ordered call prefixes. Native probes inspect the
selector's actual float, bool and integer arguments, raw ready/guard inputs and
raw final State. Arithmetic NaNs normalize only in observers, while non-NaN
results and carried State/input bits remain exact. Marker-valued words cannot
substitute for structural call counts.

[Native/reference tests](../../tools/nuisc/tests/native_application_bridge/computed_call_argument_print_epoch.rs)
use 19 raw healthy sequences per width and fourteen failure rows. These include
argument-expression divide-by-zero, helper guard failures, later child/tail
failures, and reached MIN/-1 division/remainder overflow. Healthy rows cover
changed selector decisions, skipped poisoned child guards and zero-trip poison.
No selector entry is permitted after a failing argument; exact reached prefixes,
canaries, normal/reversed YIR, scalar-call counts and last accepted State remain
checked. Owned-record work entries use the complete native word stream rather
than nonexistent scalar reference-call events.

Success uses 12 loop work, 512 helper entries and at most 32 functions. Separate
128-entry NaN controls preserve atomic exhaustion rejection; no producer budget
is widened. The [finite CLI matrix](../../tools/nuis/tests/native_session_workflow/computed_call_argument_print_epoch.rs)
uses six healthy sequences per width before and after all fourteen failures on
fresh/cache and two independent source-free images. Binary/container, host-object,
lifecycle, envelope and packaging identities remain pinned. Decimal CLI input
is not arbitrary NaN-payload evidence.

## Validation

The final locked serial chain passed 636 distinct tests: 267 targeted compiler,
111 native/reference, one CLI delivery, all 178 LLVM library tests, three
packaging, 17 buffer-loop, three host-failure, 26 maintenance-script, one
absolute-path policy, two file-limit and 27 tensor tests. Ten new controls are
included; focused repetitions count only once. The LLVM suite includes the
normally ignored host-clang record round trip at O0 and O2. This is not a claim
that every workspace suite or physical backend ran.

The ready-float fallback regression failed before the repair and passed in the
final compiler chain. The repaired source then passed all 111 native/reference
controls and a second full CLI matrix: both widths, fresh/cache and two independent
source-free images, six healthy sequences before and after all fourteen failures.
Formatting, whitespace, delivery-test compilation, the ordinary CLI build, locked
offline metadata, 4,423 local documentation links and 5,854 UTF-8 files passed.

The rebuilt ordinary CLI reports all 3,909 drift checks clean, together with
coverage, manifest, milestone, hierarchy and task-card lineage validation. All six
live selected fields exactly match source and retain their complete predecessor
values and SHA-256 digests. Historical command syntax and inertness checks passed
through stdin without executing prior chains. The selected task remains ready
and active/99; this CPU checkpoint does not certify GPU/Linux parity, stable ABI,
performance targets or self-hosting.

## Remaining Frontier

The coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99. The next
selected gate is lazy fallible bool RHS arguments over original certified fields,
without whole-record capture/read grants or speculative checked work. General
resource/I/O/async capture, wide private captures, full native ns-nova, stable ABI,
backend parity and self-hosting remain separate gates.
The later [lazy call-argument proof](nuis-native-lazy-call-argument-print-v1.md)
advances checked RHSs through guarded scalar captures without changing this
checkpoint's historical totals or source limits.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- computed_call_argument_print_epoch iteration_prints:: --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- computed_call_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- computed_call_argument_print_epoch --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete prior fields remain losslessly embedded in the tensor. Historical
validation chains remain quoted here-document data consumed by a no-op. Syntax
and inertness checks read that data through stdin without executing prior chains.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `d78d6bf2888fc564641179a890819dce64a23d3105c5894d67d7f35526b09e51` |
| `next_step` | `3f942495b8adfb97c9628da6adf4e6cbcf855c8550e504ef279af15663f73fa2` |
| `blocker` | `255a245b6e2034c4d489498397ae309182c63daf35d5d2d465f5294c388346e5` |
| `next_action` | `8d3211760a5507d26d2fde46485dd727a369c685cb36bae82393315470eeb2f0` |
| `validation_command` | `030186b52565fb474eb04271b2c921a7d3891e28155200a5da71ed708ba23c50` |
| `expected_artifact` | `5c8303bf25c04e1bb79247d004dddd59441725e9c7efa27973bf6b76578bf1b9` |

