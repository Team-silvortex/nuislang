# Versioning Notes

This directory is the lightweight anchor set for minor-line snapshots,
workflow/history checkpoints, and policy notes.

## Read This First

If you want the current line instead of historical backfill, start with:

* [nuis-0.20.0-abi-compile-vocabulary.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.20.0-abi-compile-vocabulary.md)
* [nuis-0.19.0-snapshot.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-snapshot.md)
* [nuis-0.19.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-compile-workflow.md)
* [nuis-0.19.0-mainline-goals.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-mainline-goals.md)
* [nuis-0.19.0-release-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-release-checklist.md)
* [nuis-0.19.0-mainline-regression-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-mainline-regression-matrix.md)
* [nuis-0.19.0-workflow-capability-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-workflow-capability-matrix.md)
* [nuis-0.19.0-project-capability-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-project-capability-matrix.md)
* [nuis-0.19.0-frontend-capability-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-frontend-capability-matrix.md)
* [nuis-0.19.0-address-pointer-mainline.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-address-pointer-mainline.md)

Use that set when the question is:

* what the current compile workflow is
* what the repo currently claims is ready
* which ABI terms are now canonical
* which project/frontend/workflow matrices should still be treated as live

## Previous Minor Line

When you need the immediate predecessor rather than the current line, use:

* [nuis-0.18.0-snapshot.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-snapshot.md)
* [nuis-0.18.0-mainline-goals.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-mainline-goals.md)
* [nuis-0.18.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-compile-workflow.md)
* [nuis-0.18.0-mainline-regression-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-mainline-regression-matrix.md)
* [nuis-0.18.0-release-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-release-checklist.md)
* [nuis-0.18.0-address-pointer-mainline.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-address-pointer-mainline.md)
* [nuis-0.18.0-example-routing-snapshot.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-example-routing-snapshot.md)

Use this line when the question is historical comparison for:

* control-flow completion
* address/pointer transition
* example-tree reshaping
* the first clearer single-mainline compile route before `0.19.*` cleanup

## Earlier Historical Anchors

These are still worth keeping, but they should not be treated as the default
entry route anymore:

* `0.17.*`
  [nuis-0.17.0-snapshot.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-snapshot.md),
  [nuis-0.17.0-mainline-goals.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-mainline-goals.md),
  [nuis-0.17.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-compile-workflow.md),
  [nuis-0.17.0-mainline-regression-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-mainline-regression-matrix.md),
  [nuis-0.17.0-release-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-release-checklist.md)
* `0.16.*`
  [nuis-0.16.0-snapshot.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-snapshot.md),
  [nuis-0.16.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-compile-workflow.md),
  [nuis-0.16.0-release-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-release-checklist.md),
  [nuis-0.16.0-binary-compile-maturity.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-binary-compile-maturity.md),
  [nuis-0.16.0-generic-constraint-coverage.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-generic-constraint-coverage.md),
  [nuis-0.16.0-generic-constraint-gaps.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-generic-constraint-gaps.md),
  [nuis-0.16.0-generic-surface-audit.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-generic-surface-audit.md)
* first minor-history anchors:
  [nuis-0.13.0-snapshot.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.13.0-snapshot.md),
  [nuis-0.13.0-release-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.13.0-release-checklist.md)
* policy:
  [nuis-minor-snapshot-rule.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-minor-snapshot-rule.md)

Practical rule:

* start at `0.19.*` plus the `0.20.0` ABI vocabulary unless you are
  intentionally reconstructing older assumptions
* drop to `0.18.*` when you need the immediate predecessor line
* only use `0.17.*` and `0.16.*` as historical/debugging context

## Current Reading Rule

Versioning files are anchors, not replacements for implementation truth.

For exact current behavior, still prefer:

* [../reference/README.md](/Users/Shared/chroot/dev/nuislang/docs/reference/README.md)
* [../reference/yir-tools-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-tools-reference.md)
* current checked-in parsing, lowering, verification, and CLI code

## Expected Broader Scope Later

This directory is also the natural home for later policy documents around:

* language/toolchain surface versioning
* `YIR` format/version compatibility policy
* `nustar` package format/version compatibility policy
* ABI and loader-contract evolution rules
