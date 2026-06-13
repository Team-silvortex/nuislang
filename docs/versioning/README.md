# Versioning Notes

This directory is the lightweight anchor set for phase snapshots and later
versioning policy documents.

## Current Snapshot

Start with:

* [nuis-0.18.0-snapshot.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-snapshot.md)
* [nuis-0.18.0-release-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-release-checklist.md)
* [nuis-0.18.0-mainline-goals.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-mainline-goals.md)
* [nuis-0.18.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-compile-workflow.md)
* [nuis-0.18.0-example-routing-snapshot.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-example-routing-snapshot.md)
* [nuis-0.18.0-mainline-regression-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-mainline-regression-matrix.md)
* [nuis-0.18.0-address-pointer-mainline.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-address-pointer-mainline.md)
* [nuis-0.13.0-snapshot.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.13.0-snapshot.md)
* [nuis-0.13.0-release-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.13.0-release-checklist.md)
* [nuis-minor-snapshot-rule.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-minor-snapshot-rule.md)
* [nuis-0.16.0-snapshot.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-snapshot.md)
* [nuis-0.16.0-release-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-release-checklist.md)
* [nuis-0.16.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-compile-workflow.md)
* [nuis-0.16.0-binary-compile-maturity.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-binary-compile-maturity.md)
* [nuis-0.16.0-generic-constraint-coverage.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-generic-constraint-coverage.md)
* [nuis-0.16.0-generic-constraint-gaps.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-generic-constraint-gaps.md)
* [nuis-0.16.0-generic-surface-audit.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-generic-surface-audit.md)
* [nuis-0.17.0-snapshot.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-snapshot.md)
* [nuis-0.17.0-release-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-release-checklist.md)
* [nuis-0.17.0-mainline-goals.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-mainline-goals.md)
* [nuis-0.17.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-compile-workflow.md)
* [nuis-0.17.0-lowering-capability-map.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-lowering-capability-map.md)
* [nuis-0.17.0-network-http-readiness-checklist.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-network-http-readiness-checklist.md)
* [nuis-0.17.0-mainline-regression-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-mainline-regression-matrix.md)
* [nuis-0.17.0-self-hosted-mainline-gate-plan.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-self-hosted-mainline-gate-plan.md)
* [nuis-0.17.0-generic-completion-plan.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.17.0-generic-completion-plan.md)
Those files are the shortest current “what is real enough to stand on” anchors
for the active version lines.

The `0.16.0` snapshot is the first minor-line history anchor under the new
minor-version recording rule.

The `0.16.0` workflow file is the current operational target for a cleaner and
more teachable `nuis` compile route.

The `0.16.0` binary-compile maturity file is the current target for deciding
when that route is strong enough to call a genuinely mature binary compile
story.

The `0.16.0` generic-constraint coverage file is the current short map for
which generic method-bound routes are already trustworthy and which are still
intentionally narrower.

The `0.16.0` generic-constraint gaps file is the small working checklist for
what still looks worth tightening after that coverage map.

The `0.16.0` generic-surface audit file is the compiler-facing closure matrix
for which generic constructor / specialization / pattern / method-bound
crossovers already have test-backed coverage.

The `0.17.0` snapshot is the next minor-line history anchor and marks the move
from compile-workflow cleanup toward broader completion/integration work.

The `0.17.0` release checklist is the operational reminder that the next line
should prove cross-layer integration, not only accumulate more local features.

The `0.17.0` mainline goals file is the short working map for the current
active push.

The `0.17.0` compile workflow file is the current compiler-facing explanation
of how the frontend/generic/higher-order/async pipeline actually composes.

The `0.17.0` lowering capability map is the compact answer to which lowering
routes already count as real, test-backed compiler behavior.

The `0.17.0` network/http readiness checklist is the narrow release-facing
answer to what the current repository can honestly say about that specific
story.

The `0.17.0` mainline regression matrix is the small execution map for which
test families now defend that story.

The `0.17.0` self-hosted mainline gate plan records the current answer to
whether that gate can already be written in `nuis` itself, and what must be
true before the answer becomes yes.

The `0.17.0` generic completion plan is the first detailed execution map under
that mainline and should be the default entry point for generic-track work.

The `0.18.0` snapshot is the next minor-line history anchor and marks the move
from broader subsystem integration toward a clearer single mainline centered on
control flow, async/task + memory composition, and project-backed compile
truth.

The `0.18.0` release checklist is the operational reminder that the new line
must keep those stories aligned rather than only adding more features.

The `0.18.0` compile workflow file is the current compiler-facing explanation
of how frontend ordering, project-aware lowering, loop-family truth, and
address/pointer validation now fit together.

The `0.18.0` example-routing snapshot is the short history anchor for when the
example tree started reading as one explicit frontdoor/companion/probe/legacy
system instead of one flat inventory.

The `0.18.0` mainline regression matrix is the compact answer to which checked
test families currently defend that story.

The `0.18.0` address/pointer mainline file is the short truth anchor for the
current `ref`-based address system.

## Expected Broader Scope Later

This directory is also the natural home for later policy documents around:

* language/toolchain surface versioning
* `YIR` format/version compatibility policy
* `nustar` package format/version compatibility policy
* ABI and loader-contract evolution rules

## Current Reading Rule

Phase snapshots here are anchors, not replacements for implementation truth.

For exact current behavior, still prefer:

* [../reference/README.md](/Users/Shared/chroot/dev/nuislang/docs/reference/README.md)
* [../reference/yir-tools-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-tools-reference.md)
* current checked-in parsing/lowering/verification code
