# Native Float Compare Argument Print V1

This bounded CPU worktree proof follows
[scoped logical auxiliary arguments](nuis-native-logical-helper-argument-print-v1.md)
on Git beta-0.18.0 at `45a9f0c8` (2026-10-26). It does not change source
intrinsics, ABI, runtime wrappers or backend-specific producer rules.

## Exact Comparison Operands

The [shared typed plan](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs)
now reuses the existing exact binary-type contract for all six same-kind f32/f64
comparisons. The result must be bool at its existing exact parameter position.
Mixed widths, casts, missing/effectful/cyclic helpers and record/reference operands
remain rejected. Each field read still needs its original epoch/nominal path
certificate or independently ready scalar permission; ordinary initializers,
conditions, call roots and whole-record reads gain no new authority.

Both operands retain left-before-right proof/evaluation, non-total helper calls,
private sign-wrapper support and cumulative depth/work limits. Logical bool
arguments retain their 32-edge budget and scoped auxiliary-call permission.

The proof also distinguishes non-trapping values from helper-free ready
predicates. A field or arithmetic comparison can be total yet still need a
private RHS helper: its complete 31-scalar capture union must be checked. Wider
comparisons over already-ready atoms remain accepted when the shared guarded
lowering needs no helper. No failed scalar projection can become a new whole-record
grant through the sealed fallback.

Six [compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/float_compare_argument_print_epoch_tests.rs)
cover six exact bool results, root/ready-child/unready-child reads, malformed type,
width/cast/effect/missing vetoes, original operand order and 31/32 capture bounds.
They separately pin total-but-helper-required versus genuinely helper-free trees,
raw IEEE selector bool arguments, once-only operand calls and selected/skipped
left/right operand faults. NaN output classification alone is not comparison proof.

## Execution And Delivery

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/float_compare_argument_print_epoch.ns)
places equality at parent entry, ordered comparisons in the child and inequality
after a child field update. Checked `sample` operands compose with bool/i64
auxiliary arguments, integer remainder poison, outer float sign operations and
independent State publication. Expressions are split across these read epochs
rather than increasing the existing 32-function fixture bound.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/float_compare_argument_print_epoch_cases.rs)
uses unsigned IEEE exponent/fraction classification and sign-adjusted bit ordering,
not the compiler's comparison implementation. It handles unordered inequality,
equal signed zeros, infinities, subnormals and adjacent finite values. A separate
truth-table control checks it against host comparisons on explicit word pairs.

Actual native sample probes retain raw operands, including signaling/quiet NaN
payloads, before checked bodies; selector and bool-helper probes expose comparison
results. Arithmetic NaNs normalize only in arithmetic observers. Ready/State and
sample-input words remain exact. Helper counts are structural and use the actual
previous State: a later event's floating predicate must not be predicted from its
ignored replacement seed. No marker-valued data is counted as an entry.

[Native/reference tests](../../tools/nuisc/tests/native_application_bridge/float_compare_argument_print_epoch.rs)
use 42 raw healthy sequences per width, including eighteen explicit comparison
pairs, twenty explicit fault rows and normal/reversed YIR. Two earlier overflow
rows are retargeted to reached parent/child helper bodies after the expression
layout change; they are not silently filtered by execution results. Success uses
12 loop work, 512 helper entries and at most 32 functions. Separate 128-entry NaN
cases must reject atomically while retaining last accepted State and exact reached
native prefixes.

The [finite CLI matrix](../../tools/nuis/tests/native_session_workflow/float_compare_argument_print_epoch.rs)
uses six healthy sequences per width before/after all twenty faults on fresh,
cache and two independent source-free images. Binary/container, host-object,
lifecycle, envelope and packaging identities remain pinned. Decimal transport
is not arbitrary NaN-payload evidence.

## Validation

634 distinct selected checks passed: 318 compiler, 58 native/reference, one finite
CLI delivery matrix, 178 LLVM (including ignored host O0/O2 execution), three
packaging, 17 buffer/control, three host failure/close, 26 Python, one absolute-path
policy, two file-limit and 27 tensor tests. Eleven controls are new: six compiler,
four native/reference and one CLI matrix. The focused native set follows the
typed-print/field paths affected by this change; earlier broader origin/storage
totals are not added to this run.

The rebuilt ordinary CLI reports 3,968/3,968 drift checks passing, clean coverage,
manifest, milestone, hierarchy and task lineage, and a ready selected active/99
task. All six source/runtime fields retain complete predecessor values and SHA-256
identities. Historical command syntax/inertness checks pass through stdin.
Formatting, whitespace, offline locked metadata and CLI integration compilation
pass. UTF-8 checks cover 5,886 files and documentation checks resolve 4,486 local
links. This is selected CPU evidence, not full-workspace acceptance, physical
device parity, stable ABI or completed self-hosting.

## Remaining Frontier

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99. Next is
exact pure i32-return helper operands inside authorized lazy RHS comparisons,
not general i32 arithmetic or casts. The later
[comparison-only i32 follow-up](nuis-native-i32-call-argument-print-v1.md) advances
that bounded gate while these original results remain historical.
Composed bool equality, resource/I/O/async
capture, changing backedges, wide private helpers, full native engine/backend
parity, stable ABI and compiler ownership transfer retain separate gates.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- float_compare_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- float_compare_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- float_compare_argument_print_epoch --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete preceding fields remain losslessly embedded in the tensor.
Historical command chains remain quoted here-document data consumed by a no-op;
syntax/inertness checks use stdin without replaying earlier command chains.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `7289a14375f22a454785aa6faee7d5ac6254425c3ca6c6a22a0f9f97e718fdc2` |
| `next_step` | `f5da8c549017f802e7b70d95611c6243d4d177d298de25244f4eb1e780ab9a68` |
| `blocker` | `d8bc9682ca09fc921a690d91d241492b214e505edd2d4552ae2d678ac67d5e4e` |
| `next_action` | `f755eeacf48720c485292a0eb9812862e77337bde032e3fba3338c9ee655869f` |
| `validation_command` | `46732e033528c86fda9317b8295a3583aa35807c1515751c16e24672ff641eb6` |
| `expected_artifact` | `0c19cc1abd3b8b36c18bc4e5a3863bfd05729d7dbeb0e7f9f88d2e99a02e49b6` |

