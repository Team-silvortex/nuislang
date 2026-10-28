# Native I64 Arithmetic Argument Print V1

Audit date: 2026-10-28. This bounded CPU worktree proof follows
[explicit signed widening](nuis-native-i32-widening-argument-print-v1.md).
Git remains beta-0.18.2 at `5bbd4de8` (2026-10-28). This is not a version bump;
earlier receipts retain their own baselines, permission boundaries and totals.

## Exact Arithmetic Composition

The [argument proof](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs)
now admits Add/Sub/Mul/Div/Rem over two exact plain i64 operands when an
already-authorized lazy RHS contains a comparison-restricted input. Signed
i32 widening remains explicit. Arithmetic preserves comparison-only provenance
rather than consuming it; a same-kind comparison must eventually do that.
Same-kind pure helpers may forward the result under the preceding contract.

This is a proof/admission change, not a new opcode, backend, runtime ABI or
engine-specific producer. The existing [CPU executor](../../crates/yir-domain-cpu/src/execute_scalar.rs)
uses wrapping i64 Add/Sub/Mul. The existing
[LLVM lowering](../../crates/yir-lower-llvm/src/scalar_lowering.rs) emits unflagged
i64 operations and checks zero and MIN/-1 before signed Div/Rem. Remainder
overflow is an error too. Neither extension nor wrapping arithmetic hides a
previous i32 checked fault. Div/Rem remain fallible in the proof.

Original field/epoch authority and independent child readiness are unchanged.
Mixed i32/i64, bool/float/reference/record operands, effectful or cyclic helpers,
runtime narrowing and ordinary-read escapes stay rejected. Restricted values
cannot become bool/float/i32 helper results. XOR is not inferred from the five
arithmetic operations. Arithmetic clears ordinary readiness and cannot grant it.

## Work And Source Controls

Six [profile controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i64_arithmetic_argument_print_epoch_tests.rs)
cross all five operations with all six comparisons, both operand positions,
normal/reversed declarations and root/ready-child/unready-child reads. They
check chained arithmetic/call provenance, consumers and invalid operands.
The predecessor's arithmetic veto now rejects mixed kinds instead of newly
admitted same-kind operations; its XOR veto and other rejection controls remain.

Every operation, extension and leaf shares the existing work/depth accounting.
A balanced 2043-operation i64 tree with one explicit extension and one/two calls
separates exactly 4096/4097 work. Other controls retain depth 64/65, capture union
31/32, logical roots 32/33 and original leaf visitation. No budget is increased.

Source execution covers both float widths, arithmetic wrapping, exact i64
helper argument words, comparison choices, actual prints, AND/OR suppression,
two helper fault positions and reached/skipped zero/MIN/-1 division/remainder.
Its independent oracle uses mathematical i128 modulo 2^64, not the backend's
wrapping operation chain. These focused cases supplement rather than replace
the six-comparison structural matrix and adjacent compiler regressions.

## Native And Source-Free Delivery

The [Nuis fixture](../../tools/nuisc/tests/native_application_bridge/i64_arithmetic_argument_print_epoch.ns)
performs checked i32 quotient/negation, explicit signed widening, i64 addition,
subtraction, multiplication and checked division directly in the parent
argument. The child performs widened arithmetic followed by checked remainder.
The existing unconverted i32 path and negative literal forwarding remain as
adjacent controls. These operations are no longer hidden inside a pure helper.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/i64_arithmetic_argument_print_epoch_cases.rs)
computes the full-width polynomial in i128 and restores signed modulo 2^64.
Explicit checks distinguish zero and MIN/-1 faults before i128 division.
The [native/reference matrix](../../tools/nuisc/tests/native_application_bridge/i64_arithmetic_argument_print_epoch.rs)
retains 33 raw healthy sequences per float width, twelve signed input pairs,
six finite CLI sequences and the twenty preceding fault rows. Two more rows
first complete valid i32 operations, then fail specifically at i64 MIN/-1
Div/Rem: there are 22 fault rows per width.

Actual negative i64 arguments, values outside the i32 range, all four helper
entry counts, raw float State/ready words, parent/child versions and normal/
reversed YIR remain observable. Flushed native failure prefixes, output canaries,
last accepted State, faulted close and unchanged publication are checked.
Only arithmetic NaNs normalize in test observers. Limits remain 12 loop work,
512 helper entries and at most 32 functions; separate 128-entry NaN exhaustion
must reject atomically.

The [CLI matrix](../../tools/nuis/tests/native_session_workflow/i64_arithmetic_argument_print_epoch.rs)
checks fresh/cache delivery and two independent source-free restores for both
float widths, healthy runs before/after all fault rows and unchanged binary,
container, host-object, lifecycle, envelope and packaging identities. Source,
manifest and project cache are removed before restoration. Finite decimal
transport does not establish arbitrary NaN-payload CLI parity.

## Validation

1,039 distinct selected checks passed: 569 compiler/front-end/higher-order/render,
98 native/reference, one finite CLI delivery matrix, 181 LLVM including ignored
host O0/O2 execution, 110 CPU-domain, three packaging, 17 buffer/control, three
host failure/close, 26 Python, one absolute-path policy, two file-limit, 27 tensor
and one adjacent effectful-if integration. Eleven controls are new: six profile,
four native/reference and one CLI matrix. The selected CLI matrix replaces its
predecessor; historical receipts are not added together.

The ordinary CLI reports 4,115/4,115 drift checks passing, clean coverage,
manifest, milestone, hierarchy and task lineage, with a ready selected active/99
task. All six source/runtime fields retain complete predecessor values and
SHA-256 identities. Historical shell syntax/inertness checks pass through stdin.
UTF-8 checks cover 5,960 files and 4,613 local documentation links resolve. The
proof index remains within its 300-line entry-router budget. Formatting,
whitespace, offline locked metadata and CLI integration compilation pass.

These are bounded CPU checks, not full-workspace acceptance, stable ABI,
physical GPU/provider parity, a complete native engine or compiler self-hosting.

## Remaining Frontier

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. Next is comparison-scoped i64 XOR composition: prove eager operand
ordering, exact kinds and inherited comparison-only provenance before admission.
It is still rejected here. Broader casts, ordinary reads, resource/I/O/async
capture, changing backedges, wide private captures and full backend/engine parity
retain separate gates.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- i64_arithmetic_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- i64_arithmetic_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- i64_arithmetic_argument_print_epoch --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete preceding tensor fields remain embedded losslessly with their
SHA-256 identities. Historical commands are quoted here-document data consumed
by a no-op; shell syntax/inertness checks use stdin without replaying history.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `4a31e40c966932ce1574f79839cc6871d849b3f0f2fa7410365432dde08970e1` |
| `next_step` | `63239ee9c0719faef968764304ecc8271b899408c0e71bdaedc5f6cd10dcd961` |
| `blocker` | `079442a3fc5468300be6c065f765c9ab507bad2b9c787a7a2ff061af36854450` |
| `next_action` | `ebd0231223fa22a95a8b5a235d89081626c0b7392449fbc2ce22d7b3cf5b48d4` |
| `validation_command` | `60dc217ffceeba09af2663174ac9df49e11b77b89453a953741df633f19c1dce` |
| `expected_artifact` | `372ad789a381e2fc2b07460cf6d5f954af04a3677d612a6ed4d61f15fe28e5e5` |
