# Native I32 Arithmetic Argument Print V1

Audit date: 2026-10-28. This bounded CPU worktree proof continues the
[bool XOR proof](nuis-native-bool-xor-argument-print-v1.md), using Git beta-0.18.1
at `c3320d9e` (2026-10-26) as the committed baseline. Earlier receipts retain
their original source baselines, scope and test totals. There is no new source
operator, intrinsic, runtime ABI, backend-specific producer case or wrapper.

## Canonical Wrapping, Not General Casts

The [front end](../../tools/nuisc/src/frontend/binary_lowering.rs) already emits
i32 Add/Sub/Mul as sign-extension of both operands to i64, a wide operation and
truncation to i32. The [argument proof](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs)
now recognizes only that complete shape, and only inside an already-authorized
lazy RHS. It validates both original operands as exact plain i32 values with
their existing epoch/nominal field authority or independent ordinary readiness.

All four wrapper/operation nodes consume work. Operand paths consume three depth
levels before their original value/projection is checked. Nested canonical
operations share the same budgets; they are not folded or treated as ready atoms.
Add/Sub/Mul over two signed i32 values cannot overflow i64, and the final wrapping
truncation is total. Calls in either operand may still fail and must execute once
left-to-right before an enclosing helper is entered.

The new arithmetic result is comparison-only even when its operands are total.
It may compose with canonical arithmetic and exact i32-return calls, but a
same-kind comparison must consume the permission. Bool/float/i64 consumers,
standalone sign-extension/truncation, malformed wrappers, mixed kinds, unwrapped
i32 arithmetic, Div/Rem/Xor and ordinary-profile escapes are not newly admitted.
The existing 31-scalar helper-required capture union, 32 logical edges and shared
depth/work bounds remain unchanged. No read/write authority is added by a cast.
Inline literal-narrowing constructors are not newly admitted either; independently
available typed scalar bindings keep their existing separate readiness path.

Six [compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i32_arithmetic_argument_print_epoch_tests.rs)
cover all three operations and six comparisons, both comparison operands,
normal/reversed declarations and root/ready-child/unready-child reads. They reject
missing/cyclic/effectful/record inputs, malformed conversion shapes, mixed kinds
and comparison-provenance laundering. Whole-union 31/32-capture controls and
original read order remain pinned. Balanced 817-operation trees plus three/four
i32 calls distinguish exactly 4096/4097 proof work; 32/33 logical roots and deep
wrapping chains retain rejection at their own boundaries.

Source execution crosses f32/f64, all operations/comparisons, outer AND/OR gates,
overflow/boundary pairs and first/second faults. Actual `checked32` and
`relay32` arguments expose the wrapped value, not just the final comparison.
Explicit constant construction outside this argument profile is not a new cast
grant inside it.

## Execution And Delivery

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/i32_arithmetic_argument_print_epoch.ns)
adds, subtracts and multiplies typed i32 callback inputs and versioned owned code
fields inside existing checked/transitive helpers. Parent and child updates switch
the original code field between left/right inputs. Float State, raw ready words,
bool versions and arithmetic observations remain separately observable.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/i32_arithmetic_argument_print_epoch_cases.rs)
uses signed i64 arithmetic, masks modulo 2^32 and restores the signed range. It
never interprets the compiler's conversion tree. Helper entries are counted
structurally, not from marker-valued payloads. Native probes sign-extend actual
i32 arguments, including wrapped values forwarded to transitive helpers.
Ready/State float words stay exact; only arithmetic NaNs normalize in test observers.

[Native/reference controls](../../tools/nuisc/tests/native_application_bridge/i32_arithmetic_argument_print_epoch.rs)
use 39 raw healthy sequences per float width, eighteen signed pairs, six finite
CLI sequences and sixteen explicit fault rows. Added pairs exercise MAX+1,
MIN-1, MIN times -1, 65536 products and nonzero high-bit multiplication overflow.
Normal/reversed YIR, exact arguments, counts, canaries, failure prefixes and last
accepted State stay checked. Success keeps 12 loop work, 512 helper entries and
at most 32 functions. Separate 128-entry NaN controls must reject atomically.

The [CLI matrix](../../tools/nuis/tests/native_session_workflow/i32_arithmetic_argument_print_epoch.rs)
covers every finite sequence and all eighteen signed pairs before/after every
fault row on fresh/cache delivery and two independent source-free restores.
Binary/container, host-object, lifecycle, envelope and packaging identities stay
pinned. Finite decimal transport is not arbitrary NaN-payload evidence.

## Validation

866 distinct selected checks passed: 530 compiler/front-end/higher-order/render,
78 affected native/reference, one finite CLI delivery matrix, 178 LLVM (including
ignored host O0/O2 execution), three packaging, 17 buffer/control, three host
failure/close, 26 Python, one absolute-path policy, two file-limit and 27 tensor
tests. Eleven controls are new: six compiler, four native/reference and one CLI
matrix. Earlier receipts are not summed or retroactively broadened.

The rebuilt ordinary CLI reports 4,044/4,044 drift checks passing, clean coverage,
manifest, milestone, hierarchy and task lineage, and a ready selected active/99
task. All six source/runtime fields retain complete predecessor values and SHA-256
identities. Historical command syntax/inertness checks pass through stdin.
Formatting, whitespace, offline locked metadata and CLI integration compilation
pass. UTF-8 checks cover 5,922 files and documentation checks resolve 4,543 local
links. The proof index remains within its 300-line entry-router budget.

These are bounded CPU checks, not full-workspace acceptance, stable ABI, physical
device/source-owned parser parity, a full native engine or completed self-hosting.

## Remaining Frontier

The subsequent [checked Div/Rem proof](nuis-native-i32-division-argument-print-v1.md)
owns that next gate and its separate evidence. The frontier below records this
receipt's original boundary, not the current selected task.

The coordinate stays
`standard-library/ns-nova/persistent-application-session`, active/99. Next is
checked i32 Div/Rem with exact representation and explicit zero/signed-overflow
semantics, including MIN/-1. A wide i64 quotient followed by truncation is not
proof of checked-i32 behavior and must not inherit the total arithmetic rule.
Arbitrary casts/consumers, references/resource/I/O/async captures, changing
backedges, wide private captures and full engine/backend parity remain separate.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- i32_arithmetic_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- i32_arithmetic_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- i32_arithmetic_argument_print_epoch --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete preceding fields remain losslessly embedded in the tensor.
Historical command chains remain quoted here-document data consumed by a no-op;
syntax/inertness checks use stdin without replaying older commands.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `121ec104f72ba8df58a16b2d1704947c40e3ea7fbe724c7c91356c9adf65ccf0` |
| `next_step` | `afe747396653b526bdca9f6681c6fe6aad34508392abbab5b9a791681f8b93f4` |
| `blocker` | `bbc5da0d34538eece5368e15f3b2d372011f4ee085b64397e59fa9c718293c30` |
| `next_action` | `691479009790df040d635198b01107a886317aecf2f146cfef3aca0913cf0b2f` |
| `validation_command` | `7a6a5937af242c4f70c59534bfad197241ad9a6672125da3d20195103cea074c` |
| `expected_artifact` | `f0da0fcbc51def1ccdd7e150b933f28de70ed26212088cd06291e8d32ff5409f` |
