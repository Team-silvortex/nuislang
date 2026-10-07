# Versioning Notes

This directory is the lightweight anchor set for minor-line snapshots,
workflow/history checkpoints, and policy notes.

Historical command blocks are records of their original checkout, not current
entrypoints. The pre-alpha `check-0.17-*`, `check-0.18-*` and `check-0.19-*`
scripts were retired in the 2026-09-24 cleanup. Use the current-line checklist
below and the [repository cleanup policy](../repo-cleanup-candidates.md) instead.

## Read This First

### Current Line

For the current `beta-0.16.*` line instead of historical backfill, start with:

* [nuis-beta-0.16.0-snapshot.md](nuis-beta-0.16.0-snapshot.md)
* [nuis-beta-0.16.0-release-checklist.md](nuis-beta-0.16.0-release-checklist.md)

The current committed minor baseline is `dff1bdbc` (`beta-0.16.0`, 2026-10-07).
The [previous patch](nuis-beta-0.15.9-patch.md) retains `585eb409`, and the
[previous minor snapshot](nuis-beta-0.15.0-snapshot.md) retains `05951bef`
(`beta-0.15.0`, 2026-09-24).
Git remains authoritative for exact source revisions. Patch evidence does not
certify every suite listed in the checklist, and historical results remain
separate. Cargo package versions and protocol versions remain independent.

The earlier proofs include [leading return prints](../reference/nuis-native-return-print-prefixes-v1.md),
[computed print arguments](../reference/nuis-native-computed-return-prints-v1.md),
[interleaved scalar aliases](../reference/nuis-native-return-print-aliases-v1.md)
and [staged initializers](../reference/nuis-native-staged-return-effects-v1.md),
followed by [logical initializers](../reference/nuis-native-logical-staged-initializers-v1.md).
Their acceptance receipts remain separate from the
[preceding patch](nuis-beta-0.15.8-patch.md). The
[continuing effect-region proof](../reference/nuis-native-continuing-effect-regions-v1.md)
and [internal-exit follow-up](../reference/nuis-native-effect-region-exits-v1.md),
including its [exit-only continuation extension](../reference/nuis-native-exit-only-regions-v1.md)
and [paired scalar result joins](../reference/nuis-native-effect-result-joins-v1.md)
with their [partial-exit extension](../reference/nuis-native-partial-effect-result-joins-v1.md)
and [one-sided result extension](../reference/nuis-native-one-sided-effect-result-joins-v1.md),
were developed after `585eb409` and are included in `dff1bdbc`; their separate
receipts retain their original scope and dates.

The [join-capture optimization](../reference/nuis-native-effect-join-captures-v1.md)
is worktree development after `dff1bdbc` (`beta-0.16.0`), with its own bounded
CPU receipt, not a new release or measured-performance claim.
The [equal-atom follow-up](../reference/nuis-native-equal-effect-result-joins-v1.md)
extends that worktree proof with its own acceptance receipt.

The [beta-0.15.7 patch](nuis-beta-0.15.7-patch.md) retains the typed-literal,
fresh-publication and `active/86` checkpoint; subsequent worktree proofs have
their own receipts and must not be attributed to that historical record.

The [beta-0.15.6 patch](nuis-beta-0.15.6-patch.md) retains the typed read-only
scoped input, preheader join and parent-entry snapshot checkpoint.
The [beta-0.15.5 patch](nuis-beta-0.15.5-patch.md) retains the nested typed carry
and return-invariant checkpoint.
The [beta-0.15.4 patch](nuis-beta-0.15.4-patch.md) retains the guarded branch-input
and flat typed scoped-carry checkpoint.
The [beta-0.15.3 patch](nuis-beta-0.15.3-patch.md) retains the unread-input,
shared-codec and scoped flat-i64 record-input checkpoint.
The [beta-0.15.2 patch](nuis-beta-0.15.2-patch.md) retains the record-join,
independent-seed and partial-input checkpoint. The
[beta-0.15.1 patch](nuis-beta-0.15.1-patch.md) retains the earlier capture,
snapshot and invariant-input checkpoint. Their evidence remains historical.

The previous native-session checkpoint remains historical:

* [nuis-beta-0.14.0-snapshot.md](nuis-beta-0.14.0-snapshot.md)
* [nuis-beta-0.14.0-release-checklist.md](nuis-beta-0.14.0-release-checklist.md)

Those files still record `1fcfd65` (`beta-0.14.1`, 2026-09-13); their `.0`
filenames identify the minor series, not the exact source patch.
The earlier application-session checkpoint also remains historical:

* [nuis-beta-0.12.0-snapshot.md](nuis-beta-0.12.0-snapshot.md)
* [nuis-beta-0.12.0-release-checklist.md](nuis-beta-0.12.0-release-checklist.md)

Those files still record `505c820c` (`beta-0.12.2`). The governing
application-led agreement and migration-entry history remain:

* [nuis-beta-0.11-application-led-mainline.md](nuis-beta-0.11-application-led-mainline.md)
* [nuis-beta-0.10.0-self-hosting-entry.md](nuis-beta-0.10.0-self-hosting-entry.md)

The previous curated foundation entry is:

* [nuis-beta-0.6.0-mainline-entry.md](nuis-beta-0.6.0-mainline-entry.md)

The previous curated beta anchors are:

* [nuis-beta-0.3.0-mainline-entry.md](nuis-beta-0.3.0-mainline-entry.md)
* [nuis-beta-0.1.0-mainline-entry.md](nuis-beta-0.1.0-mainline-entry.md)
* [nuis-beta-0.1.0-doc-sync-inventory.md](nuis-beta-0.1.0-doc-sync-inventory.md)
* [nuis-beta-0.0.1-mainline-entry.md](nuis-beta-0.0.1-mainline-entry.md)
* [nuis-beta-0.0.1-doc-sync-inventory.md](nuis-beta-0.0.1-doc-sync-inventory.md)

The `beta-0.2.*`, `beta-0.4.*`, `beta-0.5.*`, and `beta-0.13.*` patch sequences
remain available in Git history. No retrospective phase documents are invented
for them after the fact.

Earlier alpha anchors are:

* [nuis-alpha-0.20-mainline-entry.md](nuis-alpha-0.20-mainline-entry.md)
* [nuis-alpha-0.17-mainline-entry.md](nuis-alpha-0.17-mainline-entry.md)
* [nuis-alpha-0.16-mainline-entry.md](nuis-alpha-0.16-mainline-entry.md)
* [nuis-alpha-0.13-mainline-entry.md](nuis-alpha-0.13-mainline-entry.md)
* [nuis-alpha-0.10-mainline-entry.md](nuis-alpha-0.10-mainline-entry.md)
* [nuis-alpha-0.8-mainline-entry.md](nuis-alpha-0.8-mainline-entry.md)
* [nuis-alpha-0.8-doc-sync-inventory.md](nuis-alpha-0.8-doc-sync-inventory.md)
* [nuis-alpha-0.7-mainline-entry.md](nuis-alpha-0.7-mainline-entry.md)
* [nuis-alpha-0.6-mainline-entry.md](nuis-alpha-0.6-mainline-entry.md)
* [nuis-alpha-0.4-system-inventory.md](nuis-alpha-0.4-system-inventory.md)
* [nuis-alpha-0.4-mainline-hardening-plan.md](nuis-alpha-0.4-mainline-hardening-plan.md)
* [nuis-alpha-0.4-doc-sync-inventory.md](nuis-alpha-0.4-doc-sync-inventory.md)
* [nuis-alpha-0.1-mainline-status.md](nuis-alpha-0.1-mainline-status.md)

Use this section when the question is:

* what beta-0.15 records for native value returns, private capture aliases and
  snapshots, source-free artifact restoration, and bounded repository cleanup
* what beta-0.14 records for bounded native scalar sessions, conditional loop
  carries, artifact restoration, and the distinction between build and execution evidence
* what beta-0.12 established for persistent windows, independent cancellation
  tickets and the remaining AppKit/provider-retirement boundary
* why ns-nova application work drives general foundation fixes and measured
  compiler-module migration without coupling independent Galaxies/Nustars
* why `beta-0.10.*` activates formal stage0-to-stage1 migration without
  claiming completed self-hosting or final replacement readiness
* what `beta-0.0.1` established at the alpha-to-beta transition
* what `alpha-0.20.*` added by closing alpha around a beta-prep
  compiler/std/Nustar/Nsld/Nsdb/tensor foundation
* what the predecessor `alpha-0.17.*` line added by making registered
  heterogeneous worker execution the active integration gate
* what the predecessor `alpha-0.16.*` line added by making development tensor
  evidence the main steering surface
* what the predecessor `alpha-0.13.*` line added by widening tensor-guided
  hardening
* what the predecessor `alpha-0.10.*` line added on top of binary-linking
  convergence
* why executable-artifact closure is now the first-read Nsld/toolchain pressure
* which native executable back-half gaps are currently the weakest self-hosting
  prerequisites
* what the predecessor `alpha-0.8.*` line established for binary-linking
  convergence
* what the predecessor `alpha-0.7.*` line established for std-backed tooling smoke
* what the predecessor `alpha-0.6.*` line established for Nsld
* what exists and what is still soft in the `alpha-0.4.*` hardening baseline
* what the `beta-0.10.*` migration-entry phase established
* which documentation routes and wording are current after the beta entry
  refresh
* which broad README surfaces were refreshed for the current beta line
* what the `alpha-0.1.*` mainline established before this hardening pass
* what should count as present-tense repo truth
* which older files should now be treated as predecessor anchors

### Earlier Alpha Transition

If you want the first transition into alpha, read:

* [nuis-alpha-0.0.1-preflight-report.md](nuis-alpha-0.0.1-preflight-report.md)
* [nuis-alpha-0.0.1-closeout-board.md](nuis-alpha-0.0.1-closeout-board.md)
* [nuis-alpha-0.0.1-closeout-checklist.md](nuis-alpha-0.0.1-closeout-checklist.md)

Use this set when the question is:

* how the closeout-era `0.20.* -> alpha-0.0.1` transition led into the first
  `alpha-0.1.*` line
* which closeout-era lanes were still active versus already boundary-shaped

### Pre-Alpha Mainline Anchors

If you want the strongest pre-alpha mainline anchors that still explain the
current repository shape, read:

* [nuis-0.20.0-abi-compile-vocabulary.md](nuis-0.20.0-abi-compile-vocabulary.md)
* [nuis-0.20.0-frontend-cli-boundaries.md](nuis-0.20.0-frontend-cli-boundaries.md)
* [nuis-0.20.0-branch-runtime-lowering-matrix.md](nuis-0.20.0-branch-runtime-lowering-matrix.md)
* [nuis-0.20.0-generic-validation-regression-matrix.md](nuis-0.20.0-generic-validation-regression-matrix.md)
* [nuis-0.20.0-receiver-generic-regression-matrix.md](nuis-0.20.0-receiver-generic-regression-matrix.md)
* [nuis-0.20.0-std-refactor-frontdoor.md](nuis-0.20.0-std-refactor-frontdoor.md)
* [nuis-0.20.0-compile-gap-checklist.md](nuis-0.20.0-compile-gap-checklist.md)
* [nuis-0.20.x-to-alpha-bootstrap-roadmap.md](nuis-0.20.x-to-alpha-bootstrap-roadmap.md)
* [nuis-0.19.0-snapshot.md](nuis-0.19.0-snapshot.md)
* [nuis-0.19.0-compile-workflow.md](nuis-0.19.0-compile-workflow.md)
* [nuis-0.19.0-mainline-goals.md](nuis-0.19.0-mainline-goals.md)
* [nuis-0.19.0-release-checklist.md](nuis-0.19.0-release-checklist.md)
* [nuis-0.19.0-mainline-regression-matrix.md](nuis-0.19.0-mainline-regression-matrix.md)
* [nuis-0.19.0-workflow-capability-matrix.md](nuis-0.19.0-workflow-capability-matrix.md)
* [nuis-0.19.0-project-capability-matrix.md](nuis-0.19.0-project-capability-matrix.md)
* [nuis-0.19.0-frontend-capability-matrix.md](nuis-0.19.0-frontend-capability-matrix.md)
* [nuis-0.19.0-address-pointer-mainline.md](nuis-0.19.0-address-pointer-mainline.md)

Use this set when the question is:

* what the current compile workflow grew out of
* which ABI terms are now canonical
* where frontend/NIR truth currently outruns the deeper CLI/source-compile route
* which branch-local runtime-lowering rewrites are already regression-backed
* which generic validation surfaces are already regression-backed across
  explicit calls, struct literals, `if` / `match`, and lambda bodies
* which receiver explicit-generic method-call surfaces are already
  regression-backed across helper, async, task, result, and control-flow
  wrappers
* which `std` lanes should be normalized first during the `0.20.*` refactor
* which specific `0.20.*` compile-chain gaps were being actively closed before
  alpha

## Earlier Predecessor Minor Line

When you need the immediate predecessor rather than the current line, use:

* [nuis-0.18.0-snapshot.md](nuis-0.18.0-snapshot.md)
* [nuis-0.18.0-mainline-goals.md](nuis-0.18.0-mainline-goals.md)
* [nuis-0.18.0-compile-workflow.md](nuis-0.18.0-compile-workflow.md)
* [nuis-0.18.0-mainline-regression-matrix.md](nuis-0.18.0-mainline-regression-matrix.md)
* [nuis-0.18.0-release-checklist.md](nuis-0.18.0-release-checklist.md)
* [nuis-0.18.0-address-pointer-mainline.md](nuis-0.18.0-address-pointer-mainline.md)
* [nuis-0.18.0-example-routing-snapshot.md](nuis-0.18.0-example-routing-snapshot.md)

Use this line when the question is historical comparison for:

* control-flow completion
* address/pointer transition
* example-tree reshaping
* the first clearer single-mainline compile route before `0.19.*` cleanup

## Older Historical Anchors

These are still worth keeping, but they should not be treated as the default
entry route anymore:

* `0.17.*`
  [nuis-0.17.0-snapshot.md](nuis-0.17.0-snapshot.md),
  [nuis-0.17.0-mainline-goals.md](nuis-0.17.0-mainline-goals.md),
  [nuis-0.17.0-compile-workflow.md](nuis-0.17.0-compile-workflow.md),
  [nuis-0.17.0-mainline-regression-matrix.md](nuis-0.17.0-mainline-regression-matrix.md),
  [nuis-0.17.0-release-checklist.md](nuis-0.17.0-release-checklist.md)
* `0.16.*`
  [nuis-0.16.0-snapshot.md](nuis-0.16.0-snapshot.md),
  [nuis-0.16.0-compile-workflow.md](nuis-0.16.0-compile-workflow.md),
  [nuis-0.16.0-release-checklist.md](nuis-0.16.0-release-checklist.md),
  [nuis-0.16.0-binary-compile-maturity.md](nuis-0.16.0-binary-compile-maturity.md),
  [nuis-0.16.0-generic-constraint-coverage.md](nuis-0.16.0-generic-constraint-coverage.md),
  [nuis-0.16.0-generic-constraint-gaps.md](nuis-0.16.0-generic-constraint-gaps.md),
  [nuis-0.16.0-generic-surface-audit.md](nuis-0.16.0-generic-surface-audit.md)
* first minor-history anchors:
  [nuis-0.13.0-snapshot.md](nuis-0.13.0-snapshot.md),
  [nuis-0.13.0-release-checklist.md](nuis-0.13.0-release-checklist.md)
## Policy Anchor

Use this when the question is:

* how minor-line history files should be added and routed
* how new current-line anchors should demote older ones without deleting them

Read:

* [nuis-minor-snapshot-rule.md](nuis-minor-snapshot-rule.md)

Practical rule:

* start at the current `beta-0.16.*` effect-region/session checkpoint and validation checklist
* use `beta-0.15.*` for the preceding native-value/session checkpoint
* use `beta-0.14.*` for the previous native-session checkpoint
* use `beta-0.12.*` for the previous application-session snapshot
* use `beta-0.11.*` for the governing application-led direction
* use `beta-0.10.*` for the historical staged-migration entry
* use `beta-0.6.*` for the previous curated foundation snapshot
* use `beta-0.3.*` for the previous curated linker/runtime snapshot
* use `beta-0.1.0` for the earlier curated beta foundation snapshot
* use `beta-0.0.1` for the recorded first-beta predecessor
* use `alpha-0.20.*` for the direct alpha closeout predecessor
* use `alpha-0.17.*` for the direct registered-worker predecessor
* use `alpha-0.16.*` for the tensor-guided closure predecessor
* use `alpha-0.13.*` for the broader tensor-guided hardening predecessor
* use `alpha-0.7.*` for the predecessor std/tooling smoke entry
* use `alpha-0.6.*` for the predecessor Nsld/frontdoor entry
* use `alpha-0.4.*` inventory and hardening as historical baseline context
* use `alpha-0.1.*` for the first post-closeout alpha consolidation line
* then use `0.20.*` and `0.19.*` only when you are intentionally
  reconstructing the line that led here
* drop to `0.18.*` when you need the immediate predecessor line
* only use `0.17.*` and `0.16.*` as historical/debugging context

## Current Reading Rule

Versioning files are anchors, not replacements for implementation truth.

For exact current behavior, still prefer:

* [../reference/README.md](../../docs/reference/README.md)
* [../reference/yir-tools-reference.md](../../docs/reference/yir-tools-reference.md)
* current checked-in parsing, lowering, verification, and CLI code

## Expected Broader Scope Later

This directory is also the natural home for later policy documents around:

* language/toolchain surface versioning
* `YIR` format/version compatibility policy
* `nustar` package format/version compatibility policy
* ABI and loader-contract evolution rules
