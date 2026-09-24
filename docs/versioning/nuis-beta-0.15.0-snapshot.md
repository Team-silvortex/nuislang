# Beta 0.15 Snapshot

This minor-line anchor records the existing Git commit
`05951befc70d6a145e4978a3ff8909737e5c4fbb` (`beta-0.15.0`, 2026-09-24).
It documents accumulated implementation and repository cleanup, not another
release, a compatibility freeze or a completed self-hosting announcement.
Cargo package versions and machine-readable protocol versions are independent.

## Direction And Selected Work

The governing direction remains
[ns-nova application-led development](nuis-beta-0.11-application-led-mainline.md):
develop a Nuis-owned image application, repair general foundation gaps it exposes,
measure performance and transfer independently testable compiler responsibilities.
The goal remains `standard-library/ns-nova/interactive-image-workflow`.
Its selected prerequisite is
`standard-library/ns-nova/persistent-application-session`, still `active/86`.

The next selected task is field demand through same-name branch-local capture
aliases, preserving lexical identity and guarded evaluation. Stable aliases and
straight-line record snapshots already have a bounded projection route. This is
not broader mutable alias analysis or permission to widen native argument limits.
Use the [mainline map](../current-mainline-map.md) and live tensor for later task
selection; documentation and disk cleanup do not raise capability scores.

The [beta-0.14 snapshot](nuis-beta-0.14.0-snapshot.md) remains the record of
`1fcfd65` (`beta-0.14.1`). The [beta-0.12 snapshot](nuis-beta-0.12.0-snapshot.md)
remains the record of `505c820c` (`beta-0.12.2`). Neither is rewritten to describe
this checkpoint, and intervening patch history remains in Git.

## Highest-Signal Surfaces

| Surface | Recorded implementation and regression scope | Boundary |
| --- | --- | --- |
| Native value returns | The selected native-session profile transports admitted nested scalar records as LLVM slot values. Helper and callback probes retain exact layout/kind checks, scalar bits, independent snapshots, overlap-safe publication and zero aggregate heap allocations. | Ordinary owned/resource aggregates retain their separate ABI. This is not a public ABI, zero stack traffic or measured runtime speedup. |
| Private captures | Generated helpers pack boolean leaves, project demanded fields and normalize stable record aliases and straight-line rebound snapshots. Sparse 64-i64 state uses four private selection arguments; alias/snapshot fixtures use two/three while public inputs stay unchanged. | Same-name branch-local aliases, control-flow writes and iteration-local aliases remain conservative. User, FFI and scoped-iteration signatures are not widened. |
| Control composition | Nested counted loops, typed carries, guarded exits/returns and continuation elision compose within the admitted pure-value profile. The full composition remains within 51 reachable native functions and the unchanged 64-function limit. | Induction preflight, helper-entry and loop-work budgets remain independent. This does not admit arbitrary resource-bearing loop bodies. |
| Artifact delivery | Explicit native-session build/run-artifact regressions cover cache identity, tamper rejection, lifecycle results and source-free standalone restoration, including sparse/alias/snapshot fixtures. | The default image host still executes CPU callbacks as embedded YIR. Native scalar closure is not native GPU callback dispatch or a self-contained Nsld application image. |
| Repository hygiene | Unused placeholders, obsolete pre-alpha gates and checked-in generated bundles are retired. CLI test fixtures reclaim their temporary project and derived default outputs; cleanup preflights repository-local ignored paths. | Live tests, source examples, minor history and independent subprojects remain. This is not Nuis-rc CAS migration or a resident collector. |
| Compiler migration | Five bounded preparation gates and component production, differential/reproducibility, selection and rollback contracts remain available. | `stage0-to-stage1-migration/active` is not completed compiler replacement; general candidate-owned native object emission remains separate work. |

Implementation references are the
[native scalar session bridge](../reference/nuis-native-scalar-session-bridge-v1.md),
[native value-return and capture contract](../reference/nuis-native-scalar-value-returns-v1.md),
[artifact workflow](../reference/nuis-native-artifact-workflow.md),
[repository cleanup policy](../repo-cleanup-candidates.md),
[self-hosting readiness](../reference/nuis-self-hosting-readiness.md) and
[Nuis-rc lifetime policy](../reference/nuis-rc-cache-lifecycle.md).

## Evidence At This Checkpoint

The immediately preceding macOS aarch64 cleanup validation recorded:

* a cold locked build of all 29 workspace packages
* 72 focused CLI/tensor/workflow tests, six source-example compilation tests and
  20 maintenance tests, all passing
* 1,414 clean tensor drift checks, with clean coverage, hierarchy and lineage
* UTF-8, formatting, documentation-link and changed-file line-limit checks
* workspace disk use reduced from about 7.9 GiB to 1.93 GiB after rebuilding and
  testing, with about 6 GiB net reclaimed

These results belong to the source state committed as `05951bef`. The
[cleanup record](../repo-cleanup-candidates.md) gives the scope and storage
boundary. The earlier 797-test alias/snapshot checkpoint remains separately
dated in the native reference documents; it is not another full-suite run here.
Test totals from different checkpoints must not be added into one coverage claim.

The checked-in [Build workflow](../../.github/workflows/build.yml) runs maintenance,
UTF-8 and link checks, locked dependency fetch, one focused host-path test and a
workspace build on Ubuntu. Its definition is not evidence that a new remote run
passed. No fresh Linux/GPU, Windows, long-duration performance or complete
self-hosting result is claimed by this documentation synchronization.

## Remaining Boundaries

Native CPU frame dispatch with resources, provider effects, cross-session resource
reuse and self-contained Nsld image packaging remain separate from scalar-session
closure. Cancellation admission, host-scope retirement and provider/device drain
remain distinct observations. Existing provider evidence stays attached to its
tested workload and hardware; a portable test cannot certify a physical device.

Nuis-rc still provides read-only policy and planning, not an installed shared CAS,
compiler lease integration or resident GC. The cleanup scripts remain maintenance
tools, not a substitute for that future resource-control plane. Rendering,
control, ML and audio remain the ns-nova engine horizon, not four completed systems.

## Working Entry Points

1. [Current mainline map](../current-mainline-map.md)
2. [Mainline dependency selection](../reference/nuis-development-tensor-mainline.md)
3. [Native value returns and captures](../reference/nuis-native-scalar-value-returns-v1.md)
4. [Image showcase](../../examples/projects/domains/ns_nova_image_showcase/README.md)
5. [Beta-0.15 validation checklist](nuis-beta-0.15.0-release-checklist.md)

From the repository root:

```sh
export CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=1
cargo run --locked -p nuis -- dev-tensor --json
bash scripts/disk-audit.sh
```

These inspect current task selection and local storage; they do not execute a
native application or authorize deletion. Follow the selected profile's explicit
build/run-artifact and provider acceptance instructions for execution evidence.
