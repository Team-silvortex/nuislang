# Native I32 Division Argument Print V1

Audit date: 2026-10-28. This bounded CPU worktree proof continues the
[i32 arithmetic proof](nuis-native-i32-arithmetic-argument-print-v1.md), against
Git beta-0.18.1 at `c3320d9e` (2026-10-26). It does not create a new
source operator, intrinsic, ABI or backend-specific producer case.

## Checked Width, Not Wrapping Quotients

The [front end](../../tools/nuisc/src/frontend/binary_lowering.rs) already emits
exact i32 Div/Rem without the Add/Sub/Mul extension/truncation wrapper. The
[shared value type fact](../../tools/nuisc/src/lowering/buffer_loop_outline/control_values.rs)
now recognizes these exact-kind operations, including completed pure helper
bodies. Type recognition does not itself grant ordinary argument/read authority.

The [argument proof](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs)
admits Div/Rem only inside an already-authorized lazy RHS, with both original
operands proved as exact plain i32. Results remain comparison-only: exact i32
calls and canonical wrapping arithmetic may compose, but a same-kind comparison
must consume the permission. Arbitrary casts, mixed operands, non-i32 consumers,
effects/cycles/records, unready child reads and ordinary-profile escapes stay
rejected. A wide i64 quotient followed by truncation is not checked i32 semantics.

Division truncates toward zero; remainder has the dividend's sign. Both operations
fail for zero divisors and for MIN / -1 (also MIN % -1). Reached operations remain
fallible even when their result is unused. Short-circuit suppression is required,
not eager speculation. Operands execute once left-to-right before the enclosing
callee entry. No operand or result becomes an ordinary ready atom.

A reference-executor mismatch was repaired: generic i32 Div/Rem previously used
an integer accessor that widened i32 to i64. The
[reference executor](../../crates/yir-domain-cpu/src/execute_scalar.rs) now retains
i32 result width, rejects mixed kinds and checks both exceptional cases without
panicking or modifying state. Typed div_i32 shares the same implementation.
The [LLVM backend](../../crates/yir-lower-llvm/src/scalar_lowering.rs) already
emits checked sdiv/srem i32. Its
[strict native function validator](../../crates/yir-lower-llvm/src/function_lowering/strict_scalar.rs)
also needed its generic division/remainder rule corrected to admit matching i32
operands without coercing mixed widths. No new lowering opcode or ABI is added.

Six [compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i32_division_argument_print_epoch_tests.rs)
pin all six comparisons, both operand positions, reversed declarations, exact
original root/child read permission, provenance and forged-wide rejection.
31-scalar capture unions, 32 logical edges, depth 64 and work 4096 remain shared
bounds. Balanced 2044-operation trees distinguish exactly 4096/4097 proof work.
Source execution covers both float widths, signed results, selected/skipped
zero/overflow, first/second helper faults and unused reached helper operations.
Actual helper arguments expose the quotient/remainder, not just final booleans.

Three [reference controls](../../crates/yir-domain-cpu/src/tests_scalar_logic.rs)
check exact result kinds, signed results, mixed-kind rejection and failure
without panic or state mutation. Three LLVM controls, including a strict
matching/mixed-kind validation matrix and two [instruction controls](../../crates/yir-lower-llvm/src/tests/i32_division_tests.rs) pin
32-bit instructions, zero/MIN/-1 guards before arithmetic and unique guard labels.

## Execution And Delivery

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/i32_division_argument_print_epoch.ns)
uses typed callback inputs and versioned owned code fields: a parent lazy RHS
divides before relay32 entry, and a child lazy RHS computes remainder before
another relay32 entry. Original root/child updates, float State and raw input
words remain independently observable.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/i32_division_argument_print_epoch_cases.rs)
checks zero and MIN/-1 explicitly, then uses signed i64 arithmetic and a checked
conversion, never truncating an overflowing quotient. Native probes sign-extend
actual i32 callee inputs. Structural counts distinguish real helper entries from
marker-valued payloads. Float State/ready words stay exact; only arithmetic NaNs
normalize in test observers.

The [native/reference matrix](../../tools/nuisc/tests/native_application_bridge/i32_division_argument_print_epoch.rs)
uses 33 raw healthy sequences per float width, twelve signed pairs, six finite
CLI sequences and twenty explicit fault rows, including all four reached
zero/overflow Div/Rem combinations. Healthy rows also skip poisoned divisors.
Normal/reversed YIR, actual argument words, counts, failure prefixes, output
canaries and last accepted State are checked. Success retains 12 loop work,
512 helper entries and at most 32 functions; separate 128-entry NaN controls
must reject atomically.

The [CLI matrix](../../tools/nuis/tests/native_session_workflow/i32_division_argument_print_epoch.rs)
checks fresh/cache delivery and two independent source-free restores, healthy
execution before/after faults and all finite signed pairs. Binary/container,
host-object, lifecycle, envelope and packaging identities remain pinned.
Finite decimal transport does not establish arbitrary NaN-payload CLI parity.

## Validation

989 distinct selected checks passed: 536 compiler/front-end/higher-order/render,
82 native/reference, one finite CLI delivery matrix, 181 LLVM including ignored
host O0/O2 execution, 110 CPU-domain, three packaging, 17 buffer/control, three
host failure/close, 26 Python, one absolute-path policy, two file-limit and 27
tensor tests. Seventeen controls are new: six compiler, three reference, three
LLVM/native-kind, four native/reference and one CLI matrix. Earlier receipts
retain their own test totals and are not summed or retroactively broadened.

The rebuilt ordinary CLI reports 4,059/4,059 drift checks passing, clean coverage,
manifest, milestone, hierarchy and task lineage, with a ready selected active/99
task. All six source/runtime fields retain complete predecessor values and
SHA-256 identities. Historical shell syntax/inertness checks pass through stdin.
UTF-8 checks cover 5,930 files and 4,560 local documentation links resolve. The
proof index remains within its 300-line entry-router budget. Formatting,
whitespace, offline locked metadata and CLI integration compilation pass.

These are bounded CPU checks, not full-workspace acceptance, stable ABI, physical
device/source-owned parser parity, a complete native engine or self-hosting.

## Remaining Frontier

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. Next is a separately budgeted proof for exact inline i32 literal
construction in these comparison arguments, without admitting arbitrary casts
or relying on out-of-range truncation. Mixed types, references/resource/I/O/async
captures, changing backedges, wide private captures, full engine/backend parity
and actual compiler self-hosting remain separate gates.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- i32_division_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-domain-cpu --lib --locked -j1 -- checked_i32_division --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-lower-llvm --lib --locked -j1 -- checked_i32_division --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- i32_division_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- i32_division_argument_print_epoch --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete preceding fields remain losslessly embedded in the tensor,
with SHA-256 identities. Historical command chains remain quoted here-document
data consumed by a no-op; syntax/inertness checks use stdin without replaying
older commands.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `bb430e64887b04cfec0dd3140e2370396f22faf1381b2a5a67355c0e1a4e06ed` |
| `next_step` | `61d46de1dc3bdbfa16f57045a57fb1c5378899179d8eaf14f0798e671c0ac4c1` |
| `blocker` | `16eddb67e78cd1544462e6127eda87eb4d8f1ef25bb483557ae52d94725958a1` |
| `next_action` | `94b502410d3f77a5f4641efea16834b03421c9dcbd2a66af0bc9fd7b027535e0` |
| `validation_command` | `e577cb9c5a754086a8f807c5346945fee78841c2be8031338a3455e3c3036bda` |
| `expected_artifact` | `b70a1e6ea64dc0ede9c358d9f69dcc8ecc01cd60fd5ec5251af93035c47e52f5` |
