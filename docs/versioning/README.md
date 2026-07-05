# Versioning Notes

This directory is the lightweight anchor set for minor-line snapshots,
workflow/history checkpoints, and policy notes.

## Read This First

### Current Line

If you want the current line instead of historical backfill, start with:

* [nuis-alpha-0.8-mainline-entry.md](nuis-alpha-0.8-mainline-entry.md)
* [nuis-alpha-0.7-mainline-entry.md](nuis-alpha-0.7-mainline-entry.md)
* [nuis-alpha-0.6-mainline-entry.md](nuis-alpha-0.6-mainline-entry.md)
* [nuis-alpha-0.4-system-inventory.md](nuis-alpha-0.4-system-inventory.md)
* [nuis-alpha-0.4-mainline-hardening-plan.md](nuis-alpha-0.4-mainline-hardening-plan.md)
* [nuis-alpha-0.4-doc-sync-inventory.md](nuis-alpha-0.4-doc-sync-inventory.md)
* [nuis-alpha-0.1-mainline-status.md](nuis-alpha-0.1-mainline-status.md)

Use this first when the question is:

* what the current `alpha-0.8.*` line adds on top of the hardening baseline
* why binary linking convergence is now the first-read Nsld/toolchain pressure
* what the predecessor `alpha-0.7.*` line established for std-backed tooling smoke
* what the predecessor `alpha-0.6.*` line established for Nsld
* what exists and what is still soft in the `alpha-0.4.*` hardening baseline
* what the current mainline should optimize before `alpha-0.10.0`
* which documentation routes and wording are current after the alpha-0.8
  entry refresh
* what the `alpha-0.1.*` mainline established before this hardening pass
* what should count as present-tense repo truth
* which older files should now be treated as predecessor anchors

### Immediate Predecessor Alpha Transition

If you want the line that handed off into the current one, read:

* [nuis-alpha-0.0.1-preflight-report.md](nuis-alpha-0.0.1-preflight-report.md)
* [nuis-alpha-0.0.1-closeout-board.md](nuis-alpha-0.0.1-closeout-board.md)
* [nuis-alpha-0.0.1-closeout-checklist.md](nuis-alpha-0.0.1-closeout-checklist.md)

Use this set when the question is:

* how the closeout-era `0.20.* -> alpha-0.0.1` transition led into the current
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
* which branch-local runtime-lowering rewrites are already test-backed
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

* start at `alpha-0.8.*` mainline entry first
* use `alpha-0.7.*` for the predecessor std/tooling smoke entry
* use `alpha-0.6.*` for the predecessor Nsld/frontdoor entry
* use `alpha-0.4.*` inventory and hardening as the current baseline context
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
