# Native Logical I32 Call Argument Print V1

This bounded CPU worktree follow-up continues the
[i32 call argument proof](nuis-native-i32-call-argument-print-v1.md) on Git
beta-0.18.0 at `45a9f0c8` (2026-10-26). It introduces no ABI, source intrinsic,
runtime wrapper or backend-specific producer rule.

## Scoped Logical Parameters

The [shape policy](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_prefix.rs)
admits logical expressions only at exact bool parameter positions of completed
pure i32-return helpers inside an already-authorized lazy RHS. This is a scoped
argument-shape permission, not new read authority. Every primitive input still
requires its exact type and original epoch/nominal field path or independently
established ordinary readiness. Ordinary initializer, record and call-root
profiles do not acquire this permission.

The [typed argument plan](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs)
keeps new i32 call results comparison-only. They may pass through exact pure i32
parameters/results, but cannot be laundered through bool/float/i64 consumers,
arithmetic or casts. Missing, cyclic, transitively effectful and reference/record
signatures remain rejected. Arguments execute once in source order; each inner
AND/OR retains its own guard even when the outer helper is entered.

All arguments contribute to the complete 31-scalar helper-required capture union.
The whole-root 32 logical-edge budget and shared depth/work limits remain intact.
Rejected plans cannot recover field authority through an ordinary fallback.

Six [compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/logical_i32_call_argument_print_epoch_tests.rs)
cover normal/reversed declarations, root/ready-child/unready-child reads, exact
bool positions and primitive types, complete dependencies, nested i32 provenance,
reference/effect vetoes and ordinary-profile rejection. They pin 32/33 logical
edges, 4096/4097 work, depth and 31/32 capture boundaries. Source execution covers
both float widths, all six comparisons, signed extrema, nested AND/OR and actual
selector bool arguments, with selected/skipped argument and body failures.

## Execution And Delivery

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/logical_i32_call_argument_print_epoch.ns)
uses `checked32` and `select32(bool, i32, i32, i64)` without argument casts or i32
arithmetic. An owned code field switches from the left callback input to the
right at parent update and back at child update. Existing float State, ready
words, bool versions and outer sign wrappers remain observable.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/logical_i32_call_argument_print_epoch_cases.rs)
models each selector and checked helper structurally. Native probes print the
actual bool and sign-extended i32 arguments. Marker words 96/99/106/107/108 also
occur as payloads; helper counts are never inferred by counting marker values.
Ready and State float words stay exact. Only arithmetic NaNs normalize in the
test arithmetic observers.

[Native/reference controls](../../tools/nuisc/tests/native_application_bridge/logical_i32_call_argument_print_epoch.rs)
use 35 raw healthy sequences per float width, thirteen signed input pairs, six
finite CLI sequences and sixteen fault rows. Healthy inner AND/OR cases skip
zero-division poison while still entering the outer selector. A reached-fault
row explicitly uses equal i32 inputs to enter the inner OR RHS; successful
short-circuit cases are retained, not filtered out of an inferred fault matrix.
Normal/reversed YIR, actual argument streams, canaries, fault prefixes and last
accepted State remain checked. Success keeps 12 loop work, 512 helper entries
and at most 32 functions. Separate 128-entry NaN controls must reject atomically.

The [CLI matrix](../../tools/nuis/tests/native_session_workflow/logical_i32_call_argument_print_epoch.rs)
carries all thirteen signed pairs across six finite sequences and all sixteen
fault rows through fresh/cache delivery and two independent source-free restores.
Binary/container, host-object, lifecycle, envelope and packaging identities stay
pinned. Finite decimal delivery is not arbitrary NaN-payload evidence.

## Validation

654 distinct selected checks passed: 330 compiler, 66 affected native/reference,
one finite CLI delivery matrix, 178 LLVM (including ignored host O0/O2 execution),
three packaging, 17 buffer/control, three host failure/close, 26 Python, one
absolute-path policy, two file-limit and 27 tensor tests. Eleven controls are new:
six compiler, four native/reference and one CLI matrix. Earlier receipts retain
their historical totals and acceptance boundaries.

The rebuilt ordinary CLI reports 3,996/3,996 drift checks passing, clean coverage,
manifest, milestone, hierarchy and task lineage, and a ready selected active/99
task. All six source/runtime fields retain complete predecessor values and SHA-256
identities. Historical command syntax/inertness checks pass through stdin.
Formatting, whitespace, offline locked metadata and CLI integration compilation
pass. UTF-8 checks cover 5,900 files and documentation checks resolve 4,508 local
links. Entry routers retain their short-file limits; the proof index is 300 lines.

These are bounded CPU controls, not full-workspace acceptance, physical device
parity, a stable ABI, a full native engine or completed self-hosting.

## Remaining Frontier

The coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99. Next is
computed bool equality/inequality within this argument profile: both operands
must execute once left-to-right while each operand's internal AND/OR remains
guarded. Equality itself must not short-circuit the second operand. Original
reads, exact types, comparison-only i32 provenance and all budgets remain gates.
Ordinary profiles, i32 arithmetic/casts, resources/I/O/async capture, changing
backedges, wide private captures and full engine/backend parity stay separate.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- logical_i32_call_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- logical_i32_call_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- logical_i32_call_argument_print_epoch --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete preceding fields remain losslessly embedded in the tensor.
Historical command chains stay quoted here-document data consumed by a no-op;
syntax/inertness checks use stdin without replaying older command chains.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `007a69b1aa002d44e251f2bc4c2b9c21d4ade4c83bf35eb68ca126175844a458` |
| `next_step` | `48ed6af1bd2f6cc015906e1fe3f715f5c43556f2f23dcb30505d32cd2fa91fce` |
| `blocker` | `483a54da323eebc5eb8a81fd36176be101ac40a5d3cf4369d74351a98f9de9eb` |
| `next_action` | `863be1c5f613f363f140bfff193794cd15fcd3fc66271a9ff17acde185d891ab` |
| `validation_command` | `58fcc12abb8764a6f438e6f5cefca05708020206557b67ed9ad0ea304cf13e30` |
| `expected_artifact` | `73df37423acbf08b3802f96ff730d187fb501e868fbf8e2779e5e75242ef7aa3` |
