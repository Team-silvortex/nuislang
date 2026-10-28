# Native I32 Literal Argument Print V1

Audit date: 2026-10-28. This bounded CPU worktree proof continues the
[checked i32 Div/Rem proof](nuis-native-i32-division-argument-print-v1.md), against
Git beta-0.18.2 at `5bbd4de8` (2026-10-28). Earlier receipts retain
their own baselines and executed totals. There is no new source syntax, intrinsic,
runtime ABI or engine-specific/backend-specific producer case.

## Exact Literals, Not General Casts

The [front end](../../tools/nuisc/src/frontend/call_routing.rs) represents
`i32_from_i64(...)` as a cast. A plain integer literal is one child node;
a source negative integer uses the existing
[unary representation](../../tools/nuisc/src/frontend/unary_lowering.rs), an exact
zero-minus-nonnegative-magnitude tree. The front end is not changed or globally
constant-folded for this proof.

The [argument proof](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs)
recognizes only these complete literal forms, only inside an already-authorized
lazy RHS, and only when the signed value lies within MIN..MAX for i32. A negative
form requires an exact zero left operand and a nonnegative literal magnitude;
checked negation plus checked i32 conversion reject malformed/overflowing values.
Runtime expressions, arbitrary arithmetic, nested negations, general casts,
out-of-range wrapping constructors, non-i32 consumers and ordinary-profile
escapes are not newly admitted. This scoped restriction does not change the
existing language-wide cast semantics outside this argument profile.

The cast and every child are traversed by the existing bounded proof. Plain
literals consume two nodes; canonical negative literals consume four. They
remain total but not ready atoms/predicates. Their exact i32 result stays
comparison-only through exact i32 calls, canonical wrapping Add/Sub/Mul and
checked Div/Rem; a same-kind comparison must consume that permission.
Literal nodes capture no variables and grant no original-field read authority.
All pre-existing 31-scalar capture unions, 32 logical edges, depth/work limits,
helper purity/type checks and source-order requirements remain in force.

Six [compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i32_literal_argument_print_epoch_tests.rs)
cover bounds, MIN/MAX/-1/0/1 and marker-valued literals, both comparison operands,
all six comparisons, reversed declarations and root/ready-child/unready-child
permission. Both raw NIR and real source reject out-of-range constructors, even
when their branch would be skipped. Nonliteral/malformed conversions, mixed
kinds, effectful/cyclic calls and provenance escapes remain rejected.

Balanced 1362-operation plain-literal trees plus one/two calls distinguish
4096/4097 proof work. A second 1361-operation tree replaces one leaf with a
canonical negative literal and uses two/three calls to pin the same boundary.
Separate depth controls account for the extra negative-subtraction levels;
31/32-capture and 32/33-logical-root controls retain their rejection points.
Actual leaf visitation preserves left-before-right checking.

Source execution crosses f32/f64, five arithmetic operations, six comparisons,
AND/OR gates, signed boundaries and first/second helper faults. Reached zero and
MIN/-1 Div/Rem failures remain failures; skipped well-typed poison is suppressed.
Actual checked32/relay32 arguments expose the computed integer and exact literal
inputs, not only the final boolean result.

## Execution And Delivery

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/i32_literal_argument_print_epoch.ns)
uses literals in parent and child lazy expressions: a checked quotient plus one,
a MAX comparison, a real helper input of 107, and a checked remainder minus
negative one compared with MIN. Parent/child code-field versions, float State,
raw input words and checked/transitive helper entry probes remain observable.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/i32_literal_argument_print_epoch_cases.rs)
uses signed wide arithmetic with explicit modulo-2^32 restoration, independent
of the compiler's conversion tree. Checked division/remainder keep explicit zero
and MIN/-1 checks. Structural counts distinguish actual helper entries from
marker-valued payloads. Native probes sign-extend real i32 arguments, including
the literal 107. Float State and ready words stay exact; only arithmetic NaNs
normalize in test observers.

The [native/reference matrix](../../tools/nuisc/tests/native_application_bridge/i32_literal_argument_print_epoch.rs)
retains 33 raw healthy sequences per float width, twelve signed input pairs,
six finite CLI sequences and twenty explicit fault rows. It checks normal and
reversed YIR, exact input words, actual helper entries, flushed fault prefixes,
output canaries and the last accepted State. Success keeps 12 loop work,
512 helper entries and at most 32 functions. Separate 128-entry NaN controls
must reject atomically.

The [CLI matrix](../../tools/nuis/tests/native_session_workflow/i32_literal_argument_print_epoch.rs)
checks fresh/cache delivery plus two independent source-free restores, healthy
execution before/after faults, and unchanged binary/container, host-object,
lifecycle, envelope and packaging identities. Finite decimal CLI input does not
prove arbitrary NaN-payload transport.

## Validation

999 distinct selected checks passed: 542 compiler/front-end/higher-order/render,
86 native/reference, one finite CLI delivery matrix, 181 LLVM including ignored
host O0/O2 execution, 110 CPU-domain, three packaging, 17 buffer/control, three
host failure/close, 26 Python, one absolute-path policy, two file-limit and 27
tensor tests. Eleven controls are new: six compiler, four native/reference and
one CLI matrix. The selected CLI matrix replaces its predecessor's matrix;
historical receipts retain their own results and are not added together.

The ordinary CLI reports 4,071/4,071 drift checks passing, clean coverage,
manifest, milestone, hierarchy and task lineage, with a ready selected active/99
task. All six source/runtime fields retain complete predecessor values and
SHA-256 identities. Historical shell syntax/inertness checks pass through stdin.
UTF-8 checks cover 5,937 files and 4,572 local documentation links resolve. The
proof index remains within its 300-line entry-router budget. Formatting,
whitespace, offline locked metadata and CLI integration compilation pass.

These are bounded CPU checks, not full-workspace acceptance, stable ABI, physical
device/source-owned parser parity, a complete native engine or self-hosting.

## Remaining Frontier

The subsequent [wrapping negation proof](nuis-native-i32-negation-argument-print-v1.md)
owns that next gate and its separate evidence. The frontier below records this
receipt's original boundary, not the current selected task.

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
The next gate is exact i32 unary negation in lazy comparison arguments: establish
the source/NIR representation and MIN overflow policy before admission, rather
than granting arbitrary casts or assuming ordinary readiness. Runtime narrowing,
non-i32 consumers, references/resource/I/O/async captures, changing backedges,
wide private captures, full engine/backend parity and self-hosting remain separate.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- i32_literal_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- i32_literal_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- i32_literal_argument_print_epoch --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete preceding tensor fields remain embedded losslessly with their
SHA-256 identities. Historical command chains are quoted here-document data
consumed by a no-op; syntax/inertness checks use stdin without replaying them.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `085d786850ea08ac11424c6b693ca469ee409239c69bec0b2c070a0450777691` |
| `next_step` | `6b9cf5e738c4389dee01d036263cc61dd9a7e2011c27440a73615c3352bb4241` |
| `blocker` | `cc15e488dee134e8ed319e7c1a1b66a60d80300eca828fd7c2d4c71f98291cc1` |
| `next_action` | `dd095290d8c0d59e0a9e1b8890aebaf9e777328ab25cfbf5d726d8219a69b12b` |
| `validation_command` | `b40195c0139c0a9bf3dcb5e6f172a4407bb0922aa2a7c13fd064979df5c5e775` |
| `expected_artifact` | `d40cc80d7881772f3a03c6fc45bb09b52465527a419d17739c9304545f422f02` |
