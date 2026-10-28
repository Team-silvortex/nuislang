# Native Bool XOR Argument Print V1

This bounded CPU worktree proof continues the
[bool equality proof](nuis-native-bool-equality-argument-print-v1.md) on Git
beta-0.18.1 at `c3320d9e` (2026-10-26). Both remain worktree follow-ups to
that commit. Earlier receipts keep their own source baselines and test totals.
There is no new YIR instruction, runtime ABI, wrapper or source intrinsic.

## Source Syntax And Exact Types

The front end now accepts binary `^` for matching plain bool or i64 operands.
It associates left-to-right, binds less tightly than addition/multiplication and
more tightly than comparisons/equality, and is eager rather than short-circuiting.
For example, `a || b && c ^ d == e` groups as
`a || (b && ((c ^ d) == e))`. Parenthesize comparison operands explicitly.
Compound `^=` and doubled `^^` are not introduced.

The [parser](../../tools/nuisc/src/frontend/parser_exprs.rs), shared AST/NIR
inference, generic/lambda paths and renderers preserve the native XOR operation.
Debug NIR now renders XOR with the type-neutral caret instead of an integer-only
builtin name. The legacy `i64_xor(...)` entry keeps explicit exact-i64 checks;
adding bool inference must not let that integer-specific entry accept bools.
Mixed kinds, i32, floating-point, references, optionals and modified generic
primitive types are not accepted as XOR operands. No XOR trait overload is added.

Six [front-end controls](../../tools/nuisc/src/frontend/tests_frontend_core/bool_xor.rs)
pin precedence/association, bool truth tables, actual YIR `cpu.xor` and LLVM
`xor i1`, integer compatibility, strict legacy builtin input kinds, modified NIR
types, generic/lambda inference, UTF-8 token identity and tampered token rejection.
The existing stage-neutral symbol record carries caret as scalar 94; the protocol
is not replaced. This does not establish source-owned parser/candidate parity.

## Eager XOR, Guarded Operands

The [shape policy](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_prefix.rs)
shares an explicit typed-call-only eager-bool operand gate for Eq/Ne/Xor.
Ordinary prefixes and record profiles keep that gate off. XOR inherits the same
context on both sides; unlike AND/OR, it never grants its RHS lazy call/read
permission. Each operand's own internal AND/OR still has its independent guard.

The [typed argument plan](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs)
uses exact shared type inference for bool XOR while retaining original epoch/
nominal reads and completed pure call signatures. New i32 call results remain
comparison-only. XOR cannot launder them into arbitrary bool consumers, arithmetic,
casts or ambient readiness. Every operand contributes to the complete 31-scalar
helper-required capture union. Internal AND/OR edges share the existing 32-edge
budget, and work/depth bounds remain unchanged. XOR itself is not a new guard.
The conservative ready-predicate distinction is not widened.

Six [argument-proof controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/bool_xor_argument_print_epoch_tests.rs)
cover exact types, normal/reversed declarations, root/ready-child/unready-child
reads, ordinary/record vetoes and missing/effectful operands. A constant first
operand cannot suppress proof of the second. Read order, 31/32 captures, 32/33
logical edges, 4096/4097 work and depth rejection stay pinned. Source controls
cross both float widths, both inner logical operators, outer gates, signed
extrema and first/second faults, checking actual i32 and selector bool arguments.

## Execution And Delivery

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/bool_xor_argument_print_epoch.ns)
uses real caret operators inside a completed pure i32 selector's bool parameter,
then consumes its i32 result in a comparison. Owned code/bool fields change at
parent/child updates; float State, raw ready words and arithmetic observations
remain independent.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/bool_xor_argument_print_epoch_cases.rs)
evaluates each side separately before applying XOR and counts helper entries
structurally, not by scanning marker-valued payloads. Native probes expose actual
bool and sign-extended i32 arguments. Ready/State float words stay exact; only
arithmetic NaNs normalize in the test arithmetic observers.

[Native/reference controls](../../tools/nuisc/tests/native_application_bridge/bool_xor_argument_print_epoch.rs)
retain 35 raw healthy sequences per float width, thirteen signed input pairs,
six finite CLI sequences and sixteen explicit fault rows. They cover healthy
inner poison suppression, reached second-operand faults, normal/reversed YIR,
exact argument streams, helper counts, canaries, failure prefixes and last
accepted State. Success keeps 12 loop work, 512 helper entries and at most 32
functions. Separate 128-entry NaN controls must reject atomically.

The [CLI matrix](../../tools/nuis/tests/native_session_workflow/bool_xor_argument_print_epoch.rs)
checks fresh/cache delivery and two independent source-free restores, with every
finite sequence before and after all fault rows. Binary/container, host-object,
lifecycle, envelope and packaging identities remain pinned. Finite decimal
transport is not arbitrary NaN-payload evidence.

## Validation

856 distinct selected checks passed: 524 compiler/front-end/higher-order/render,
74 affected native/reference, one finite CLI delivery matrix, 178 LLVM (including
ignored host O0/O2 execution), three packaging, 17 buffer/control, three host
failure/close, 26 Python, one absolute-path policy, two file-limit and 27 tensor
tests. Seventeen controls are new: six front-end, six argument-proof, four
native/reference and one CLI matrix. Earlier receipts retain their own totals;
the wider front-end regression selection is not a sum of historical acceptance.

The rebuilt ordinary CLI reports 4,030/4,030 drift checks passing, clean coverage,
manifest, milestone, hierarchy and task lineage, and a ready selected active/99
task. All six source/runtime fields retain complete predecessor values and SHA-256
identities. Historical command syntax/inertness checks pass through stdin.
Formatting, whitespace, offline locked metadata and CLI integration compilation
pass. UTF-8 checks cover 5,915 files and documentation checks resolve 4,532 local
links. The proof index remains within its 300-line entry-router budget.

These are bounded CPU checks, not full-workspace acceptance, stable ABI, physical
device parity, a full native engine or completed self-hosting.

## Remaining Frontier

The coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99. Next is
same-kind i32 Add/Sub/Mul in comparison arguments, with verified canonical
sign-extension/operation/wrapping rather than arbitrary cast permission.
The subsequent [i32 arithmetic proof](nuis-native-i32-arithmetic-argument-print-v1.md)
covers that bounded follow-up without changing this receipt's historical totals.
Original reads, comparison-only call provenance, source order and existing capture/work/
entry limits remain gates. Resource/I/O/async capture, changing backedges, wide
captures, source-owned parser parity and full engine/backend parity stay separate.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- bool_xor --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- bool_xor_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- bool_xor_argument_print_epoch --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete preceding fields remain losslessly embedded in the tensor.
Historical command chains remain quoted here-document data consumed by a no-op;
syntax/inertness checks use stdin without replaying older commands.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `714474bb45d7ecc22b33d3d533d398065a1f817569b50285b47f10dd2c39aa74` |
| `next_step` | `fc5262e1df87528488a2af9ea0dc9e10364aebf963f4a0bc3570116f53040759` |
| `blocker` | `329066f6d5e702513c20dd7908c34d3f4c613225041a9b4d8370b7702105d5b0` |
| `next_action` | `2cd20d3d0c9c40492cbdcac299c144547b353f121a5bc4973813181ce94db40d` |
| `validation_command` | `8e784fe2f537b7bc13c252cab93545b4c5054a0c6715cf8d7ac3897881eb9e86` |
| `expected_artifact` | `b8971f62afedf1fa0dbf20379f7850acd21974d04f098e1c3d81aa8c3311e67f` |

