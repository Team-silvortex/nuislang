# Native Logical Helper Argument Print V1

This bounded CPU worktree follow-up starts from Git beta-0.18.0 at
`7f61e841` (2026-10-26), continuing the
[auxiliary-call operand proof](nuis-native-helper-call-argument-print-v1.md).
No release, ABI, source intrinsic, runtime wrapper or backend producer rule changes.

## Scoped Logical Parameters

The [shared shape walk](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_prefix.rs)
now tracks lazy-RHS context independently from logical-root and record-root
permissions. A completed pure bool/i64 helper may accept a logical argument only
at an exact bool parameter and only inside an already-authorized lazy RHS.
Float-return calls retain their existing permission. Exact primitive signatures
still reject unresolved/cyclic dependencies, transitive effects and qualified or
record/reference parameters.

This is a shape grant, not read authority. The
[typed operand plan](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs)
still proves every original epoch/path or independent ready scalar read, argument
kind and arity. Calls remain non-total. Ordinary prefixes/call roots and the
separate record-constructor profile do not gain this permission. The previous
closed-root test remains a negative control outside the new lazy scope.

All roots share 32 logical edges, depth/work limits and a complete 31-scalar RHS
capture union. The existing scalar projection and guarded lowering keep nested
argument work in the reached body, including under private float sign/word wrappers.
No expression replay, whole-record capture or rejected-plan fallback is introduced.

Four [compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/logical_helper_argument_print_epoch_tests.rs)
cover both result kinds and function order, root/ready-child/unready-child reads,
exact bool positions, malformed/record/missing/cyclic/effectful inputs, closed
ordinary roots, cumulative 32/33-edge and 4096/4097-work boundaries, 31/32-scalar
capture unions and selected/skipped nested calls in both float widths.

## Execution And Delivery

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/logical_helper_argument_print_epoch.ns)
passes nested AND/OR arguments to bool-return `accept` and i64-return `weight`.
Both perform a checked quotient after their arguments are evaluated. Parent and
child loops retain original integer/bool fields, changed bool versions, nested
remainder poison, outer float sign wrappers and independent final State publication.

The [independent scalar oracle](../../tools/nuisc/tests/native_application_bridge/logical_helper_argument_print_epoch_cases.rs)
records quotient, positive, accept and weight entries structurally. Native probes
print each actual bool/i64 argument tuple before helper body checks; argument
failure must prevent outer helper entry, and body failure must prevent later
selector/State publication. Marker-valued input words 101 through 104 cannot
masquerade as entries. Raw ready/State words stay exact; only arithmetic NaNs
normalize in test observers.

[Native/reference tests](../../tools/nuisc/tests/native_application_bridge/logical_helper_argument_print_epoch.rs)
use 23 raw healthy sequences per width and twenty fault rows, normal/reversed YIR,
exact reached native prefixes, helper counts, canaries and last-State retention.
Successful fixtures use 12 loop work, 512 helper entries and at most 32 functions;
separate 128-entry NaN cases must reject atomically without raising producer limits.

The [finite CLI matrix](../../tools/nuis/tests/native_session_workflow/logical_helper_argument_print_epoch.rs)
runs six healthy sequences per width before/after all twenty failures on fresh,
cache and two independent source-free images. Binary/container, host-object,
lifecycle, envelope and packaging identities stay pinned. Decimal CLI transport
is not arbitrary NaN-payload evidence.

## Validation

690 distinct selected checks passed: 312 compiler, 120 native/reference, one
finite CLI delivery matrix, 178 LLVM (including ignored host O0/O2 execution),
three packaging, 17 buffer/control, three host failure/close, 26 Python, one
absolute-path policy, two file-limit and 27 tensor tests. Eight controls are new:
four compiler, three native/reference and one CLI matrix.

The rebuilt ordinary CLI reports 3,954/3,954 drift checks passing, clean coverage,
manifest, milestone, hierarchy and task lineage, and a ready selected task at
active/99. Existing drift IDs are retained; three old four-tuple locators were
updated to track the explicit RHS context instead of removing their checks.
All six source/runtime fields retain complete predecessor values and SHA-256
identities. Historical commands pass shell syntax/inertness checks through stdin.
Formatting, whitespace, offline locked metadata and CLI integration compilation
pass. UTF-8 checks cover 5,879 files and documentation checks resolve 4,476 local
links. These are selected CPU checks, not full-workspace acceptance, physical
device parity or stable ABI evidence.

## Remaining Frontier

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99. The next
gate is exact same-kind float comparisons in the existing guarded call-argument
profile, with IEEE unordered/NaN/signed-zero and reached operand-fault evidence.
I32 arithmetic/return-helper grants, casts, general resource/I/O/async capture,
changing backedges, wide captures, full native engine/backend parity, stable ABI
and compiler ownership transfer remain separate.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- logical_helper_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- logical_helper_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- logical_helper_argument_print_epoch --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete preceding fields remain losslessly embedded in the tensor.
Historical command chains are quoted here-document data consumed by a no-op;
shell syntax and inertness are checked through stdin, never by replaying them.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `60b3595860968af987fc0b2770b1cda51d2607c877f62634068c3683ba5add88` |
| `next_step` | `f8fbc819dd86b1378f9fb69a19c648cc6dcafb14399aa89c7eff694f4c492b9b` |
| `blocker` | `d069a09dd0d99fd6b66efccc866dfe0767f3d1ac61ef4309122565f418345ffa` |
| `next_action` | `9ed540a1dc609a9b4322cf388716ec02cd4a6620da01598888b3bbbedefa0697` |
| `validation_command` | `3c44fee8b49078f12a9ae15b36a02d1d0302710c5f7d5f5796650a201b74b308` |
| `expected_artifact` | `093dcb6e1c1372aff8cd49d147cdecf0c0f204912694858f27ed92dadb40443d` |

