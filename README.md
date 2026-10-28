# nuislang

> An AOT-first heterogeneous systems language and toolchain built around
> `nuis -> NIR -> YIR -> registered Nustar backends -> Nsld -> runtime`.

Nuis treats CPU, shader, kernel, data, network, and C compatibility as
registered execution domains under shared function/node, clock, GLM, artifact,
and lifecycle contracts. LLVM and host operating systems are bootstrap
backends, not the language's semantic root.

The [software manufacturing architecture](docs/reference/nuis-software-manufacturing-architecture.md)
sets the wider direction: the OS hosts execution; Nuis owns production contracts.
Nuis-rc is the intended machine-local resource control plane, with decentralized
acquisition and separate discovery, identity, integrity and trust. This is an
architecture commitment, not a claim of an implemented CAS or resident collector.

## Current Line

The repository is on `beta-0.18.*`. Git history is authoritative; the dated
[current snapshot](docs/versioning/nuis-beta-0.18.0-snapshot.md) records committed
identity separately from worktree follow-ups. Cargo package and protocol versions
are independent. Use the [validation checklist](docs/versioning/nuis-beta-0.18.0-release-checklist.md)
for operating gates and the [versioning index](docs/versioning/README.md) for history.

The mainline is **ns-nova application-led development**: grow one Nuis-owned
interactive image application, fix the foundation gaps it exposes, measure
performance, and progressively replace independently testable compiler modules
with Nuis implementations. This is a formal staged self-hosting migration line,
not a self-hosted compiler, stable public API or production-ready engine.

Start with the [current mainline map](docs/current-mainline-map.md),
[development tensor](docs/reference/nuis-development-tensor.md) and
[documentation index](docs/README.md). Detailed source/native/source-free proofs
live in the [native session proof index](docs/reference/nuis-native-session-proof-index.md),
not in duplicated entrypoint changelogs.

## Quick Start

Run from the repository root with the Rust toolchain, the host LLVM/linker
requirements for the selected profile, and any provider-specific prerequisites.
Start with a host CLI route; GPU/window examples require their registered backend.

```bash
export CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=1
cargo run -p nuis -- dev-tensor
cargo run -p nuis -- workflow examples/projects/tooling/filesystem_io_report_demo
cargo run -p nuis -- check examples/projects/tooling/filesystem_io_report_demo
cargo run -p nuis -- build examples/projects/tooling/filesystem_io_report_demo target/nuis-readme/cli-report
cargo run -p nuis -- run-artifact target/nuis-readme/cli-report
```

For a compiled Metal image application on a supported Apple Silicon host, follow
the [image showcase instructions](examples/projects/domains/ns_nova_image_showcase/README.md)
for `--export-frame`, `--window-session` and optional
`--window-parent-session`. The launcher still supplies registered provider
sessions and artifact sidecars; copying only the executable is not this route.

Use `--json` on supported inspection/workflow surfaces for structured evidence.
Do not treat inspection as execution: `run-artifact --json` does not run the app.

## Capability Snapshot

| Surface | Verified Scope | Important Boundary |
| --- | --- | --- |
| Small host CLI tools | Native builds exercise argv, piped stdin, stdout/stderr, file reads/writes, text statistics, reports and simple PGM transformations. | Examples have bounded inputs; this is not blanket compatibility or production certification. |
| Compiler | Parsing, types, generics, control flow, NIR/YIR verification, LLVM lowering and AOT emission have focused regressions, including bounded Buffer-writing loops. | Supported syntax does not imply every combination lowers; arbitrary `while` bodies and some aggregate early-return shapes remain incomplete. |
| Image application | Nuis generates RGBA8 data, inline WGSL runs RGB inversion on real Metal, and compiled host execution exports checked PPM frames. | The host embeds the YIR lifecycle runtime and uses registered providers; it is not fully native CPU execution or a self-contained Nsld image. |
| Persistent window | Registered Nuis open/event/close callbacks retain state and one provider connection across AppKit events. A separate registered CPU parent can consume one terminal outcome. | Explicit mode, bounded dispatch/replay, one-child parent profile; not an unlimited engine loop or general supervisor. |
| Cancellation | Independent tickets carry provider-worker drain into a typed, non-success result. AppKit and the headless build/run-artifact route share the classifier and launch policy; Metal regressions retain prior success evidence. | AppKit cancellation policy remains scripted; headless uses bounded scalar scripts. Linux hardware evidence, Windows transport, parent cancellation and resource reuse remain open. |
| Other backends | Checked-in routes include Linux CUDA/Vulkan and Apple Metal/CoreML provider work. | Evidence is backend- and workload-specific; reference results and hardware-free conformance do not certify physical execution. |
| Nsld | Deterministic plans/NSB assembly, first ARM64 Mach-O and x86_64 Linux ELF private-shell routes, loader admission, publication and final-output selection. | Broader architecture/provider parity, PE/COFF final execution and self-contained application packaging remain incomplete. |
| Self-hosting | Five bounded preparation gates, Nuis compiler-component proofs, differential/reproducibility evidence and explicit selection/rollback contracts. | `stage0-to-stage1-migration/active` is not completed compiler replacement; the bounded candidate-to-Nsld path stops before native object emission. |

The [observable CLI regression](tools/nuis/tests/std_filesystem_smoke.rs)
builds and runs actual host executables, rather than only checking source.
For example, [cli_wc_demo](examples/projects/tooling/cli_wc_demo) reads one
4096-byte chunk and uses ASCII word separators; it is not a complete `wc`.
Large-file streaming, broader text semantics, long-running concurrency and
cross-platform distribution need their own acceptance tests.

Tensor `stable/100` means stable for the recorded bounded milestone, not
language, stdlib, ABI, package, or whole-engine stability. Current task selection
comes from `nuis dev-tensor --json`, not a hardcoded progress total in this page.

Development-resource management follows the
[Nuis-rc heap/stack contract](docs/reference/nuis-rc-cache-lifecycle.md): default
shared heap packages, explicit project-owned stack packages, and root/lease-based
GC. The read-only policy core is tested; shared-store migration, compiler leases,
and the actual resident collector remain unimplemented. Existing caches are unchanged.

## Current Mainline

The application goal is `standard-library/ns-nova/interactive-image-workflow`.
The selected prerequisite remains
`standard-library/ns-nova/persistent-application-session`; consult
`nuis dev-tensor --json` for its live evidence, next action and validation command.

The image showcase exercises real Metal image processing and explicit stateful
AppKit sessions. Its compiled host still embeds the YIR lifecycle runtime.
The separate native-session profile executes bounded registered CPU callbacks;
one profile's acceptance must not be attributed to the other.

### Native CPU Progress

| Area | Current Bounded Evidence | Remaining Boundary |
| --- | --- | --- |
| Returns and storage | [Independent effect order](docs/reference/nuis-native-return-effect-prefix-v1.md), [certified storage reuse](docs/reference/nuis-native-return-effect-storage-v1.md), and [owned-record origins](docs/reference/nuis-native-origin-record-join-v1.md). | Unproved reductions retain independent payload storage and a separate pending signal. |
| Print values | [Pure computed i64 values](docs/reference/nuis-native-computed-iteration-print-v1.md), [root entry reads](docs/reference/nuis-native-carry-epoch-print-v1.md) and [typed field/call arguments](docs/reference/nuis-native-i64-xor-argument-print-v1.md). | Exact fields, eager bool operands, comparison-scoped i32 literals, wrapping arithmetic/negation, checked Div/Rem, explicit signed widening, restricted i64 arithmetic and eager XOR; not ambient readiness, whole-record captures, effects or I/O/resource capture. |
| Nested iteration | The [child-epoch follow-up](docs/reference/nuis-native-child-epoch-print-v1.md) checks fresh ready child entries, skipped work and later-parent failures. | Worktree evidence at the dated audit, not automatically part of the committed release. |

The [mainline map](docs/current-mainline-map.md) owns the next selected proof and
its remaining acceptance boundary; the
[proof index](docs/reference/nuis-native-session-proof-index.md) owns the complete
source/native/source-free history and each receipt's exact acceptance limits.

Cancellation admission, host-scope retirement and provider/device retirement
are separate facts. Rendering, control, ML and audio are ns-nova's engine
horizon, not four completed workflows. Independent Galaxies and Nustars share
YIR/GLM/time contracts without engine-specific compiler cases.

## Native Artifact Closure

The checked-in linker pressure route is:

```bash
cargo run -p nuis -- build examples/projects/tooling/native_artifact_closure_demo target/nuis-readme/native-artifact
cargo run -p nuis -- artifact-doctor target/nuis-readme/native-artifact
cargo run -p nsld -- drive target/nuis-readme/native-artifact/nuis.build.manifest.toml
cargo run -p nsld -- drive target/nuis-readme/native-artifact/nuis.build.manifest.toml --apply --until-clean --json
cargo run -p nuis -- run-artifact target/nuis-readme/native-artifact
```

`nsld drive` is non-mutating without `--apply`. Applying writes whitelisted
next artifacts and reports blocked boundaries instead of bypassing them.
Compatibility output remains the default; explicit private-image selection
requires independently validated admission and selection evidence.
Read the [native workflow](docs/reference/nuis-native-artifact-workflow.md),
[Nsld frontdoor](docs/reference/nsld-linker-frontdoor.md), and
[binary assembly gap map](docs/reference/nsld-binary-assembly-gap-map.md).

## Repository Map

| Path | Responsibility |
| --- | --- |
| [`tools/`](tools) | Project/compiler/linker/debugger/bundler frontdoors and host/YIR tools |
| [`crates/`](crates) | Reusable compiler, semantic, artifact, runtime and domain capabilities |
| [`nustar-packages/`](nustar-packages) | Static manifests, backend registrations, ABI targets and packaged assets |
| [`stdlib/`](stdlib) | Nuis sources for core, std, PixelMagic, WitSage and ns-nova |
| [`examples/`](examples) | Runnable projects, source probes, verifier cases and explicit legacy material |
| [`docs/reference/`](docs/reference) | Current implementation contracts and boundaries |
| [`docs/versioning/`](docs/versioning) | Minor-line history and long-range policy |
| [`subprojects/`](subprojects) | Separately owned Vulpoya and Yalivia shells |
| [`scripts/`](scripts) | Maintenance and developer-machine helpers |

See the [repository layout](docs/repo-layout.md) and
[documentation index](docs/README.md) before adding another top-level entry.

## Toolchain Boundaries

```text
nuis             workflow and project frontdoor
nuisc            compiler core and AOT artifact producer
nsld             deterministic linking and binary assembly
nuis-runtime     lifecycle loader and execution context
yir-runtime-host embedded-YIR application/window host boundary
nuis-host-runner host compatibility launcher
nsdb             YIR debugging and replay
nsbdr            OS bundle/distribution adapter over final Nsld outputs
```

Frontdoors should be thin adapters over reusable capabilities. The compiler knows
Nustar registration contract shapes and asks registered packages for domain
behavior; the CFFI package owns the generated GNU resolver and symbol-version registry.
The C world is an explicit compatibility domain, not a privileged linker model.
See the [CFFI contract](docs/reference/cffi-von-neumann-domain-contract.md),
[pointer safety boundary](docs/reference/ffi-pointer-safety-boundary.md), and
[toolchain capability boundary](docs/reference/toolchain-galaxy-core-boundary.md).

## Libraries And Examples

```text
core -> std -> pixelmagic
core -> std -> witsage
core -> std -> ns-nova
```

`core` is the semantic base; `std` owns practical systems contracts. PixelMagic
owns image algorithms, WitSage owns classical ML, and ns-nova composes application
policy without absorbing either package. The [stdlib index](stdlib/README.md)
and [examples router](examples/README.md) distinguish runnable projects from
recipes, metadata proofs and legacy probes. A file's existence or a backend
declaration alone is not execution evidence.

## Development

The [CI cold-start contract](docs/reference/nuis-ci-cold-start.md) separates
source-only checks, locked dependency fetch, focused portability validation and
the workspace build. A clean checkout must not need existing Cargo caches or
generated `target/` files to pass its source guards.

The current Ubuntu [Build workflow](.github/workflows/build.yml) runs those
checks, including one exact host-path portability test; it does not run the full
compiler semantic, native-session, self-hosting candidate or physical GPU suites.
A green Build is not whole-toolchain or cross-platform certification.

Prefer focused checks rather than rebuilding the workspace on every edit:

```bash
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test std_filesystem_smoke std_tooling_observable_cli_smoke_checks_reports_and_stdin -j 1 -- --exact --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis dev_tensor -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test file_line_limit -j 1 -- --test-threads=1
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s scripts/tests -v
python3 scripts/check-workspace-architecture.py
cargo fmt --all -- --check
git diff --check
```

The [current validation checklist](docs/versioning/nuis-beta-0.18.0-release-checklist.md)
separates existing CI coverage from focused native, migration and provider checks.
The [beta-0.12 checklist](docs/versioning/nuis-beta-0.12.0-release-checklist.md)
remains historical evidence, not a current acceptance result.
For disk cleanup, inspect `scripts/disk-clean-safe.sh` output before choosing
`--apply`. Add `--build-binaries` to retire hashed test/build executables while
keeping root CLI binaries and libraries; `--workspace` explicitly drops the
whole root build tree. Cleanup is workspace-only and refuses tracked or symlinked
paths. See the [cleanup policy](docs/repo-cleanup-candidates.md); do not remove
source, live regressions or unrelated project data. Generated example bundles
are rebuilt from source, not shipped in the checkout.

Dev/test incremental caching is opt-in; debug information and release settings
are unchanged. Use `CARGO_INCREMENTAL=1` explicitly when faster repeated edits
justify the extra storage. The [architecture/storage audit](docs/repo-architecture-audit.md)
separates enforced domain boundaries from compiler/linker/debugger extraction debt.

Use repository-relative paths and registered target/provider identities, not
personal directories, private host addresses or a fixed macOS version. Prefer
remote Linux infrastructure for Docker and CUDA-heavy tests. Rust/Nuis source
defaults to 800 lines, tests to 1000, Markdown to 2000; see the
[file-line policy](docs/repo-file-line-policy.md).

## Long-Range Direction

Staged migration began at `beta-0.10.*`; ns-nova now supplies application pressure
for foundation hardening, measured optimization and module-by-module ownership
transfer. The five closed preparation gates do not replace the Rust compiler.
The bounded Nuis candidate, trust/rollback chain and remaining native-object
boundary are documented in [self-hosting readiness](docs/reference/nuis-self-hosting-readiness.md)
and [candidate-to-Nsld materialization](docs/reference/nuis-compiler-candidate-nsld-materialization.md).

The approximate stage2-equivalent target remains `gamma-0.5.*` through
`gamma-0.10.*`, not a deadline or a claim of full engine maturity. Wider
Vulpoya/Yalivia collaboration, runtime/framework maturity and a self-owned
heterogeneous OS/hardware stack are longer-range work. Read the
[OS roadmap](docs/versioning/nuis-long-range-heterogeneous-os-roadmap.md),
[GLM positioning](docs/glm-spec/glm-heterogeneous-flow-graph-positioning.md) and
[Vulpoya/YIR review boundary](docs/glm-spec/vulpoya-yir-secondary-review-positioning.md).
Hardware extensibility and scheduling performance are design goals, not measured
advantages over other compiler infrastructures.
