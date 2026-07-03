# Docs Index

This folder is the documentation entry point once you move past the top-level
`README`.

The docs are currently split into two broad categories:

* current reference / implementation-facing material
* longer-range design/spec material
* historical archived material

There is also a practical split inside the current tree:

* mainline docs
  these explain the repository paths you should rely on today
* experimental / design docs
  these explain probe directions, future contracts, and semantic sketches that
  are intentionally not fully locked yet

## Read This First

If you want to understand the repository as it exists today, start here:

* [current-mainline-map.md](current-mainline-map.md)
* [versioning/nuis-alpha-0.7-mainline-entry.md](versioning/nuis-alpha-0.7-mainline-entry.md)
* [versioning/nuis-alpha-0.6-mainline-entry.md](versioning/nuis-alpha-0.6-mainline-entry.md)
* [versioning/nuis-alpha-0.4-system-inventory.md](versioning/nuis-alpha-0.4-system-inventory.md)
* [versioning/nuis-alpha-0.4-mainline-hardening-plan.md](versioning/nuis-alpha-0.4-mainline-hardening-plan.md)
* [versioning/nuis-alpha-0.4-doc-sync-inventory.md](versioning/nuis-alpha-0.4-doc-sync-inventory.md)
* [versioning/nuis-long-range-heterogeneous-os-roadmap.md](versioning/nuis-long-range-heterogeneous-os-roadmap.md)
* [reference/nuis-frontdoor-surface-reference.md](reference/nuis-frontdoor-surface-reference.md)
* [reference/nuis-native-artifact-workflow.md](reference/nuis-native-artifact-workflow.md)
* [versioning/nuis-alpha-0.1-mainline-status.md](versioning/nuis-alpha-0.1-mainline-status.md)
* [reference/README.md](reference/README.md)
* [repo-layout.md](repo-layout.md)

If your immediate question is “what is the canonical compile route today?”,
start with the `alpha-0.7.*` mainline entry, then the std/frontdoor/native artifact
reference pair before drilling into deeper reference material.

If your immediate question is “which docs are current, which are predecessor
anchors, and what wording is safe?”, read the `alpha-0.7.*` mainline entry
first, then the `alpha-0.4.*` documentation sync baseline before editing broad
docs.

If your immediate question is “what is the current minor-line history anchor?”,
start with the `alpha-0.7.*` mainline entry, then use the `alpha-0.4.*`
inventory and hardening plan as baseline context. Use `alpha-0.1.*`, `0.20.*`,
and `0.19.*` only when you intentionally need predecessor lines.

If your immediate question is “what long-range hardware/OS shape should current
architecture avoid foreclosing?”, read the long-range heterogeneous OS roadmap.

If your immediate question is “which ABI words are now preferred before
current alpha work broadens the surface further?”, read the `0.20.0` ABI
compile vocabulary file as predecessor context.

Then branch by the kind of truth you want:

* current runnable project examples
  - [examples/projects/README.md](../examples/projects/README.md)
* current source-level `.ns` examples
  - [examples/ns/README.md](../examples/ns/README.md)
* current stdlib/source-asset maps
  - [stdlib/README.md](../stdlib/README.md)
  - [stdlib/std/README.md](../stdlib/std/README.md)
* cleanup policy / archiving candidates
  - [repo-cleanup-candidates.md](repo-cleanup-candidates.md)
  - [repo-file-line-policy.md](repo-file-line-policy.md)

## Grammar And Frontend Notes

Use these when you want parser/frontend context:

* [grammar/README.md](grammar/README.md)

## Design / Spec Direction

These folders describe broader architecture direction and are useful, but they
should be read together with the current reference docs above:

* [fabric-spec/README.md](fabric-spec/README.md)
* [glm-spec/README.md](glm-spec/README.md)
* [versioning/README.md](versioning/README.md)
* [yir-spec/README.md](yir-spec/README.md)
* [historical/README.md](historical/README.md)

Important current reading rule:

* if a broader design note and the current checked-in tool/reference behavior
  differ, prefer the current `reference/` documents plus the implementation
  itself
* `fabric-spec/DFIR.md` is historical draft material, not a current verifier
  contract

If your immediate question is "how do `GLM`, compiler-native `YIR` verification,
and the future `vulpoya` analyzer fit together?", start with:

* [glm-spec/glm-heterogeneous-flow-graph-positioning.md](glm-spec/glm-heterogeneous-flow-graph-positioning.md)
* [glm-spec/vulpoya-yir-secondary-review-positioning.md](glm-spec/vulpoya-yir-secondary-review-positioning.md)

## Historical Archive

These files are kept on purpose, but they are no longer part of the shortest
path for understanding the current repository:

* [historical/README.md](historical/README.md)
* [historical/nuislang-whitepaper-v0.44b.md](historical/nuislang-whitepaper-v0.44b.md)
