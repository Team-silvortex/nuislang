# Native I32 Negation Argument Print V1

Audit date: 2026-10-28. This bounded CPU worktree proof continues the
[exact i32 literal proof](nuis-native-i32-literal-argument-print-v1.md), against
Git beta-0.18.2 at `5bbd4de8` (2026-10-28). Earlier receipts retain
their original source baselines, scope and executed totals. There is no new
source token, intrinsic opcode, runtime ABI or engine/backend-specific producer.

## Wrapping Negation, Not General Widening

Owned plain i32 now has builtin unary minus, consistently with existing wrapping
i32 Add/Sub/Mul: negating MIN returns MIN, and nested negation does not trap.
The [front end](../../tools/nuisc/src/frontend/unary_lowering.rs) emits one exact
shape: cast back to i32 after i64 zero-minus-sign-extended-i32. The operand occurs
once. The wide subtraction cannot overflow i64; the final truncation supplies
the i32 wrapping result. No generic i32-to-i64 conversion operator is exposed.

The [generic classifier](../../tools/nuisc/src/frontend/generic_rewrite/exprs_operators.rs)
recognizes the same exact owned i32 type. As with existing numeric builtins,
plain i32 takes builtin precedence over user Negatable implementations. Modified
reference/optional/generic forms are not newly builtin, and nonprimitive trait
rules remain separate. AST/lambda inference already preserves the operand type;
no broader inference rule is added.

The [argument proof](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs)
admits only the complete zero/subtract/sign-extend/truncate shape inside an
already-authorized lazy RHS. It proves the original operand as exact i32 with
its existing original-field/epoch authority or independent ordinary readiness.
Negation preserves operand captures and fallibility, but never grants ready-atom
or ready-predicate status. Its result remains comparison-only through exact i32
calls, canonical wrapping arithmetic, checked Div/Rem and further negations.
A same-kind comparison must consume that permission.

All four representation nodes are charged, including the zero leaf. Operand
proof begins three depth levels below the outer cast. Missing extensions,
nonzero/computed zero, mixed kinds, effects/cycles/records, unready child reads,
standalone widening, non-i32 consumers and ordinary-profile escapes remain
rejected. The 31-scalar helper-required capture union, 32 logical edges, depth
and shared 4096-node work budgets remain independent and unchanged.

The [front-end controls](../../tools/nuisc/src/frontend/tests_frontend_core/unary_i32.rs)
and exact builtin-type control cover representation, MIN wrapping, nesting,
once-only call operands, generic/lambda agreement, invalid modified types and
unchanged i64/float numeric behavior. An identity Negatable implementation for
i32 deliberately checks that generic specialization uses the same builtin rule.

Six [argument controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i32_negation_argument_print_epoch_tests.rs)
cover all six comparisons, both operand positions, reversed declarations,
root/ready-child/unready-child reads, malformed shapes and provenance escapes.
A negated balanced 2042-operation tree plus zero/one helper call distinguishes
exactly 4096/4097 proof work; separate controls pin depth, capture union, logical
roots and original leaf visitation order. Source execution crosses f32/f64,
signed boundaries, AND/OR gates and first/second helper faults. Actual callee
arguments expose wrapped negation, not just the comparison boolean. Composed
Div/Rem still reject zero and MIN/-1 before negation; skipped poison stays skipped.

## Print Effect Frontier

A new unchanged-numeric-profile control exposed a pre-existing effect-order gap:
the [two-print-arm if shortcut](../../tools/nuisc/src/lowering/if_lowering.rs)
linked the selected value to its print, but did not chain that print into the
surrounding statement-effect frontier. A preceding top-level print could execute
later, and the following statement did not inherit the missing frontier.

The selected print now uses the same statement-effect chaining as ordinary and
guarded prints. The regression keeps the expected source order, checks both
runtime gate values and verifies transitive effect paths before/after the if.
It does not require direct edges when a valid intervening helper exists.
The [adjacent effectful-if check](../../tools/nuisc/tests/effectful_if_compile.rs)
retains selected foreign-call return handling. This fix does not widen the
shortcut's branch-expression or effect admission policy.

## Execution And Delivery

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/i32_negation_argument_print_epoch.ns)
uses triple negation over a checked quotient-plus-one in the parent, negated
versioned child fields, a real helper argument of negative 107 and a negated
checked remainder expression. Parent/child field versions and floating State
remain separately observable. No C/ObjC/Rust runtime wrapper is added.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/i32_negation_argument_print_epoch_cases.rs)
negates signed i64 values and restores the signed modulo-2^32 range, independently
of the compiler's extension/truncation tree. Existing zero/MIN/-1 checks remain
explicit. Native probes sign-extend actual i32 arguments; structural counts
separate true callee entries from marker-valued data. Float State/ready words
stay exact; only arithmetic NaNs normalize in test observers.

The [native/reference matrix](../../tools/nuisc/tests/native_application_bridge/i32_negation_argument_print_epoch.rs)
retains 33 raw healthy sequences per float width, twelve signed input pairs,
six finite CLI sequences and twenty explicit fault rows. Normal/reversed YIR,
actual argument words, counts, flushed failure prefixes, output canaries and
last accepted State are checked. Success retains 12 loop work, 512 helper entries
and at most 32 functions. Separate 128-entry NaN controls must reject atomically.

The [CLI matrix](../../tools/nuis/tests/native_session_workflow/i32_negation_argument_print_epoch.rs)
checks fresh/cache delivery and two independent source-free restores, healthy
execution before/after faults, and unchanged binary/container, host-object,
lifecycle, envelope and packaging identities. Finite decimal transport does not
establish arbitrary NaN-payload CLI parity.

## Validation

1,015 distinct selected checks passed: 553 compiler/front-end/higher-order/render,
90 native/reference, one finite CLI delivery matrix, 181 LLVM including ignored
host O0/O2 execution, 110 CPU-domain, three packaging, 17 buffer/control, three
host failure/close, 26 Python, one absolute-path policy, two file-limit, 27 tensor
and one adjacent effectful-if integration. Sixteen controls are new: five
frontend/builtin/effect-order, six argument, four native/reference and one CLI
matrix. The selected CLI matrix replaces its predecessor's matrix; historical
receipts retain their own results and are not added together.

The ordinary CLI reports 4,086/4,086 drift checks passing, clean coverage,
manifest, milestone, hierarchy and task lineage, with a ready selected active/99
task. All six source/runtime fields retain complete predecessor values and
SHA-256 identities. Historical shell syntax/inertness checks pass through stdin.
UTF-8 checks cover 5,945 files and 4,587 local documentation links resolve. The
proof index remains within its 300-line entry-router budget. Formatting,
whitespace, offline locked metadata and CLI integration compilation pass.

These are bounded CPU checks, not full-workspace acceptance, stable ABI, physical
device/source-owned parser parity, a complete native engine or self-hosting.

## Remaining Frontier

The following frontier is historical. The later [explicit widening proof](nuis-native-i32-widening-argument-print-v1.md)
advances only its stated lazy comparison permission; it does not revise this
receipt's executed scope or totals.

The coordinate stays `standard-library/ns-nova/persistent-application-session`,
active/99. Next is an explicit sign-preserving i32-to-i64 widening contract for
lazy comparison arguments: establish source/NIR representation, permitted
consumers and provenance before admission, not runtime narrowing or blanket
argument readiness. Current internal negation extensions do not grant that
separate conversion permission. References/resource/I/O/async captures, changing
backedges, wide private captures, full engine/backend parity and actual compiler
self-hosting remain separate gates.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- i32_negation unary_i32 --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- i32_negation_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- i32_negation_argument_print_epoch --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test effectful_if_compile --locked -j1 -- --test-threads=1
```

## Predecessor Lineage

All six complete preceding tensor fields remain embedded losslessly with their
SHA-256 identities. Historical commands remain quoted here-document data consumed
by a no-op; syntax/inertness checks use stdin without replaying older commands.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `d7fd740ce4dd225ff4e1dd88507cd6665692776b935be132bccec8d98907d292` |
| `next_step` | `50594b3df8ca9cd20b955971eecb0d5b842755f97aa1eb0b23bceaa55e699abe` |
| `blocker` | `d1fd579f80dfe6b35f57aba69810aa450a17d48e1fc0c6a535d93d5a99768ac3` |
| `next_action` | `e9406f85d025d17a6b7cbbce076eb0c313591377f271d508ae0b940eade1eb58` |
| `validation_command` | `28ef2848b2ddd16c8430d710d5c863683e9ef676acd2cf8d99c886f7a2acaa9f` |
| `expected_artifact` | `250f59e9f6580d9b9536e31e96e73aff12604f8f37b9451cdfc3d6bc09c2c36c` |
