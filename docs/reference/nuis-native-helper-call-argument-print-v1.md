# Native Helper Call Argument Print V1

This bounded CPU worktree follow-up continues the
[lazy checked RHS proof](nuis-native-lazy-call-argument-print-v1.md) on Git
beta-0.17.9 at `2275b6af`. No release version, ABI, source intrinsic, runtime
wrapper or backend-specific producer rule changes.

## Completed Auxiliary Calls

The [shared signature query](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers.rs)
now exposes only completed pure primitive signatures. Every parameter and result
must have the exact unqualified scalar type; unresolved/cyclic dependencies,
transitive effects and record/reference parameters cannot grant read authority.
The existing float-only query delegates to that shared contract and retains its
exact expected-result check.

The [typed operand plan](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs)
tracks whether an operand belongs to a logical RHS. Bool/i64-return helpers gain
field-read permission only in that subtree, including their nested arguments.
They are not newly authorized as an always-reached left operand, an ordinary
initializer or an arbitrary float-call argument. Float-return calls keep their
existing scope. I32-return helper operands, casts and inline float comparisons
remain outside this grant.

Each call validates arity and every argument type before its result is used.
All caller reads share the original epoch/nominal path certificate, cumulative
work/depth and 31-scalar RHS capture union. Calls remain non-total even when a
particular body is simple. The [existing capture builder](../../tools/nuisc/src/lowering/buffer_loop_outline/logical_field_captures.rs)
and [guarded logical lowering](../../tools/nuisc/src/lowering/buffer_loop_outline/logical_conditions.rs)
keep nested calls inside the selected RHS body rather than eager capture arguments.
The sealed ordinary fallback still cannot recover rejected epoch reads.

Five [compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/helper_call_argument_print_epoch_tests.rs)
cover complete signatures in both function orders, five exact parameter kinds,
missing/effectful/cyclic/record/width/arity vetoes, RHS-only nesting, child-entry
rules, capture-union/depth bounds and actual selected/skipped call counts under
float sign wrappers. Earlier rejection fixtures now use genuine effects instead
of pure integral calls this later proof legitimately supports.

Logical argument roots of the newly admitted bool/i64 helpers remain closed in
the print shape profile. Their non-logical primitive arguments and nested scalar
calls are covered; a logical argument tree needs its own scoped shape/guard proof.
Ordinary prefix, return and record-read routes are not widened by analogy.

## Execution And Delivery

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/helper_call_argument_print_epoch.ns)
uses a checked i64 quotient helper and a bool helper that calls it transitively,
inside parent/child AND/OR predicates. It retains nested remainder poison,
versioned bool fields, original integer fields, outer float negation and
independent final State publication.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/helper_call_argument_print_epoch_cases.rs)
models each source helper entry and checked body in order. Native probes expose
actual i64/bool-helper input pairs, selector arguments, raw ready/guard inputs and
raw final State. A first argument failure must prevent the outer bool entry;
a transitive failure must prevent the later selector. Arithmetic NaNs normalize
only in test observers, while non-NaN results and carried/input words remain exact.

[Native/reference controls](../../tools/nuisc/tests/native_application_bridge/helper_call_argument_print_epoch.rs)
use 21 raw healthy sequences per width and twenty failure rows. They cover
skipped helper poison, nested and transitive zero/MIN-overflow, later parent/child
checks, tail failures, exact reached prefixes, canaries, normal/reversed YIR and
last accepted State. Helper counts are accumulated structurally in the independent
oracle, not inferred by scanning marker-valued data. Collision words 101/102 are
included. Owned-record work entries still use the full native word stream.

Success uses 12 loop work, 512 helper entries and at most 32 functions. Separate
128-entry NaN controls preserve atomic exhaustion rejection; no producer budget
is widened. The [finite CLI matrix](../../tools/nuis/tests/native_session_workflow/helper_call_argument_print_epoch.rs)
uses six healthy sequences per width before and after all twenty failures on
fresh/cache and two independent source-free images. Binary/container, host-object,
lifecycle, envelope and packaging identities remain pinned. Decimal CLI values
are not arbitrary NaN-payload evidence.

## Validation

683 distinct selected checks passed: 308 compiler, 117 native/reference, one
finite CLI delivery matrix, 178 LLVM (including ignored host O0/O2 execution),
three packaging, 17 buffer/control, three host failure/close, 26 Python, one
absolute-path policy, two file-limit and 27 tensor tests. Nine controls are new
at this checkpoint: five compiler, three native/reference and one CLI matrix.

The rebuilt ordinary CLI reports 3,939/3,939 drift checks passing, clean coverage,
manifest, milestone, hierarchy and task lineage, and a ready selected task at
active/99. All six source fields match the actual CLI and retain their complete
preceding values and SHA-256 identities; quoted command history passes shell
syntax and inertness checks through stdin. Formatting, whitespace, offline locked
metadata and CLI integration compilation pass. UTF-8 checks cover 5,870 files;
documentation checks resolve 4,449 local links. These are selected CPU checks,
not full-workspace acceptance, physical-device parity or stable ABI evidence.

## Remaining Frontier

The coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99. The next
selected gate is scoped logical bool arguments of auxiliary helpers inside lazy
RHSs. Shape permission must follow the completed signature and lazy subtree,
without opening ordinary call roots, whole records or eager checked work.
Inline float comparisons, i32 arithmetic/return-helper grants, casts, resources,
I/O/async capture, full native ns-nova, stable ABI, backend parity and self-hosting
remain separate gates.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- helper_call_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- helper_call_argument_print_epoch lazy_call_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- helper_call_argument_print_epoch --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete preceding fields remain losslessly embedded in the tensor.
Historical command chains remain quoted here-document data consumed by a no-op,
checked for syntax/inertness through stdin without executing earlier chains.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `16e1d272788c8e4b466745e38fb4774e3e98fa8cbc405d7386c449db23d47cfa` |
| `next_step` | `48bc61c24cabaf93df45b2aabeab371339c79ff787f7105335cb241c40444199` |
| `blocker` | `cb403ffb9a5fc74a5a8fc2c09d0adb10b29827a3ebe13d8a4d70b5e3f98d40b8` |
| `next_action` | `7e7b29555799ee3d1d95425e9a26c82669a3c913e718240097639b3785cbbdae` |
| `validation_command` | `9a6d7afb2a73fe0fcb4d440de731479b855ff747497ea0cb3a0e36b699f73c42` |
| `expected_artifact` | `d08f72f3dc18e943955fd0ca522f54367d8ee5882ee3d38e7404f1b4ff0e82e3` |

