# Beta 0.17 Snapshot

Audit date: 2026-10-25. This minor-line anchor separates committed identity,
worktree evidence and future gates. It is not a new release, compatibility freeze
or completed self-hosting announcement. Cargo and protocol versions are independent.

## Committed Identity

| Identity | Git Commit | Date |
| --- | --- | --- |
| Minor-series entry, beta-0.17.0 | `ccc16af9` | 2026-10-15 |
| Latest committed patch at this audit, beta-0.17.9 | `2275b6af` | 2026-10-25 |
| Previous minor entry, beta-0.16.0 | `dff1bdbc` | 2026-10-07 |

Git remains authoritative after this dated snapshot. The
[beta-0.16 record](nuis-beta-0.16.0-snapshot.md) retains its original results;
they are not renamed as beta-0.17 acceptance. The `.0` filenames identify the
minor series, not a requirement that every linked proof belongs to its first patch.

## Current Direction

The governing line is [ns-nova application-led development](nuis-beta-0.11-application-led-mainline.md):
grow the Nuis application, repair general foundation gaps, measure performance
and transfer independently testable compiler responsibilities incrementally.

The goal is `standard-library/ns-nova/interactive-image-workflow`. Its selected
prerequisite is `standard-library/ns-nova/persistent-application-session`,
active/99 at this audit. Documentation maintenance does not increase that score.
Use the [mainline map](../current-mainline-map.md) and
[live tensor](../reference/nuis-development-tensor.md) to select work.

## Committed Native CPU Scope

The committed tree through `2275b6af` contains the bounded root/child carry-epoch
and typed-field proofs. The following are proof families, not a claim that
one patch newly ran every suite or that all source combinations lower.

| Family | Evidence |
| --- | --- |
| Selected loop returns and parent-local exits | [Mixed exits](../reference/nuis-native-stateful-loop-return-parent-mixed-v1.md). |
| Independent effect order and certified storage sharing | [Effect prefix](../reference/nuis-native-return-effect-prefix-v1.md), [storage reuse](../reference/nuis-native-return-effect-storage-v1.md). |
| Ordered complete record construction and owned origins | [Field payloads](../reference/nuis-native-return-effect-storage-field-payload-v1.md), [two-arm origins](../reference/nuis-native-origin-record-join-v1.md). |
| Explicit pure-i64 printing | [Computed print policy](../reference/nuis-native-computed-iteration-print-v1.md), distinct from legacy literal grants. |
| Root carry reads before update | [Root epoch](../reference/nuis-native-carry-epoch-print-v1.md), scoped to one print rather than general readiness. |
| Child entry and typed owned fields | [Child epochs](../reference/nuis-native-child-epoch-print-v1.md), [full-width fields](../reference/nuis-native-f64-field-epoch-print-v1.md); not general compound arguments. |

The [proof index](../reference/nuis-native-session-proof-index.md) retains the
complete source/native/source-free chain. Each receipt owns its Git baseline,
budgets and actual results. Do not add totals from different checkpoints.

## Follow-Up Provenance

The [child-epoch proof](../reference/nuis-native-child-epoch-print-v1.md)
was a worktree follow-up to `2db5c82c` and is now committed in `2275b6af`.
It derives fresh child print permissions from real ready entry values, without
inheriting unavailable ancestor grants. Its receipt records its own local M2
CPU, reference/native and source-free delivery validation.

The [field-epoch proof](../reference/nuis-native-field-epoch-print-v1.md) is also
committed there. It extends one-print reads to exact original i64 paths in
owned nested record carries, without granting whole-record readiness.

The [typed-bool argument proof](../reference/nuis-native-bool-field-epoch-print-v1.md)
is committed there, limited to exact bool parameters of completed
pure-i64 print helpers. It does not widen ordinary bool or record availability.

The [typed-i32 proof](../reference/nuis-native-i32-field-epoch-print-v1.md) adds
the shared exact-parameter query and signed State-word witnesses as a further
committed follow-up. Ordinary casts, record readiness and effects stay separate.

The [typed-f32 proof](../reference/nuis-native-f32-field-epoch-print-v1.md) is a
committed follow-up, pinning raw callee/State IEEE words independently of
finite decimal CLI delivery. It does not authorize f64 or general float reads.

The [typed-f64 proof](../reference/nuis-native-f64-field-epoch-print-v1.md) is a
committed follow-up, preserving all 64 callee/State bits and explicit
budget rejection. Compound pre-update arguments remain a separate proof.

## Uncommitted Follow-Ups

The [typed unary-float proof](../reference/nuis-native-unary-float-field-epoch-print-v1.md)
is not part of the committed patch `2275b6af`; it is the current worktree follow-up.
It admits one exact sign
operation over a certified f32/f64 field argument without general readiness.

The [nested unary-float proof](../reference/nuis-native-nested-unary-float-field-epoch-print-v1.md)
is a further worktree follow-up, composing exact sign operations under shared
depth/work limits while preserving original owned-field certificates.

The [binary float proof](../reference/nuis-native-binary-float-field-epoch-print-v1.md)
is a further worktree follow-up, adding same-kind field/ready-leaf arithmetic and
aligning native zero-divide failure with the reference CPU contract.

The [pure float-call proof](../reference/nuis-native-float-call-field-epoch-print-v1.md)
is a further worktree follow-up, composing completed pure float helpers with
exact argument reads and source-ordered failures under shared bounds.

The [computed call-argument proof](../reference/nuis-native-computed-call-argument-print-v1.md)
is a further worktree follow-up, adding typed integer/bool operands and total
logical RHSs without changing ordinary prefix or field-read authority.

The [lazy call-argument proof](../reference/nuis-native-lazy-call-argument-print-v1.md)
is a further worktree follow-up, sharing guarded logical lowering with print
values and projecting exact scalar RHS captures, including under sign wrappers.

The [helper call-argument proof](../reference/nuis-native-helper-call-argument-print-v1.md)
is a further worktree follow-up, adding completed bool/i64 helper operands only
inside lazy RHS scope with exact arguments and transitive purity checks.

A later Git commit can include this work without changing the meaning of this
dated snapshot. Inspect current history and diff rather than treating links as
proof of release membership:

```sh
git log -1 --format='%h %s %cs'
git status --short
```

## Current Boundaries

The embedded-YIR image application, bounded native CPU session and Nsld private
image are distinct execution routes. One route's acceptance does not certify the
others or a self-contained heterogeneous application.

The next selected CPU gate is logical bool arguments of auxiliary helpers inside
lazy RHS scope. Completed pure bool/i64 operands are covered with exact reads and
transitive dependencies; ordinary call roots, whole records and eager checked
work remain closed.
General resource/I/O/async capture, outer
writes, stable ABI and full GPU/Linux/PE parity remain separate.

Five bounded migration-preparation gates are closed. Actual compiler ownership
transfer and the candidate's native-object boundary remain distinct; follow
[self-hosting readiness](../reference/nuis-self-hosting-readiness.md).
The approximate stage2-equivalent target remains `gamma-0.5.*` through
`gamma-0.10.*`, not a release deadline.

## Validation And Maintenance

Use the [operating checklist](nuis-beta-0.17.0-release-checklist.md) for current
commands and profile-specific evidence. The [repository audit](../repo-cleanup-candidates.md)
records documentation/navigation maintenance separately from runtime acceptance.
Historical packaging-refactor counts remain with that audit.

This snapshot does not imply a new test run, Git commit/tag/push, physical
Linux/GPU/Windows execution, performance advantage, global memory-safety proof,
completed self-hosting or a stable public ABI.
