# Native Bool Equality Argument Print V1

This bounded CPU worktree proof follows the
[logical i32 parameter proof](nuis-native-logical-i32-call-argument-print-v1.md)
on Git beta-0.18.1 at `c3320d9e` (2026-10-26). It adds no ABI, source intrinsic,
runtime wrapper or backend-specific producer rule. Earlier receipts retain their
historical source baselines and acceptance totals.

## Eager Comparisons, Guarded Operands

The [shape policy](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_prefix.rs)
opens computed Eq/Ne operands only in the existing typed call-argument profile.
Ordinary prefixes, record fields and general call roots keep their closed policy.
Both operands inherit the existing context: equality itself does not authorize a
lazy RHS. Each operand's internal AND/OR still has its own guarded evaluation.
This admits shape, not new read or write authority.

The [typed argument plan](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_binary_float.rs)
now accepts computed exact bool Eq/Ne operands instead of requiring bool atoms.
Both operands execute once left-to-right, even when the first is false for Eq or
true for Ne. Completed pure signatures, exact primitive types and original epoch/
nominal field reads remain required. New i32 call results stay comparison-only;
arbitrary consumers, arithmetic, casts, missing/effectful/reference helpers and
mixed types are not granted permission by wrapping them in bool equality.

All operands contribute to the complete 31-scalar helper-required capture union.
Equality is not a new logical edge or a new guard; its internal AND/OR edges share
the existing 32-edge budget and the shared depth/work bounds. The conservative
ready-predicate distinction remains unchanged. Bool XOR is still a separate gate.

Six [compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/bool_equality_argument_print_epoch_tests.rs)
cover normal/reversed declaration order, root/ready-child/unready-child reads,
ordinary/record-profile vetoes, exact types, missing/effectful dependencies and
comparison-only provenance. Constant left operands cannot suppress proof of the
right operand. Read order, 31/32 captures, 32/33 logical edges, 4096/4097 work and
depth rejection are pinned. Source controls cross Eq/Ne, both float widths,
AND/OR in both operands, outer gates, signed extrema and first/second failures,
observing the actual checked i32 arguments and outer selector bools.

## Execution And Delivery

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/bool_equality_argument_print_epoch.ns)
compares two logical operands in a parent Eq and a post-child Ne, inside bool
parameters of a completed pure i32 selector. Its i32 result is consumed by a
comparison. Owned code/bool fields change at parent and child updates; float
State, raw ready words and outer arithmetic remain separately observable.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/bool_equality_argument_print_epoch_cases.rs)
evaluates each side into a separate value before comparing. It tracks source
helper entries structurally. Native probes print actual bool and sign-extended
i32 arguments, including marker words used as ordinary payload data. Ready/State
float words remain exact; only arithmetic NaNs normalize in test observers.

[Native/reference controls](../../tools/nuisc/tests/native_application_bridge/bool_equality_argument_print_epoch.rs)
use 35 raw healthy sequences per float width, thirteen signed pairs, six finite
CLI sequences and sixteen explicit fault rows. They retain healthy skipped inner
poison and reached second-operand failures even after a false/true first operand,
without filtering fault rows by observed outcomes. Normal/reversed YIR, argument
streams, canaries, failure prefixes, helper counts and last accepted State remain
checked. Success keeps 12 loop work, 512 helper entries and at most 32 functions;
separate 128-entry NaN controls must reject atomically rather than raising limits.

The [CLI matrix](../../tools/nuis/tests/native_session_workflow/bool_equality_argument_print_epoch.rs)
covers fresh/cache delivery and two independent source-free restores, with every
finite sequence before and after all fault rows. Binary/container, host-object,
lifecycle, envelope and packaging identities stay pinned. Finite decimal delivery
is not arbitrary NaN-payload evidence.

## Validation

664 distinct selected checks passed: 336 compiler, 70 affected native/reference,
one finite CLI delivery matrix, 178 LLVM (including ignored host O0/O2 execution),
three packaging, 17 buffer/control, three host failure/close, 26 Python, one
absolute-path policy, two file-limit and 27 tensor tests. Eleven controls are new:
six compiler, four native/reference and one CLI matrix. Earlier receipts retain
their own historical acceptance totals.

The rebuilt ordinary CLI reports 4,010/4,010 drift checks passing, clean coverage,
manifest, milestone, hierarchy and task lineage, and a ready selected active/99
task. All six source/runtime fields retain complete predecessor values and SHA-256
identities. Historical command syntax/inertness checks pass through stdin.
Formatting, whitespace, offline locked metadata and CLI integration compilation
pass. UTF-8 checks cover 5,907 files and documentation checks resolve 4,519 local
links. The proof index remains within its 300-line entry-router budget.

These are bounded CPU controls, not full-workspace acceptance, physical device
parity, stable ABI, a full native engine or completed self-hosting.

## Remaining Frontier

The coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99. The next
gate is computed bool XOR in the same typed call-argument profile: eager ordered
operands, independently guarded internal logic, exact reads/types/provenance and
all budgets. The subsequent [bool XOR proof](nuis-native-bool-xor-argument-print-v1.md)
covers that scoped follow-up without rewriting this historical acceptance.
Ordinary profiles, i32 arithmetic/casts, resource/I/O/async captures,
changing backedges, wide private captures and full engine/backend parity stay
separate gates. No new stable/100 or self-hosting claim follows from this proof.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- bool_equality_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- bool_equality_argument_print_epoch --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- bool_equality_argument_print_epoch --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete preceding fields remain losslessly embedded in the tensor.
Historical command chains remain quoted here-document data consumed by a no-op;
syntax/inertness checks use stdin without replaying older commands.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `d2c352e1be3d94a1e1d39cfb19c2d4d0df42d1acc7fcfe85d810cfcb7d2cdc8b` |
| `next_step` | `6e63e48bae386862931c48833267b460376abfe8217b45935e80895c314154f2` |
| `blocker` | `b88bb4962842d3f2c36528d3b7e9ede4b5c9ae32c0eef78768e0ebaae9aa8f39` |
| `next_action` | `f0d6f249033269c6d665130dae85599caa99a94a9bc911cef57453c772504e77` |
| `validation_command` | `03925bd2e0fc0e1606d822a82c64bfdac068a53397a546da8607238e0d721ea0` |
| `expected_artifact` | `8141c6e4fb8990bb84cfe22ca63021238544f09c60420ff4d8c51d16d7939aad` |

