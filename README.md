# nuislang

> An AOT-first heterogeneous systems language and toolchain built around
> `nuis -> NIR -> YIR -> registered Nustar backends -> Nsld -> runtime`.

Nuis treats CPU, shader, kernel, data, network, and C compatibility as
registered execution domains under one semantic, clock, GLM, artifact, and
lifecycle contract. LLVM and host operating systems are important bootstrap
backends, but they do not define the language model.

## Current Line

The repository is on `beta-0.7.*`. Git history is the authoritative source for
the exact patch checkpoint; the independent Cargo package versions are not the
project release number yet.

This is an early-beta hardening line, not an API-stability or production-ready
claim. The important change from the earlier alpha and first-beta snapshots is
that the major pieces now form one exercised toolchain spine:

```text
nuis source / nuis.toml
  -> nuis workflow frontdoor
  -> nuisc frontend and semantic checks
  -> NIR
  -> YIR + GLM / clock / domain verification
  -> registered Nustar lowering and backend artifacts
  -> Nsld link graph, NSB image, and host-shell finalization
  -> nuis-runtime lifecycle dispatch
  -> run-artifact / Nsdb execution evidence and replay metadata
```

The development tensor currently reports clean recursive hierarchy, milestone,
manifest, and `693/693` implementation-drift checks across `26/26` registered
coordinates. Five independent self-hosting readiness gates keep the
bootstrap-critical average honest at `97/100`; the weakest coordinate is now
`standard-library/std/compiler-data-model` at `usable`, `80/100`. The
stage-neutral handoff is `usable/84`, while the stage driver and differential
gate are both `usable/87`. The dedicated
`nuis bootstrap-build` frontdoor emits a complete compiler-image,
stage-handoff, dependency-closure, native-output, reproducible-identity, exact
audit record, and component-bound diagnostic proof for one project-form
compiler component. The `nuis bootstrap-diff` frontdoor compares an explicit
stage0/candidate pair across thirteen semantic, dependency, diagnostic, and
native-output identities while keeping replacement authorization separate. The
`nuis bootstrap-status` frontdoor reports `1/5` gates closed, while the paged
compiler data model is `usable`, `80/100` and the producer-neutral five-stage
handoff is `usable`, `84/100`. Its shared structural codec independently parses
and canonically re-renders AST/NIR payload hierarchy without reconstructing AST
from source. `nuis bootstrap-candidate-probe` now also compiles and executes a
pure Nuis typed structural consumer, then binds its stage0 component and native
image to an explicitly execution-only proof. `nuis bootstrap-candidate-build`
then feeds all five serialized payloads through twelve exact Nuis scalar
exports, independently verifies their folds, and drives the token payload
through a bounded Nuis-native DFA. The artifact layer independently decodes
the same token stream before production v2 emits a distinct attested
stage1-candidate and reaches repository-native `13/13` differential equivalence
while retaining `replacement_authorized = false`. `nuis
bootstrap-reproducibility` now performs
two cache-bypassed clean candidate builds, rereads both evidence roots, requires
stable compiler-image, native-output, and differential identities, and emits a
path-free reproducibility aggregate with independent replacement authorization
still disabled. This closes one real Nuis leaf production and reproducibility
loop; it is not a claim that the compiler is already self-hosted.

Nsld now carries the first ARM64 Mach-O and x86_64 Linux ELF routes through
private shell construction, independent validation, real OS-loader execution,
admission replay, atomic publication, and ordinary final-output selection. The
Linux route exercises versioned, hash-whitelisted `libc` and `libm` symbols in
one real GNU loader process. Explicit private selection persists relocatable
owner-private `nuis-nsld-final-output-selection-evidence-file-v1` evidence;
compatibility output remains the non-mutating default. Stale admission,
signature, registration, or image identity blocks mutation even when candidate
ELF bytes have not changed.

The first GNU resolver providers and symbol-version rows now belong to
`official.cffi`. Nuisc validates and preserves their registration contract,
while Nsld generates a static runtime table at build time without changing the
existing private-image or admission identity. With the producer-neutral
structural codec, typed Nuis consumer, bounded token decoder, candidate
execution proof, paged compiler data, first attested stage1 leaf, and
two-clean-build aggregate in place, the tensor now routes mainline work to
complete token-record storage, canonical token re-emission, and broader
compiler-data paging. A changed stage payload, reversible replacement
authorization, Galaxy hardening, broader ELF architecture coverage, and
PE/COFF remain separate registered foundation work.

Start with these documents:

* [Current mainline map](docs/current-mainline-map.md)
* [Beta 0.6 mainline entry](docs/versioning/nuis-beta-0.6.0-mainline-entry.md)
* [Development tensor](docs/reference/nuis-development-tensor.md)
* [Self-hosting readiness](docs/reference/nuis-self-hosting-readiness.md)
* [Compiler data model](docs/reference/nuis-compiler-data-model.md)
* [Compiler stage handoff](docs/reference/nuis-compiler-stage-handoff.md)
* [Compiler component build](docs/reference/nuis-compiler-component-build.md)
* [Compiler candidate execution](docs/reference/nuis-compiler-candidate-execution.md)
* [Compiler candidate production](docs/reference/nuis-compiler-candidate-production.md)
* [Compiler component differential gate](docs/reference/nuis-compiler-component-differential.md)
* [Compiler component reproducibility](docs/reference/nuis-compiler-component-reproducibility.md)
* [Native artifact workflow](docs/reference/nuis-native-artifact-workflow.md)
* [Nsld linker frontdoor](docs/reference/nsld-linker-frontdoor.md)
* [Binary assembly gap map](docs/reference/nsld-binary-assembly-gap-map.md)
* [Documentation index](docs/README.md)

Use [the versioning index](docs/versioning/README.md) only when you need older
minor-line snapshots. Alpha and pre-alpha documents are historical context, not
the default description of current behavior.

## Capability Snapshot

The following surfaces are implemented and exercised today:

* `nuis` owns project orientation, checking, testing, benchmarking, building,
  artifact inspection, runtime handoff, release checks, Galaxy workflows, and
  the development tensor.
* `nuisc` owns parsing, type/control-flow/generic validation, NIR and YIR
  production, verification, LLVM lowering, AOT emission, and project metadata.
  Its normalized `while let` path now carries ordered `i64` fields and both
  identity-updated and condition-driven replacement `bool` fields across real
  native backedges. The versioned
  bool transport selects a source-typed neutral value before encoding into the
  loop ABI and decodes during field-identity-preserving variant rebuild. Its
  per-slot codec is serialized in an optional YIR tail contract, then parsed
  and fail-closed validated by both the CPU domain and LLVM lowering. A shared
  YIR-core arity contract now admits single-state affine replacement sources;
  legacy YIR without the tail remains valid.
  Structured `continue` and `break` retain every matched binding's
  pre-transition value. `i32`, floating-point, and owned payloads still fail
  before carry construction with a typed-scalar or GLM-owned diagnostic.
* Nustar registration covers `cpu`, `data`, `shader`, `kernel`, `network`, and
  first-class `official.cffi` host compatibility without making the compiler a
  finite table of backend implementations. The CFFI package also owns the first
  generated GNU resolver and symbol-version registry.
* Registered provider paths carry lifecycle, clock, GLM, artifact, and
  completion evidence. Checked-in routes include host CPU, Metal/CoreML, and
  Linux CUDA/Vulkan provider work.
* `nsld` owns deterministic link planning, closure, NSB assembly, provider
  payload placement, native entry planning, the first ARM64 Mach-O and x86_64
  Linux ELF private-shell routes, loader admission, publication, and persisted
  final-output selection evidence.
* `nuis-runtime` and `nuis-host-runner` own the current lifecycle loader and
  host execution bridge; `nsdb` consumes YIR-level trace, handoff, cursor, and
  replay metadata.
* `std`, PixelMagic, and WitSage provide checked-in Nuis source contracts and
  runnable pressure routes for host IO/filesystem/text, concurrency, image
  processing, and classical ML.

The following boundaries are still intentionally incomplete:

* broader ELF architecture/provider parity and a PE/COFF final executable route
* stable package/import/autoinjection and public API compatibility policy
* complete raw-pointer and unsafe interoperability policy
* provider-neutral graph execution with equal maturity across all hardware
  families, especially the early Data provider lane
* a self-hosted compiler and a Nuis-native operating-system/runtime substrate

Tensor `stable` means stable for the recorded milestone slice. It must not be
read as a promise of language, stdlib, ABI, or package compatibility.

## Quick Start

When the next command is unclear, ask the workflow frontdoor first:

```bash
cargo run -p nuis -- dev-tensor
cargo run -p nuis -- bootstrap-status
cargo run -p nuis -- bootstrap-build examples/projects/tooling/bootstrap_compiler_data_model_demo build/bootstrap-component
cargo run -p nuis -- bootstrap-candidate-probe examples/projects/tooling/bootstrap_structural_projection_candidate build/bootstrap-candidate
cargo run -p nuis -- bootstrap-candidate-build examples/projects/tooling/bootstrap_structural_projection_candidate build/bootstrap-candidate-production
cargo run -p nuis -- bootstrap-reproducibility examples/projects/tooling/bootstrap_structural_projection_candidate build/bootstrap-reproducibility
cargo run -p nuis -- workflow examples/projects/kernel_tensor_demo
cargo run -p nuis -- project-doctor examples/projects/kernel_tensor_demo
cargo run -p nuis -- check examples/projects/kernel_tensor_demo
cargo run -p nuis -- test examples/projects/kernel_tensor_demo
cargo run -p nuis -- build \
  examples/projects/kernel_tensor_demo \
  target/nuis-readme/kernel_tensor_demo
cargo run -p nuis -- run-artifact \
  target/nuis-readme/kernel_tensor_demo
```

Use `--json` on workflow, tensor, inspection, linker, runtime, and release
surfaces when another tool needs structured evidence.

### Native Artifact Closure

The shortest checked-in linker pressure route is:

```bash
cargo run -p nuis -- build \
  examples/projects/tooling/native_artifact_closure_demo \
  target/nuis-readme/native_artifact_closure_demo

cargo run -p nuis -- artifact-doctor \
  target/nuis-readme/native_artifact_closure_demo

cargo run -p nsld -- drive \
  target/nuis-readme/native_artifact_closure_demo/nuis.build.manifest.toml

cargo run -p nsld -- drive \
  target/nuis-readme/native_artifact_closure_demo/nuis.build.manifest.toml \
  --apply --until-clean --json

cargo run -p nuis -- run-artifact \
  target/nuis-readme/native_artifact_closure_demo
```

`nsld drive` without `--apply` is non-mutating. Applying mode writes only
whitelisted next artifacts and stops with structured evidence when a boundary
is blocked rather than silently bypassing it.

Useful inspection commands:

```bash
cargo run -p nuis -- dump-ast examples/projects/kernel_tensor_demo
cargo run -p nuis -- dump-nir examples/projects/kernel_tensor_demo
cargo run -p nuis -- dump-yir examples/projects/kernel_tensor_demo
cargo run -p nuis -- project-status examples/projects/kernel_tensor_demo
cargo run -p nuis -- verify-build-manifest \
  target/nuis-readme/kernel_tensor_demo/nuis.build.manifest.toml
```

## Repository Map

| Path | Responsibility |
| --- | --- |
| [`tools/`](tools) | CLI frontdoors: `nuis`, `nuisc`, `nsld`, `nsdb`, `nsbdr`, the host runner, and YIR tools |
| [`crates/`](crates) | Reusable compiler, semantic, artifact, runtime, YIR, verifier, lowering, and domain capabilities |
| [`nustar-packages/`](nustar-packages) | Static Nustar manifests, backend registration metadata, ABI targets, and packaged assets |
| [`stdlib/`](stdlib) | Nuis source assets for `core`, `std`, PixelMagic, WitSage, and the later ns-nova framework |
| [`examples/`](examples) | Current projects and source probes, invalid/verifier cases, YIR anchors, and explicit legacy material |
| [`docs/reference/`](docs/reference) | Present-tense implementation and protocol truth |
| [`docs/versioning/`](docs/versioning) | Minor-line snapshots and long-range policy anchors |
| [`docs/*-spec/`](docs) | Broader grammar, GLM, fabric, and YIR design direction |
| [`subprojects/`](subprojects) | Explicitly separated Vulpoya and Yalivia project shells |
| [`scripts/`](scripts) | Repository maintenance and developer-machine helpers |

See [the detailed repository layout](docs/repo-layout.md) before adding a new
top-level directory. CLI code should stay an adapter over reusable capability
code; historical material belongs under the existing historical/versioning
routes rather than the mainline entry path.

## Toolchain Boundaries

```text
nuis             workflow and project frontdoor
nuisc            compiler core and AOT artifact producer
nsld             linker frontdoor and binary assembly owner
nuis-runtime     lifecycle loader and execution context
nuis-host-runner host compatibility launcher
nsdb             YIR semantic debugger and replay frontdoor
nsbdr            OS bundle/distribution adapter over final Nsld outputs
yir-*            lower-level YIR inspection, packing, execution, and export
```

`nsld`, `nsdb`, and `nsbdr` are command adapters over reusable toolchain
capabilities, not independent CLI-only logic piles. The same rule applies to
Nustar domains: the compiler knows registration contract shapes and asks the
registered package for domain behavior.

The C world is an explicit compatibility domain, not the hidden default machine
model. Read the [CFFI domain contract](docs/reference/cffi-von-neumann-domain-contract.md),
[FFI pointer safety boundary](docs/reference/ffi-pointer-safety-boundary.md),
and [toolchain capability boundary](docs/reference/toolchain-galaxy-core-boundary.md)
for the current rules.

## Libraries And Examples

The current official source layering is:

```text
core -> std -> pixelmagic
core -> std -> witsage
core -> std -> ns-nova
```

`core` is the smallest semantic base. `std` owns practical systems contracts.
PixelMagic exercises shader-facing image pipelines, WitSage exercises
kernel-facing classical ML, and ns-nova remains intentionally later than the
AOT, linker, runtime, std, and official-Galaxy foundations.

Use [the stdlib index](stdlib/README.md) and [the examples router](examples/README.md)
rather than treating every old `.ns` or handwritten YIR file as equally
current. The default runnable layer is [`examples/projects/`](examples/projects);
[`examples/legacy/`](examples/legacy) is explicit predecessor material.

## Development

Focused checks are preferred over rebuilding the whole workspace on every edit:

```bash
CARGO_INCREMENTAL=0 cargo test -q -p nuisc --lib --no-run -j 1
CARGO_INCREMENTAL=0 cargo test -q -p nsld -j 1
CARGO_INCREMENTAL=0 cargo test -q -p nuis --test self_contained_nsb_smoke -j 1
cargo fmt --check
git diff --check
```

For a small local disk, preview cleanup before applying it:

```bash
scripts/disk-clean-safe.sh
scripts/disk-clean-safe.sh --apply
scripts/disk-clean-safe.sh --apply --workspace --cargo-cache
```

Local development should not depend on absolute filesystem paths, private host
addresses, or one macOS release. Prefer project-relative paths and registered
target/provider identities. Use remote Linux infrastructure for Docker and
CUDA-heavy validation when available.

Rust and Nuis implementation files use an 800-line default, tests use 1000,
and Markdown uses 2000. See [the file-line policy](docs/repo-file-line-policy.md).

## Long-Range Direction

Nuis aims at a self-owned heterogeneous computing stack rather than a classic
C-shaped language with a thin syntax layer. Beta first hardens the compiler,
runtime, linker, package, stdlib, and provider foundations. Self-hosting
pressure becomes explicit later in beta, while a later gamma line is reserved
for whole-toolchain coordination, Vulpoya/Yalivia integration, and native
framework maturity before any `1.0.0` claim.

Read the [long-range heterogeneous OS roadmap](docs/versioning/nuis-long-range-heterogeneous-os-roadmap.md),
[GLM heterogeneous flow-graph positioning](docs/glm-spec/glm-heterogeneous-flow-graph-positioning.md),
and [Vulpoya/YIR secondary review positioning](docs/glm-spec/vulpoya-yir-secondary-review-positioning.md)
for that direction. Current implementation claims still come from the code,
tests, reference docs, and development tensor.
