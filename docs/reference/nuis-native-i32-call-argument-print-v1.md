# Native I32 Call Argument Print V1

This bounded CPU worktree follow-up continues the
[float comparison proof](nuis-native-float-compare-argument-print-v1.md) on Git
beta-0.18.0 at `45a9f0c8` (2026-10-26). It introduces no ABI, source intrinsic,
runtime wrapper or backend-specific producer rule.

## Comparison-Only Results

The [typed argument plan](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs)
now admits completed exact pure i32-return calls only inside an already-authorized
lazy RHS. Their new permission is comparison-only: it may pass through exact
pure i32 call parameters/results but must be consumed by a same-kind comparison.
It cannot become generic bool/float/i64 argument readiness, arithmetic, a cast or
an ordinary initializer/call-root read. Existing independently authorized i32
field arguments retain their prior behavior.

Every primitive parameter still needs its exact type and original epoch/nominal
field path or independent ordinary readiness. Missing/cyclic/transitively effectful
helpers and record/reference signatures stay closed. Calls remain non-total,
source-ordered and once-only; all parameters contribute to the complete 31-scalar
helper-required RHS capture union. The ready-atom/predicate distinction, 32 logical
edges and shared depth/work limits remain unchanged. Rejected plans cannot
recover field authority through the ordinary fallback.

The [shape policy](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_prefix.rs)
still does not admit logical bool argument roots of these i32 helpers. Non-logical
primitive arguments and i32 forwarding chains are covered; logical parameters
need their own scoped shape/guard proof rather than a blanket result-kind change.

Six [compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i32_call_argument_print_epoch_tests.rs)
cover all six comparisons, normal/reversed function order, exact primitive input
positions, root/ready-child/unready-child reads and missing/effectful/cyclic/record
vetoes. They reject arbitrary result consumers, mixed widths, casts and argument
arithmetic, retain the closed logical-parameter boundary, and pin original read
order plus 31/32-capture/depth bounds. Source-level signed MIN/MAX/negative/zero
controls observe actual selector bool arguments and nested helper counts, including
selected/skipped left and transitive-body zero failures. Unit-source constant
construction outside this argument profile is not a new cast grant.

## Execution And Delivery

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/i32_call_argument_print_epoch.ns)
uses typed i32 callback inputs and checked `checked32`/transitive `relay32`
helpers in all six comparisons. It replaces an owned i32 field with the other
input at parent update and restores the first at child update, without argument
casts or i32 arithmetic. Float State, ready words, bool versions and outer float
sign wrappers remain independently observable.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/i32_call_argument_print_epoch_cases.rs)
compares sign-extended i32 values as signed i64 values, tracks every source helper
entry structurally and carries actual field versions through parent/child updates.
Native probes sign-extend actual i32 arguments before printing; marker values
106/107 are also payload data, never inferred entry counts. Ready/State float
words remain exact; only arithmetic NaNs normalize in arithmetic observers.

[Native/reference controls](../../tools/nuisc/tests/native_application_bridge/i32_call_argument_print_epoch.rs)
use 33 raw healthy sequences per float width, twelve explicit signed pairs,
six finite CLI sequences and sixteen fault rows. They pin exact signed input
words and typed values, argument failure before outer entry, transitive failure
prefixes, normal/reversed YIR, canaries and last accepted State. Success retains
12 loop work, 512 helper entries and at most 32 functions; separate 128-entry
NaN cases must reject atomically rather than raising producer budgets.

The [CLI matrix](../../tools/nuis/tests/native_session_workflow/i32_call_argument_print_epoch.rs)
includes all twelve signed pairs across its six finite sequences, before/after
all sixteen faults on fresh/cache and two independent source-free images.
Binary/container, host-object, lifecycle, envelope and packaging identities stay
pinned. Finite decimal delivery is not arbitrary NaN-payload evidence.

## Validation

644 distinct selected checks passed: 324 compiler, 62 affected native/reference,
one finite CLI delivery matrix, 178 LLVM (including ignored host O0/O2 execution),
three packaging, 17 buffer/control, three host failure/close, 26 Python, one
absolute-path policy, two file-limit and 27 tensor tests. Eleven controls are new:
six compiler, four native/reference and one CLI matrix. Prior receipts retain
their own acceptance totals and original boundaries.

The rebuilt ordinary CLI reports 3,982/3,982 drift checks passing, clean coverage,
manifest, milestone, hierarchy and task lineage, and a ready selected active/99
task. All six source/runtime fields retain complete predecessor values and SHA-256
identities. Historical command syntax/inertness checks pass through stdin.
Formatting, whitespace, offline locked metadata and CLI integration compilation
pass. UTF-8 checks cover 5,893 files and documentation checks resolve 4,497 local
links. These are selected CPU checks, not full-workspace acceptance, physical
device parity, stable ABI or completed self-hosting.

## Remaining Frontier

The coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99. Next is
logical bool parameters of completed pure i32 helpers inside authorized lazy RHS
comparison operands, preserving comparison-only provenance. The subsequent
[logical parameter proof](nuis-native-logical-i32-call-argument-print-v1.md)
covers that scoped follow-up without changing this receipt's historical totals.
I32 argument arithmetic
and casts, arbitrary consumers, composed bool equality, resources/I/O/async capture,
changing backedges, wide private captures, full native engine/backend parity,
stable ABI and compiler ownership transfer remain separate gates.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- i32_call_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- i32_call_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- i32_call_argument_print_epoch --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete preceding fields remain losslessly embedded in the tensor.
Historical command chains remain quoted here-document data consumed by a no-op;
syntax/inertness checks use stdin without replaying older command chains.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `4fb51dcfe333b4d38412403987d03e8fbb6ab4d9e48882038842ec9c55f9b0a6` |
| `next_step` | `01b51f58d7da98b20a780d96efb8794ff9ecd07cdfe2dd45fdf37b0ab08d486b` |
| `blocker` | `9b1775d9654110093366989781ab4d824c98a03fae2f72f8849ccaa5a03e7381` |
| `next_action` | `5bc56a9111515cea59102bf13e4711fa1299d5472c24c54e9481cf1a37f77083` |
| `validation_command` | `20cf0ace179133ca47189afae4b333a8dc8f7389f36e3c77ae9cdaf13f183b51` |
| `expected_artifact` | `1793a57d23b426a86d370547eedf8887c51a669a2ee453f7d4abafea7434d2c0` |

