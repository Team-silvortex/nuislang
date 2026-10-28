# Native I32 Widening Argument Print V1

Audit date: 2026-10-28. This bounded CPU worktree proof follows the
[wrapping i32 negation proof](nuis-native-i32-negation-argument-print-v1.md).
The committed baseline remains Git beta-0.18.2 at `5bbd4de8` (2026-10-28);
this follow-up does not create a release. Earlier receipts retain their own
executed totals and historical permission boundaries.

## Explicit Signed Widening

The [source entry](../../tools/nuisc/src/frontend/call_routing.rs)
`i64_from_i32(value)` accepts exactly one plain owned i32 argument and no type
arguments. It emits the existing NIR CastI32ToI64 node; LLVM already lowers it
to signed extension. No new runtime opcode, ABI or backend-specific producer
is introduced. The operand is evaluated once. MIN, negative one and MAX retain
their signed mathematical values in i64; this is neither zero extension nor
runtime narrowing.

[AST inference](../../tools/nuisc/src/frontend/types/ast_calls.rs) and
[lambda local inference](../../tools/nuisc/src/frontend/lambda_expansion_types.rs)
recognize the same exact input/output types. Generic calls and a lambda-local
`let wide = i64_from_i32(x)` agree. Wrong arity, generic arguments, i64/bool/float,
modified reference/optional/generic types and double widening remain errors.
A raw integer literal is i64 and needs the existing explicit i32 constructor.

The [frontend controls](../../tools/nuisc/src/frontend/tests_frontend_core/i32_widening.rs)
check a once-only call operand, rendered source spelling, verification, signed
boundaries and generic/lambda behavior. They distinguish i32 wrapping negation
before extension from i64 negation after extension, including MIN.

## Scoped Comparison Permission

The [argument proof](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs)
admits explicit widening only within an already-authorized lazy RHS. It first
proves an exact i32 operand using its original field/epoch authority or existing
ordinary readiness. Widening preserves captures and fallibility, but marks the
i64 result comparison-only and clears ready-atom/ready-predicate flags.

A restricted argument may cross a pure scalar helper only when the result
preserves that exact argument kind. Thus explicitly widened i64 can pass through
multiple i64-returning helpers before a same-kind comparison consumes the
restriction. An i32-restricted argument cannot silently pass into an i64-returning
helper: the explicit conversion must be in the argument tree. Mixed restricted
kinds, bool/float/narrowing consumers and ordinary-profile escapes stay closed.

The extension node and operand both count toward the shared 4096-node work
budget. Operand depth increases by one. The 31-scalar helper-required capture
union, 32 logical edges, depth 64, original-leaf visitation and child readiness
remain independent limits. No standalone widening grant or general i64 arithmetic
over these restricted values is inferred. Arithmetic within a separately proved
pure helper retains its ordinary source semantics.

Six [profile controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/i32_widening_argument_print_epoch_tests.rs)
cover six comparisons and both operand positions, normal/reversed declarations,
root/ready-child/unready-child reads, exact call consumers and rejection of
cross-kind/effectful/cyclic/record inputs. A 2043-operation i32 tree plus one or
two i64 helper calls separates exactly 4096/4097 work. Separate controls cover
depth 64/65, capture union 31/32 and logical roots 32/33. Source checks retain
zero and MIN/-1 faults before conversion and suppress them when skipped.

Five predecessor rejection controls now reject double widening rather than the
newly admitted exact single conversion. Stable drift-check IDs retain their
identity while their call-provenance locator follows the kind-preserving rule.
This advances only that explicit veto, not earlier ordinary-read permissions.

## Native Execution And Delivery

The [Nuis fixture](../../tools/nuisc/tests/native_application_bridge/i32_widening_argument_print_epoch.ns)
extends checked quotient/negation/literal/remainder arguments, then passes them
through relay64 and checked64. relay64 adds 4294967296 in its own pure body.
This intentionally places checked64 arguments and return values outside the i32
range. Parent/child fields retain distinct versions, and an unconverted i32
helper path remains as an adjacent compatibility control. No foreign runtime
wrapper is added.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/i32_widening_argument_print_epoch_cases.rs)
uses explicit signed i32-to-i64 values, independently masked i32 arithmetic and
a full-width i64 offset. Native probes record actual i32 and i64 arguments
without narrowing the latter. Separate counters identify all four checked/relay
helpers; marker-valued inputs cannot substitute for call-count evidence.

The [native/reference matrix](../../tools/nuisc/tests/native_application_bridge/i32_widening_argument_print_epoch.rs)
retains 33 raw healthy sequences per float width, twelve signed input pairs,
six finite CLI sequences and twenty fault rows. Actual negative relay64 words
and checked64 words above i32 MAX must occur. Normal/reversed YIR, exact float
State/ready words, flushed failure prefixes, output canaries, last accepted State
and faulted-session close are checked. Only arithmetic NaNs normalize in test
observers. Bounds remain 12 loop work, 512 helper entries and at most 32 functions;
separate 128-entry NaN controls must reject atomically.

The [CLI matrix](../../tools/nuis/tests/native_session_workflow/i32_widening_argument_print_epoch.rs)
checks fresh and cached artifacts and two independent source-free restores,
healthy execution before/after faults, binary/container and host-object identity,
lifecycle, envelope and packaging identity. Finite decimal CLI inputs do not
prove arbitrary NaN-payload transport. The ordinary CLI is not rebuilt while
delivery is running.

## Validation

1,029 distinct selected checks passed: 563 compiler/front-end/higher-order/render,
94 native/reference, one finite CLI delivery matrix, 181 LLVM including ignored
host O0/O2 execution, 110 CPU-domain, three packaging, 17 buffer/control, three
host failure/close, 26 Python, one absolute-path policy, two file-limit, 27 tensor
and one adjacent effectful-if integration. Fifteen controls are new: four frontend,
six argument, four native/reference and one CLI matrix. The selected CLI matrix
replaces its predecessor; historical receipts are not added together.

The ordinary CLI reports 4,101/4,101 drift checks passing, clean coverage,
manifest, milestone, hierarchy and task lineage, with a ready selected active/99
task. All six source/runtime fields retain complete predecessor values and
SHA-256 identities. Historical shell syntax/inertness checks pass through stdin.
UTF-8 checks cover 5,953 files and 4,601 local documentation links resolve. The
proof index remains within its 300-line entry-router budget. Formatting,
whitespace, offline locked metadata and CLI integration compilation pass.

These are bounded CPU checks, not full-workspace acceptance, stable ABI,
physical GPU/provider parity, a complete native engine or compiler self-hosting.

## Remaining Frontier

The following frontier is historical. The later [restricted i64 arithmetic proof](nuis-native-i64-arithmetic-argument-print-v1.md)
advances only its stated five-operation permission; this receipt retains its
original executed totals and source scope.

The coordinate stays `standard-library/ns-nova/persistent-application-session`,
active/99. Next is same-kind i64 arithmetic over explicitly widened restricted
values: define permitted wrapping/checked operations and retain comparison-only
provenance before admission. General casts, runtime narrowing, arbitrary ready
reads, changing backedges, wide private captures and resource/I/O/async capture
remain separate gates. No GPU/engine completion is inferred from this CPU proof.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- i32_widening --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- i32_widening_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- i32_widening_argument_print_epoch --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete preceding tensor fields remain embedded losslessly with their
SHA-256 identities. Historical commands remain quoted here-document data consumed
by a no-op. Shell syntax/inertness checks use stdin, not argv or command replay.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `e38105e3229eecef0decde4d1538fb14b7573e976d5640e21ef574ac7906c29c` |
| `next_step` | `fda35c77ada04577ba4e363be3ca634aac9e5995db8152cc0c84293b29a447b4` |
| `blocker` | `7a70d37c966f3069958500ac3e0b0184d4d4d3e84b70468e456a617ba2889345` |
| `next_action` | `d530e227c94cb652483cc3d367835e22687b6340f928d2ffb4c290e946ff5ef6` |
| `validation_command` | `9cd5fd7a937973526f2f6650e33acb335c8e52ba5ed7e971e36e6a53f154fe4f` |
| `expected_artifact` | `78e0d42069198013d344b8f97508b3bb94b4630d5a58576f791c4d07d7233235` |
