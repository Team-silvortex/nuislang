# Documentation Index

This directory separates present-tense implementation truth from phase history
and longer-range design. If two documents disagree, prefer checked-in behavior,
tests, the development tensor, and `docs/reference/` in that order.

## Read This First

Use this short route for the current `beta-0.7.*` repository:

1. [Current mainline map](current-mainline-map.md)
2. [Beta 0.6 mainline entry](versioning/nuis-beta-0.6.0-mainline-entry.md)
3. [Development tensor](reference/nuis-development-tensor.md)
4. [Self-hosting readiness](reference/nuis-self-hosting-readiness.md)
5. [Native artifact workflow](reference/nuis-native-artifact-workflow.md)
6. [Nsld linker frontdoor](reference/nsld-linker-frontdoor.md)
7. [Binary assembly gap map](reference/nsld-binary-assembly-gap-map.md)
8. [Nustar multi-backend artifact contract](reference/nustar-multi-backend-artifact-contract.md)
9. [Repository layout](repo-layout.md)

The [versioning index](versioning/README.md) routes older beta, alpha, and
pre-alpha snapshots. Those files explain how the current shape emerged; they
are not the default source for current capability claims.

## Truth Layers

| Layer | Purpose |
| --- | --- |
| [`reference/`](reference) | Current implementation-facing contracts, workflows, and boundaries |
| [`versioning/`](versioning) | Minor-line phase anchors, transition records, and roadmap policy |
| [`grammar/`](grammar) | Parser and source grammar notes |
| [`yir-spec/`](yir-spec) | YIR design and protocol direction |
| [`glm-spec/`](glm-spec) | GLM and heterogeneous flow-graph design direction |
| [`fabric-spec/`](fabric-spec) | Data-fabric design material, including historical drafts |
| [`historical/`](historical) | Explicitly archived whitepapers and predecessor material |

Current reference material outranks broader design sketches when they differ.
In particular, `fabric-spec/DFIR.md` is historical draft material rather than a
current verifier contract.

## By Goal

For the compiler and project frontdoor:

* [Self-hosting readiness](reference/nuis-self-hosting-readiness.md)
* [Bootstrap language subset](reference/nuis-bootstrap-language-subset.md)
* [Compiler data model](reference/nuis-compiler-data-model.md)
* [Compiler stage handoff](reference/nuis-compiler-stage-handoff.md)
* [Compiler component build](reference/nuis-compiler-component-build.md)
* [Compiler candidate execution](reference/nuis-compiler-candidate-execution.md)
* [Compiler candidate production](reference/nuis-compiler-candidate-production.md)
* [Compiler component differential gate](reference/nuis-compiler-component-differential.md)
* [Compiler component reproducibility](reference/nuis-compiler-component-reproducibility.md)
* [Nuis frontdoor surface](reference/nuis-frontdoor-surface-reference.md)
* [Native artifact workflow](reference/nuis-native-artifact-workflow.md)
* [YIR tools](reference/yir-tools-reference.md)
* [Control-flow lowering](reference/control-flow-lowering-contract.md)
* [Generic diagnostic ownership](reference/generic-diagnostic-ownership-contract.md)

For native binaries, runtime, and debugging:

* [Nuis binary format protocol](reference/nuis-binary-format-protocol.md)
* [Nsld linker frontdoor](reference/nsld-linker-frontdoor.md)
* [Nsld binary assembly gap map](reference/nsld-binary-assembly-gap-map.md)
* [Executable finalizer registry](reference/nsld-executable-finalizer-registry.md)
* [Nsdb YIR debugger frontdoor](reference/nsdb-yir-debugger-frontdoor.md)
* [Toolchain capability boundary](reference/toolchain-galaxy-core-boundary.md)

For heterogeneous domains and host compatibility:

* [Nustar capability split](reference/nustar-capability-split-boundary.md)
* [Multi-backend artifact contract](reference/nustar-multi-backend-artifact-contract.md)
* [CFFI/von-Neumann domain contract](reference/cffi-von-neumann-domain-contract.md)
* [FFI pointer safety boundary](reference/ffi-pointer-safety-boundary.md)
* [Linux CUDA provider bring-up](reference/linux-cuda-provider-bringup.md)
* [Provider completion trust registry](reference/provider-completion-trust-registry.md)

For std and official Galaxies:

* [Standard library index](../stdlib/README.md)
* [Std mainline layering](reference/std-mainline-layering-contract.md)
* [PixelMagic mainline](reference/pixelmagic-mainline-contract.md)
* [Shader/kernel project contract](reference/std-shader-kernel-project-contract.md)
* [Project examples](../examples/projects/README.md)
* [Source examples](../examples/ns/README.md)

For long-range architecture:

* [Heterogeneous OS roadmap](versioning/nuis-long-range-heterogeneous-os-roadmap.md)
* [GLM heterogeneous flow graph](glm-spec/glm-heterogeneous-flow-graph-positioning.md)
* [Vulpoya/YIR secondary review](glm-spec/vulpoya-yir-secondary-review-positioning.md)

## Maintenance Rule

New present-tense behavior belongs in `reference/` and should be reachable from
this index or a focused local README. A minor-line transition gets one concise
versioning anchor; older entries are demoted in the routers rather than
rewritten. Historical drafts stay available but should never be inserted back
into the shortest current reading path.
