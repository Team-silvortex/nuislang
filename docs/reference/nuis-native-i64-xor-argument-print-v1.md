# Native I64 XOR Argument Print V1

Audit date: 2026-10-28. This bounded CPU worktree proof follows
[restricted i64 arithmetic](nuis-native-i64-arithmetic-argument-print-v1.md).
The committed baseline is Git beta-0.18.3 at `2c5b4796` (2026-10-28).
Earlier receipts retain their historical baselines, scope and executed totals.
This follow-up does not create another release.

## Exact Eager Composition

The [argument proof](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs)
now admits XOR over two exact plain i64 operands in an already-authorized lazy
comparison context. An explicitly widened or arithmetic-derived restricted
operand keeps the result comparison-only. Same-kind pure helpers may forward
that result; a comparison must ultimately consume the restriction.

XOR does not create a lazy context for either operand. Both operands evaluate
once in source order, even when one is zero. The enclosing AND/OR may skip the
whole expression, but XOR itself is not a short-circuit operator. Operand
fallibility and original field/epoch capture authority remain part of the proof.
The result grants neither ordinary value readiness nor predicate readiness.

The existing [CPU executor](../../crates/yir-domain-cpu/src/execute_scalar.rs)
and [LLVM bitwise lowering](../../crates/yir-lower-llvm/src/bitwise_lowering.rs)
already support full-width i64 XOR; the caret syntax and typed source inference
already exist. This change introduces no opcode, ABI, backend-specific producer
or implicit i32 widening. Bool XOR retains its earlier separate admission.
Mixed i32/i64/bool/float/reference/record operands, runtime narrowing, effectful
or cyclic helpers and cross-kind consumers remain rejected.

## Source And Budget Controls

Six [profile controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i64_xor_argument_print_epoch_tests.rs)
cover six comparisons, both operand positions, normal/reversed declarations,
root/ready-child/unready-child reads, arithmetic/helper composition and provenance
escapes. Earlier XOR vetoes now reject mixed kinds instead of the newly admitted
exact operation; their other boundaries and test identities remain intact.

Every XOR, conversion and leaf shares existing accounting. A balanced
2043-operation tree, one extension and one/two helper calls distinguish exactly
4096/4097 work. Separate controls retain depth 64/65, capture union 31/32 and
logical roots 32/33. Leaf visitation is checked in both operand orders.

Source execution covers both float widths, every comparison, both XOR orders,
MIN/MAX, negative and high-bit masks, arithmetic-derived operands, actual i64
callee words, predicate choices and prints. Independent left/right guards,
including both poisoned and zero-valued XOR operands, distinguish eager XOR
from the enclosing AND/OR suppression rule.

Reference scalar call events are completion records: a nested checked64 event
precedes its containing relay64 event. The [execution engine](../../crates/yir-exec/src/execution_engine.rs)
emits the event only after executing and validating the callee. Source controls
assert that completion order. They do not mislabel it as callee entry order;
the native probes below separately observe entries and flushed fault prefixes.

## Native Execution And Delivery

The [Nuis fixture](../../tools/nuisc/tests/native_application_bridge/i64_xor_argument_print_epoch.ns)
combines widened checked arithmetic with a fallible full-width mask helper.
The parent evaluates arithmetic before its mask call; the child reverses those
operand roles. Negative-literal XOR remains a third composition path. An
independent scalar mask guard is a normal fixture parameter, not a runtime ABI
change or new protocol slot. Entry probes record its actual transported value.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/i64_xor_argument_print_epoch_cases.rs)
reconstructs all 64 XOR bits using integer quotient/remainder, without using the
backend's XOR primitive. Existing signed modulo arithmetic and checked i32/i64
fault oracles remain separate. Full-width helper probes record values without
truncation, and four structural counters distinguish real calls from marker data.

The [native/reference matrix](../../tools/nuisc/tests/native_application_bridge/i64_xor_argument_print_epoch.rs)
retains 33 raw healthy sequences per float width, twelve signed pairs, six
finite CLI sequences and the 22 preceding fault rows. Two additional mask-guard
faults exercise the right XOR operand in the parent and the left in the child:
there are 24 fault rows per width. A skipped mask fault and zero-trip poison
remain healthy. A native zero-valued left XOR operand still reaches the right
mask call and its fault; the matching non-poisoned row stays healthy.
Source ordering, raw float State/ready words, original field
versions, last accepted State, canaries and faulted close remain observable.

Budgets stay 12 loop work, 512 helper entries and at most 32 functions. Separate
128-entry NaN exhaustion must reject atomically. Only arithmetic NaNs normalize
in test observers; raw State/ready words retain their complete payloads.

The [CLI matrix](../../tools/nuis/tests/native_session_workflow/i64_xor_argument_print_epoch.rs)
checks fresh/cache artifacts and two independent source-free restores for each
float width. Healthy runs before/after fault rows, exact outputs, binary/container,
host-object, lifecycle, envelope and packaging identities are retained. Source,
manifest and project cache are removed before restoration. Finite decimal CLI
transport is not arbitrary NaN-payload transport parity.

## Validation

1,049 distinct selected checks passed: 575 compiler/front-end/higher-order/render,
102 native/reference, one finite CLI delivery matrix, 181 LLVM including ignored
host O0/O2 execution, 110 CPU-domain, three packaging, 17 buffer/control, three
host failure/close, 26 Python, one absolute-path policy, two file-limit, 27 tensor
and one adjacent effectful-if integration. Eleven controls are new: six profile,
four native/reference and one CLI matrix. The selected CLI matrix replaces its
predecessor; historical receipts are not added together.

The ordinary CLI reports 4,131/4,131 drift checks passing, clean coverage,
manifest, milestone, hierarchy and task lineage, with a ready selected active/99
task. All six source/runtime fields retain complete predecessor values and
SHA-256 identities. Historical shell syntax/inertness checks pass through stdin.
UTF-8 checks cover 5,967 files and 4,626 local documentation links resolve. The
proof index remains within its 300-line entry-router budget. Formatting,
whitespace, offline locked metadata and CLI integration compilation pass.
The dated snapshot follows the committed beta-0.18.3 identity without rewriting
any earlier receipt's baseline or executed totals.

These are bounded CPU checks, not full-workspace acceptance, stable ABI,
physical GPU/provider parity, a complete native engine or compiler self-hosting.

## Remaining Frontier

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. Next is an audit of source-local scalar temporaries containing these
restricted arithmetic/XOR expressions. Establish initializer, original-epoch
and provenance behavior before any new admission; this receipt does not claim
that every local-binding combination is absent or supported. Ordinary readiness,
runtime narrowing, resource/I/O/async capture, changing backedges, wide private
captures and full backend/engine parity retain separate gates.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- i64_xor_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- i64_xor_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- i64_xor_argument_print_epoch --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete preceding tensor fields remain embedded losslessly with their
SHA-256 identities. Historical commands are quoted here-document data consumed
by a no-op; syntax/inertness checks use stdin without replaying historical work.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `4a9ff278ae103d1b0b32e052b159eddbb0aa84bbd77f5b959887863caeed3c7d` |
| `next_step` | `42284d70cd0cf39f1066f726fef1f189172443185bd7fa30fc4cbc33ece99fd7` |
| `blocker` | `6dc30acea2b7744c763585cc9eb2f6f8fa18c1be784d343f55a26610c7ac11fa` |
| `next_action` | `7e96b07459ae1dc160d379c9eb044e03912c32dfc96081b19874c3a9869b51b7` |
| `validation_command` | `ad1a6d94ee883b9ce0dfc8c3870d8c2b2e8839cc0ea9a922df2800afbc20d7bc` |
| `expected_artifact` | `915ebf9f824ed20c665965d345567dc87c45a41ed535cd5ca2237e88dcf0537e` |
