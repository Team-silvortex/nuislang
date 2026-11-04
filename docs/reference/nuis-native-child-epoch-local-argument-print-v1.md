# Native Child Epoch-Local Argument Print V1

Audit date: 2026-11-04. Checkout baseline: `dc87ec43`; latest version-label
commit: beta-0.18.4, `ceaa2caa`. This worktree follows the accepted
[root-owned route](nuis-native-epoch-local-argument-print-v1.md), retaining its
complete six-field tensor checkpoint. No version or release is created here.

## Entry Ownership, Not Depth Permission

The [borrowed owner](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/epoch_block_owner.rs)
binds one complete iteration-effects block to one actual entry epoch. Both the
root validator and the child-loop validator construct this owner at their real
entry boundary. Pair admission checks block/epoch identity; the existing plan
also checks scope, layouts, catalog and definition/observer positions.

Ordinary branch recursion calls the non-owning block validator. It does not
inherit the parent's owner, regardless of nesting depth. This replaces the
root-only depth discriminator, not the source nesting limit. Owner-less epoch-pair
admission is closed even at depth zero; ordinary ready-local rules are unchanged.

The [child boundary](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/literal_loops.rs)
still creates its fresh epoch from `changed` intersected with already-available
entry names, before resetting child writes. A parent's print-only permission
does not make a missing value ready, and cannot authorize a grandchild entry.
The child receives its actual normalized complete effects block, not a copied
definition/consumer fragment or an ancestor's effects view.

The original single-definition, adjacent observer, literal-prefix, one lazy
comparison-scoped use, purity and exact-kind requirements remain. The eager
initializer retains the ordinary non-logical shape gate. Existing logical-root,
work/depth/capture, loop-nesting and helper-entry limits are not widened. No
ordinary local availability or write authority is published by pair admission.

## Real Nested Source

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/child_epoch_local_argument_print.ns)
has a one-trip outer loop and a dynamic child loop. An explicit identity
`finish(total, 1)` self-update makes the outer scalar ready before constructing
the child record. The fresh record is therefore available at child entry; child
writes then reset ordinary readiness without losing that entry certificate.

Inside the child, the original i32 field is widened, offset, XORed and passed
through a checked helper to initialize an i64 local. The adjacent observer uses
it once in a lazy comparison. Only afterward does the record change. Each child
backedge uses the new field, while callback re-entry consumes accepted State
rather than the decoy initial argument. The original Let and Print remain in
place through existing normalization; there is no new opcode, ABI or emitter.

The outer identity update has no public print/State effect, so the existing
independent bit-by-bit XOR and eager-fault oracle remains the semantic model.
The reference probe additionally counts `finish` calls: child trips plus tail
plus one outer-entry identity call. This guards against silently skipping that
entry work while keeping the oracle independent of backend arithmetic.

## Execution And Refusals

The [child native controls](../../tools/nuisc/tests/native_application_bridge/child_epoch_local_argument_print.rs)
reuse parameterized source-template probes, not a second implementation of the
root driver. Sixteen raw sequences and nine fault rows per f32/f64 kind cover
State bits, zero words, eager initialization, lazy faults, later update/tail
faults, canaries, reversed YIR, accepted-state preservation and faulted close.

The witness configures four shared loop-work units: one outer plus at most three
child iterations. This accounts for the new real outer loop; the root witness
keeps its original three units. Helper entries remain 64. An independent limit-8
probe still rejects before callback success or State publication.

Source controls reject a branch borrowing its child's owner, an unready outer
record, parent-print laundering, an unready grandchild, aliases, assignment,
duplicate/non-lazy use and hidden initializer logical trees. The reassignment
control uses a legal mutable declaration, so the admission rejection is not
mistaken for an earlier frontend immutability error. Native compile/emit
refusals separately retain branch, duplicate and non-lazy boundaries.

The [child CLI control](../../tools/nuis/tests/native_session_workflow/child_epoch_local_argument_print.rs)
uses the same source-template delivery workflow as the root. Twelve finite
sequences and nine fault rows per kind run through fresh/cache artifacts and
two independent restores after deleting source, manifest and project cache.
Binary/container, host-object, envelope, lifecycle and packaging identities are
checked. Direct probes check flushed fault prefixes; trapping CLI stdout is
only checked to the extent its child flushed it.

## Validation

The final source state passed 313 distinct selected tests/checks:

| Selected Set | Passed |
| --- | ---: |
| Compiler temporaries/epoch/iteration-print/effect library regressions | 220 |
| Root/child epoch-local, ready-local, i64 XOR/arithmetic native controls | 21 |
| Root and child f32/f64 fresh/cache/two source-free delivery matrices | 2 |
| Python CI-script regressions | 40 |
| Repository file budgets | 2 |
| Host-absolute-path policy | 1 |
| Development tensor tests | 27 |

Nine controls are new relative to the preceding root receipt: three child source
tests, one owner identity test, four child native/reference controls and one
child CLI matrix. Existing root test names remain, and both root and child are
rerun after sharing their parameterized workflow. Earlier receipts, matrix rows,
preliminary failures and repeated runs are not added to the 313 total.

Structural validation passed for 6,004 UTF-8 text files, 4,688 local Markdown
links, offline locked metadata, formatting and whitespace. Workspace architecture
remains 29 packages, six domains and 74 normal/build edges. The proof index stays
at 300 lines; the dispatch validator is 532 lines and drift registry 781 lines.
The rebuilt ordinary CLI report matches all six source fields and full
predecessor hashes, retains all 4,164 preceding drift IDs and passes all 4,174
checks. Coverage, hierarchy and task lineage are clean. The selected coordinate
remains active/99.

This is bounded native host CPU evidence on the current machine, not Linux/GPU
provider parity, arbitrary resource/I/O/async captures, a complete native engine,
stable ABI, whole-program memory-safety proof or self-hosting.

## Next Gate

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. Next is repeated parent entry: zero/dynamic parent trips, fresh child
seeding on each parent backedge, and faults after earlier accepted child work.
The one-trip outer witness does not prove that broader execution matrix. The
[subsequent repeated-parent receipt](nuis-native-parent-reentry-argument-print-v1.md)
records that separate matrix; the historical scope and totals here are unchanged. Wider
prefixes, multiple local uses and ancestor-grant inheritance remain closed.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- temporaries:: print_epoch:: iteration_prints:: iteration_effects:: --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- epoch_local_argument_print:: local_scalar_argument_print:: i64_xor_argument_print_epoch:: i64_arithmetic_argument_print_epoch:: --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- epoch_local_argument_print --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete preceding fields remain lossless with these identities. The
preceding validation command is quoted no-op here-document data, checked through
stdin for syntax and inertness without replaying historical work.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `5c0ca1ce8680b250ceebbf6abb23b021718aaa88e973a5ef2f3ee98d8dcd86d9` |
| `next_step` | `6f53a993714406b3326329064506eb676621f06c02e8edb89c16b1fc24d4f326` |
| `blocker` | `ae76d0b5b3984111b1b2440d9eff0ecef0ad707cdd01951e7009d6d2fe45c6b0` |
| `next_action` | `15a052bedf0105ca9d79664ca7d6c7b1d3d5483a5b051a5f86e2f4ef231b331c` |
| `validation_command` | `5734d22ecbfe2f7c08ea9a0a6c446622ab7e96e162abc37ef9c7924137ac8e76` |
| `expected_artifact` | `bdc37d510c7b81d9e7f100c1c09dfa54f361ddccbe16f2c94bfa0403950b9ab8` |
