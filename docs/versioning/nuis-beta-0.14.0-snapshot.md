# Beta 0.14 Snapshot

This minor-line anchor records Git checkpoint
`1fcfd655b2552f543baa38d4abf9d8a566ecbd3c` (`beta-0.14.1`, 2026-09-13).
The `.0` filename identifies the minor series; it does not assign every
capability below to its first patch. This is a documentation checkpoint, not
a new release, compatibility freeze, or completed self-hosting announcement.
Cargo package versions remain independent of the project release line.

## What This Line Means

The governing direction remains
[ns-nova application-led development](nuis-beta-0.11-application-led-mainline.md):
use a Nuis-owned interactive image application to expose foundation gaps,
measure them, and transfer independently testable compiler responsibilities
incrementally. The current goal is
`standard-library/ns-nova/interactive-image-workflow`; the selected prerequisite
remains `standard-library/ns-nova/persistent-application-session`.
Use the [current mainline map](../current-mainline-map.md) and live development
tensor for task selection rather than treating this snapshot as a progress counter.

The [beta-0.12 snapshot](nuis-beta-0.12.0-snapshot.md) and its
[checklist](nuis-beta-0.12.0-release-checklist.md) remain historical records of
`505c820c` (`beta-0.12.2`). They are not rewritten to describe this checkpoint.
The intervening patch history remains in Git; this note is not a retrospective
claim about the exact patch that introduced every accumulated capability.

## Highest-Signal Current Surfaces

| Surface | Checked-in implementation and regression scope | Boundary |
| --- | --- | --- |
| Native scalar sessions | Explicit `native-session-aot-bundle:<id>` selects registered native open/event/close helpers. The profile includes typed scalar state, bounded acyclic helpers, admitted flat-i64 aggregate returns, and checked counted loops. | Resources and provider effects remain outside this profile. The default image host still executes CPU callbacks as embedded YIR. |
| Conditional loop carries | Omitted or empty arms preserve the carry's own value. Pure comparison leaves compose with `&&`/`||`, nesting and grouping through shared condition metadata and short-circuit LLVM blocks. | Induction must remain finite and non-wrapping within 65536 iterations per admitted loop. Nested carry-update arms, arbitrary loop bodies and richer carry payloads remain separate work. |
| Artifact delivery | Native-session frontdoor regressions cover build/run-artifact, exact state output, tamper rejection before open, registration-specific cache isolation, and standalone materialization after deleting the original build directory. | These are selected host-native scalar workflows, not physical GPU certification or self-contained Nsld image completion. |
| Image/session lifecycle | Registered image workflows retain explicit window/headless selection, typed cancellation and provider-owned drain boundaries, with workload-specific Metal evidence documented separately. | Cancellation admission, host-scope retirement and provider/device retirement are different observations. General parent cancellation and cross-session resource reuse are not established. |
| Compiler migration | Five bounded preparation gates and candidate production, stage handoff, differential/reproducibility and selection/rollback contracts are recorded. | `stage0-to-stage1-migration/active` is not full compiler replacement. General candidate-owned native object emission remains a later boundary. |
| Development-resource policy | Nuis-rc implements read-only heap/stack lifetime selection and GC planning over a complete caller-verified ownership graph. Roots, leases and dependency closures protect live objects. | A plan is not deletion authority. Shared-store migration, compiler lease integration and the resident collector remain unimplemented. |

The implementation-facing references are the
[native scalar session bridge](../reference/nuis-native-scalar-session-bridge-v1.md),
[cancellation contract](../reference/nuis-yir-application-cancellation-v1.md),
[provider drain contract](../reference/nuis-yir-provider-session-drain-v1.md),
[self-hosting readiness](../reference/nuis-self-hosting-readiness.md), and
[RC cache lifecycle](../reference/nuis-rc-cache-lifecycle.md).

Source-level regression entry points include
[native application bridge](../../tools/nuisc/tests/native_application_bridge.rs),
[native-session workflow](../../tools/nuis/tests/native_session_workflow.rs), and
[compiler candidate production](../../tools/nuis/tests/compiler_structural_projection_candidate.rs).
Their presence describes coverage available to run, not a fresh execution result
from this documentation update.

## What The Green Build Establishes

[Build #605](https://github.com/Team-silvortex/nuislang/actions/runs/34745939424)
completed successfully for this exact source checkpoint. Its
[workflow](../../.github/workflows/build.yml) runs Python CI-guard regressions,
UTF-8 and documentation-link checks, locked dependency fetch, the focused
[host-path portability test](../../scripts/check-host-absolute-paths.sh), and
`cargo build --workspace --locked` on Ubuntu.

It does not run the full compiler semantic suite, native-session workflows,
compiler candidate migration suite, or physical GPU tests. Build success is not
whole-toolchain or cross-platform certification. No new native or hardware test
result is asserted by this documentation-only synchronization; use the separate
[validation checklist](nuis-beta-0.14.0-release-checklist.md) and record actual
commands, revisions, prerequisites and outcomes.

## Reading And Working Order

1. [Current mainline map](../current-mainline-map.md)
2. [Mainline dependency selection](../reference/nuis-development-tensor-mainline.md)
3. [Native scalar session bridge](../reference/nuis-native-scalar-session-bridge-v1.md)
4. [Image showcase](../../examples/projects/domains/ns_nova_image_showcase/README.md)
5. [Self-hosting readiness and per-gate commands](../reference/nuis-self-hosting-readiness.toml)
6. [Beta-0.14 validation checklist](nuis-beta-0.14.0-release-checklist.md)

From the repository root:

```sh
export CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=1
cargo run --locked -p nuis -- dev-tensor --json
cargo run --locked -p nuis -- workflow examples/projects/tooling/filesystem_io_report_demo
```

These commands inspect task selection and the project workflow; they do not
execute a native application or certify a provider. Follow the selected profile's
build/run-artifact and validation instructions for execution evidence.

Rule of thumb: a bounded capability, a build result, a native execution result
and a physical provider result are distinct facts. Preserve that separation
while application work drives the next compiler migration slice.
