# Native Float Call Field Epoch Print V1

This bounded CPU worktree follow-up continues the
[binary float proof](nuis-native-binary-float-field-epoch-print-v1.md) on Git
beta-0.17.9 at `2275b6af`. It changes no release version, ABI, source intrinsic,
runtime wrapper or backend policy. It is not heterogeneous application closure.

## Completed Pure Calls

The [shared typed plan](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs)
now composes completed pure f32/f64-return calls with same-kind arithmetic and
canonical sign operations inside exact float arguments of completed pure-i64
print helpers. It checks each original argument against the callee's exact
parameter type, not just its result. The [signature query](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers.rs)
uses the completed pure catalog, rejecting unresolved, cyclic, transitively
effectful, record-parameter and qualified signatures.

All arguments must be exact primitive scalar types. Each field retains its
original entry/ownership/nominal-layout/path certificate; other leaves require
independent ordinary readiness. Nonfloat parameters accept scalar atoms and
certified direct fields, not newly authorized inline integer/bool computation,
casts or integral-return calls. Bind such an ordinary ready value before the
print when its existing source route permits it. The [epoch owner](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch.rs)
does not update ambient readiness or grant ordinary initializer/condition reads.

One iterative stack shares depth 64 and work 4096 across calls, arithmetic and
signs. Arity preflight bounds pending work before pushing arguments. Arguments
are checked left before right; the source expression is rendered without replay,
reassociation or synthetic ready substitution. Execution retains existing helper
entry accounting. No call-name, fixture-name or expression-template producer case
is added.

Six [compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/float_call_print_epoch_tests.rs)
cover complete signatures, all five parameter kinds, wrong slots/arity/width,
cycles/effects, original root/child certificates, stale layouts, ordinary and
ancestor-read rejection, cumulative limits, raw NIR and selected checked faults.
Older rejection fixtures now use genuinely effectful float helpers rather than
pure calls this later admission legitimately supports.

## Execution And Delivery

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/float_call_field_epoch_print.ns)
uses nested identity/scale calls over parent and child fields, both noncommutative
operand orders, invariant float input and separate checked integer guards.
The [independent oracle](../../tools/nuisc/tests/native_application_bridge/float_call_field_epoch_print_cases.rs)
executes each operation at its actual f32/f64 precision and models source-order
helper entries before checked failures. It shares schedule data with the prior
binary proof, not that proof's arithmetic result implementation.

Native probes record actual ready/guard inputs, each helper's arguments in order,
computed observer input and final raw State. Only computed NaN arguments normalize
for cross-backend comparison; signed zeros, finite values and infinities remain
bit-exact. Raw ready and carried State retain payload bits, including NaNs.
Production computation is not changed by test observers.

[Native/reference controls](../../tools/nuisc/tests/native_application_bridge/float_call_field_epoch_print.rs)
exercise 21 raw healthy sequences per width, adding skipped poisoned arguments
and zero-trip rows to the shared schedule. Sixteen failure rows include the prior
integer/float/tail failures plus left, right and both helper-guard failures.
Exact reached prefixes, normal/reversed YIR, canaries, structural scalar-call
counts, last accepted State and faulted-session behavior remain checked. Owned
record calls have no scalar reference-call event; the complete native word stream
pins their work entry instead.

The witness uses 12 loop work, 512 helper entries and at most 32 functions.
Separate 128-entry NaN controls retain atomic exhaustion rejection. These are
explicit fixture limits, not a wider producer budget.
The [finite CLI matrix](../../tools/nuis/tests/native_session_workflow/float_call_field_epoch_print.rs)
uses seven healthy sequences per width before and after sixteen failures on
fresh/cache and two independent source-free images. Binary/container, host
objects, lifecycle, envelope and packaging identities remain pinned. Decimal
CLI input is not arbitrary NaN-payload evidence.

## Validation

The locked serial validation chain passed 627 distinct tests: 261 targeted
compiler, 108 native/reference, one CLI delivery, all 178 LLVM library tests,
three packaging, 17 buffer-loop, three host-failure, 26 maintenance-script,
one absolute-path policy, two file-limit and 27 tensor tests. Ten new controls
are included; focused repetitions count only once. The LLVM suite includes the
normally ignored host-clang record round trip at O0 and O2. This is not a claim
that every workspace suite or physical backend ran.

Both widths passed fresh/cache and two independent source-free images, with
seven healthy sequences before and after all sixteen failures. Formatting,
whitespace, delivery-test compilation, the ordinary CLI build, locked offline
metadata, 4,410 local documentation links and 5,847 UTF-8 files passed.

The rebuilt ordinary CLI reports all 3,893 drift checks clean, as well as clean
coverage, manifest, milestone, hierarchy and task-card lineage validation. All six
live selected fields exactly match source and retain their complete predecessor
values and SHA-256 digests. Historical command syntax and inertness checks passed
through stdin without executing prior chains. The selected task remains ready
and active/99; CPU acceptance does not certify GPU/Linux parity, stable ABI,
performance targets or self-hosting.

## Remaining Frontier

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Computed nonfloat call arguments still need a shared typed argument/read proof
when they use pre-update fields. Ordinary ready locals retain their independent
route. General effects/resources/I/O/async capture, stable ABI, broader backend
parity, complete native ns-nova and self-hosting remain separate gates.
The later [computed call-argument proof](nuis-native-computed-call-argument-print-v1.md)
advances typed integer/bool trees with total logical RHSs; this checkpoint's
historical totals and boundaries remain unchanged.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- float_call_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- float_call_field_epoch_print --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- float_call_field_epoch_print --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete prior fields remain losslessly embedded in the tensor. Historical
validation chains are quoted here-document data consumed by a no-op, not commands
to execute again. Syntax and inertness checks read that data through stdin.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `fdaa9e9d7b3cbdb753afa5d38402cac036f104f8f44849d69cc77a71b7130805` |
| `next_step` | `822485e6b7d743217b0af8ae5ec596d019082f0d7274d2df9c12ba31faa55322` |
| `blocker` | `3e5198957f4509f29515545c04a2fd45f16c3f70726966933af5ec6ef8dc2e0d` |
| `next_action` | `2e17048ba284dd4078bdfec4cd7a5719a54de97eda62154501e9516cfb9c89cd` |
| `validation_command` | `0f03080cd6245adc2545b4727179b515cb67cf8829c78c49590405c402b28754` |
| `expected_artifact` | `b69d7b1667a2a721b523d763397eaac2e313aa871dc80ec1449154118e461387` |

