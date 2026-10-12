# Documentation Index

This directory separates present-tense implementation truth from phase history
and longer-range design. If two documents disagree, prefer checked-in behavior,
tests, the development tensor, and `docs/reference/` in that order.

## Read This First

Use this short route for the current `beta-0.16.*` repository, anchored at
[`beta-0.16.0`](versioning/nuis-beta-0.16.0-snapshot.md) (`dff1bdbc`, 2026-10-07).
The beta-0.15 snapshots and patches remain historical:

1. [Current mainline map](current-mainline-map.md)
2. [Beta-0.16 checkpoint](versioning/nuis-beta-0.16.0-snapshot.md)
3. [Mainline dependency selection](reference/nuis-development-tensor-mainline.md)
4. [Native scalar session bridge](reference/nuis-native-scalar-session-bridge-v1.md)
5. [Stateful window contract](reference/nuis-yir-window-session-v3.md)
6. [Cancellation and host retirement](reference/nuis-yir-application-cancellation-v1.md)
7. [Runnable image application](../examples/projects/domains/ns_nova_image_showcase/README.md)
8. [Focused validation checklist](versioning/nuis-beta-0.16.0-release-checklist.md)

The [native loop snapshot proof](reference/nuis-native-scalar-loop-snapshots-v1.md)
tracks post-loop boundaries and safe capture normalization without replacing the released patch history.
The [artifact publication contract](reference/nuis-artifact-file-publication-v1.md)
records the reproduced cache-overwrite `SIGKILL` class and fresh-file replacement.
The [typed selected-return handoff](reference/nuis-native-typed-effect-returns-v1.md)
separates exact scalar return evidence from native callback effect authority.
The [explicit literal-print policy](reference/nuis-native-literal-print-policy-v1.md)
checks bounded registered native effects without widening the default pure bridge;
the [explicit build policy](reference/nuis-native-literal-print-build-policy-v1.md)
now binds authorization through cache, standalone restoration and native launch.
The [guarded scalar-call follow-up](reference/nuis-native-effectful-scalar-selection-v1.md)
repairs source effect order and one-sided existing-scalar rebinding without
granting effects or evaluating inactive arguments. Bounded nested updates also
keep descendant predicates inside ancestor guards rather than hoisting them.
Bounded sequential scalar leaves also preserve private staging, repeated binding
versions and unused calls without exporting locals or relaxing effect grants.
The unified bounded ordered-region walk replaces the original prefix, suffix and
two-child production proofs while retaining their regression fixtures. Adjacent
and longer sibling sequences, including final-child regions, preserve current
merged values and private staging under shared work/capture/depth limits and
single-target publication, with native source-free execution and trap evidence.
Single-edge logical child predicates now preserve once-only owned-bool atom/
literal and bounded computed nonlogical left gates, including calls, comparisons
and checked arguments. Bounded direct logical children now form trees under one
shared region edge/work/depth budget, with linear helper growth and complete
guarded RHS subtrees. Fresh owned-bool let/const logical initializers now share
that proof within selected scalar regions and prefix/middle/suffix stages,
keeping private declarations, current versions and cross-stage constant seals.
Existing owned-bool let roots now update private stage bindings or the sole
published bool target, preserving read-before-write captures and constant seals.
Single-statement child leaves now reuse the stage proof for an existing
owned-bool target, with both-sided or retained outcomes. The
[top-level logical-arm follow-up](reference/nuis-native-effectful-scalar-selection-v1.md#top-level-logical-arms)
now also admits single-statement existing-bool updates through that shared plan,
including consecutive current merges and condition-only effects. Hidden logical
leaves and generalized resource/exit transport remain separate. The
[paired logical-return proof](reference/nuis-native-effectful-scalar-selection-v1.md#paired-logical-bool-returns)
now also covers two-sided single owned-bool returns, keeping shared condition/
value budgets, original exits and caller continuation. The
[private return-prefix follow-up](reference/nuis-native-effectful-scalar-selection-v1.md#private-logical-return-prefixes)
reuses selected staging with isolated fresh scopes, current versions, constant
seals and complete unused-call effects. The
[complete return-tree follow-up](reference/nuis-native-effectful-scalar-selection-v1.md#nested-logical-return-trees)
now composes bounded all-return nested selections with guarded inner conditions,
isolated private leaves and one shared original-work budget. The
[private ancestor-staging follow-up](reference/nuis-native-effectful-scalar-selection-v1.md#private-return-ancestor-staging)
now retains fresh scalar work before terminal inner selections, with current
private versions/seals and complete descendant captures. The
[private bool-update follow-up](reference/nuis-native-effectful-scalar-selection-v1.md#private-return-bool-updates)
adds bounded consecutive paired/one-sided updates of prefix-owned bools, retaining
current values without permitting incoming or sealed writes. The
[nested private-update follow-up](reference/nuis-native-effectful-scalar-selection-v1.md#nested-private-return-bool-updates)
adds nested terminal exports with bounded private child staging, seals and unused
work, retaining actual current outcomes and shared original-work limits. The
[ordered private-update follow-up](reference/nuis-native-private-ordered-return-updates-v1.md)
adds source-ordered writes to the single authorized target in those prefixes,
without treating permission as a source-defined version. The
[retention-tail follow-up](reference/nuis-native-private-retention-return-updates-v1.md)
discovers actual earlier writes when terminal selections only retain current
values, keeping fresh declarations and no-write candidates separate from
ownership. The [trailing-private-work follow-up](reference/nuis-native-private-trailing-return-updates-v1.md)
also proves final private let/const work without resetting seals, dropping unused
work or exporting the fresh binding. The [terminal-private-child follow-up](reference/nuis-native-private-terminal-child-return-updates-v1.md)
adds terminal updates of one prefix-defined private child without exporting it
instead of the ancestor. The [terminal mixed-update follow-up](reference/nuis-native-private-mixed-return-updates-v1.md)
preserves both source orders of ancestor/private-child writes with one outward
ancestor. The [flat mixed-continuation follow-up](reference/nuis-native-private-flat-mixed-continuations-v1.md)
keeps later child reads in branch-current scopes with one outward ancestor and
bounded nonrecursive recipe copying. The [nested branch-tree follow-up](reference/nuis-native-private-nested-mixed-continuations-v1.md)
keeps branch-current nested gates and copies only flat suffix recipes at original
leaves. The [sequential mixed-continuation follow-up](reference/nuis-native-private-sequential-mixed-continuations-v1.md)
preserves both current bool versions with bounded private scalar transport and
linear installation. The [nested sequential follow-up](reference/nuis-native-private-nested-sequential-mixed-continuations-v1.md)
also composes branch-local sequences with whole-forest preflight and no tail
copies. The [i32-pair follow-up](reference/nuis-native-private-i32-pair-continuations-v1.md)
also preserves an exact signed private child. The [i64-pair follow-up](reference/nuis-native-private-i64-pair-continuations-v1.md)
adds an exact private owned result for a full-width signed child. The [float-pair follow-up](reference/nuis-native-private-float-pair-continuations-v1.md)
reuses that result contract for exact f32/f64 values and bit-preserving native joins.
The [event-failure follow-up](reference/nuis-native-private-continuation-event-failure-v1.md)
separates last-state publication, fatal native traps and recoverable cleanup.
The [event-budget follow-up](reference/nuis-native-private-continuation-event-budget-v1.md)
checks exact inclusive entry limits and unchanged last state at a rejected private
publication continuation, including repeated event/close reset.
The [i32-ancestor follow-up](reference/nuis-native-private-i32-ancestor-continuations-v1.md)
adds one exact signed outward ancestor beside a private bool, retaining the
surrounding bool-return contract and original single-target authority.
The [i64-ancestor follow-up](reference/nuis-native-private-i64-ancestor-continuations-v1.md)
preserves the full signed width in exact private owned fields and keeps one
outward target inside the surrounding bool return.
The [float-ancestor follow-up](reference/nuis-native-private-float-ancestor-continuations-v1.md)
preserves exact f32/f64 outward fields beside a private bool, with raw native
callee-bit and event-publication checks through cache and source-free images.
The [dual-i64 follow-up](reference/nuis-native-private-dual-i64-continuations-v1.md)
adds one exact i64 outward owner beside a private i64 child, with actual pair
observations and cache/source-free event-publication proof.
The [i64/float follow-up](reference/nuis-native-private-i64-float-continuations-v1.md)
adds exact f32/f64 private fields beside one i64 outward owner, with raw IEEE
pair observations and cache/source-free publication checks.
The [float/i64 follow-up](reference/nuis-native-private-float-i64-continuations-v1.md)
also carries a private i64 beside an exact f32/f64 outward owner, preserving
actual IEEE bits, integer extrema and last accepted publication.
The [same-width float follow-up](reference/nuis-native-private-same-float-continuations-v1.md)
adds exact f32/f32 and f64/f64 private pairs with independent input and output words.
The [cross-width float follow-up](reference/nuis-native-private-cross-float-continuations-v1.md)
adds f32/f64 and f64/f32 private fields without numeric conversion or narrowing.
The [i32/i64 follow-up](reference/nuis-native-private-i32-i64-continuations-v1.md)
adds exact signed-width fields without reusing bool/i32 packing.
The [i64/i32 reverse follow-up](reference/nuis-native-private-i64-i32-continuations-v1.md)
retains a full i64 outward owner beside a private i32, using the same independent fields.
The [same-width i32 follow-up](reference/nuis-native-private-i32-i32-continuations-v1.md)
keeps both integer fields distinct through crossed extrema and source-free publication checks.
The [i32/float follow-up](reference/nuis-native-private-i32-float-continuations-v1.md)
adds exact f32/f64 children beside an i32 owner, separating raw IEEE and finite CLI proofs.
The [float/i32 reverse follow-up](reference/nuis-native-private-float-i32-continuations-v1.md)
adds exact floating owners beside i32 children, including current zero/negative-one updates.
The [three-field continuation proof](reference/nuis-native-private-three-field-continuations-v1.md)
adds two private numeric children under a separate lexical certificate and a field-driven carrier.
Further private-child arities and generalized resource transport,
other entering writes and mixed/single exits
remain separate.

The [application-led agreement](versioning/nuis-beta-0.11-application-led-mainline.md)
still governs this line. The explicit scripted AppKit cancellation route does not imply
general interactive/parent cancellation, provider/device retirement, a self-contained image
or completed compiler self-hosting. Use the references below for each boundary.

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
* [Compiler stage transformation](reference/nuis-compiler-stage-transformation.md)
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

* [Registered application session](reference/nuis-yir-application-session-v1.md)
* [Stateful window](reference/nuis-yir-window-session-v3.md)
* [Application cancellation](reference/nuis-yir-application-cancellation-v1.md)
* [Terminal outcomes](reference/nuis-yir-application-outcome-v1.md)
* [Independent CPU parent](reference/nuis-yir-application-outcome-pump-v1.md)
* [Native artifact workflow](reference/nuis-native-artifact-workflow.md)
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
* [ns-nova engine](../stdlib/ns-nova/README.md)
* [Std mainline layering](reference/std-mainline-layering-contract.md)
* [PixelMagic mainline](reference/pixelmagic-mainline-contract.md)
* [Shader/kernel project contract](reference/std-shader-kernel-project-contract.md)
* [Project examples](../examples/projects/README.md)
* [Source examples](../examples/ns/README.md)
* [Observable CLI regression](../tools/nuis/tests/std_filesystem_smoke.rs)

CLI evidence covers real bounded host input/output programs. Device declarations,
recipe names and report text are not substitutes for dispatched device results.

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
