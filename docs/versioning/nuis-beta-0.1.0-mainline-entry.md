# `nuis` `beta-0.1.0` Mainline Entry

This file is the current short entry point for the second beta minor line.

`beta-0.1.0` keeps the early-beta foundation-hardening posture while making
host compatibility an explicit registered domain. It does not mean stable
public APIs, package compatibility, self-hosting, or production-complete
heterogeneous execution.

The recorded predecessor is:

* [nuis-beta-0.0.1-mainline-entry.md](nuis-beta-0.0.1-mainline-entry.md)

The self-hosting horizon remains:

```text
beta-0.0.* through beta-0.9.*
  -> foundation closure, bug fixing, performance, and repeatability
beta-0.10.*
  -> formal stage0-to-stage1 self-hosting migration begins
gamma-0.5.*
  -> target stage2-equivalent compiler ownership
```

Short rule:

`beta-0.1.0 turns C compatibility from an implicit CPU privilege into a
registered, whitelistable, lifecycle-visible Nuis domain`

## Canonical Reading Order

1. [../current-mainline-map.md](../current-mainline-map.md)
2. [../reference/nuis-development-tensor.md](../reference/nuis-development-tensor.md)
3. [../reference/cffi-von-neumann-domain-contract.md](../reference/cffi-von-neumann-domain-contract.md)
4. [../reference/nuis-native-artifact-workflow.md](../reference/nuis-native-artifact-workflow.md)
5. [../reference/nsld-linker-frontdoor.md](../reference/nsld-linker-frontdoor.md)
6. [../reference/nsld-binary-assembly-gap-map.md](../reference/nsld-binary-assembly-gap-map.md)
7. [../reference/nustar-multi-backend-artifact-contract.md](../reference/nustar-multi-backend-artifact-contract.md)
8. [../reference/nuis-binary-format-protocol.md](../reference/nuis-binary-format-protocol.md)
9. [../reference/nsdb-yir-debugger-frontdoor.md](../reference/nsdb-yir-debugger-frontdoor.md)
10. [../reference/std-mainline-layering-contract.md](../reference/std-mainline-layering-contract.md)
11. [nuis-beta-0.0.1-mainline-entry.md](nuis-beta-0.0.1-mainline-entry.md)

## Current Connected Spine

```text
nuis source / nuis.toml
  -> nuis frontdoor
  -> nuisc
  -> NIR
  -> YIR + GLM / clock / domain verification
  -> registered Nustar packages and backend artifacts
  -> Nsld container / closure / final-output planning
  -> NSB image or explicit host-assisted final-output boundary
  -> run-artifact / Nsdb trace, replay, and completion evidence
  -> development tensor handoff
```

The compiler knows the shape of registration contracts, not a finite list of
backend implementations. Domain-specific parsing, ABI policy, lowering,
artifacts, execution adapters, and verification evidence remain registered
capabilities.

## Beta-0.1.0 Boundary

This line establishes:

* `official.cffi` as a first-class registered Nustar
* `mod cffi` as the required Nuis source boundary for `extern` declarations
* exact signature and hash-whitelist ownership in the CFFI manifest
* a CFFI YIR registration surface and lifecycle-visible artifact unit
* an explicit bootstrap bridge from CFFI policy to registered CPU/LLVM host
  machine-code production
* continued rejection of implicit `mod cpu` extern escape hatches
* migrated checked-in std, example, project, and test FFI sources
* development-tensor drift coverage for the new current line

The current bridge intentionally loads both `official.cffi` and
`official.cpu`. CFFI owns source admission and ABI policy; CPU owns the current
host machine-code realization. This is a visible bootstrap dependency, not a
claim that CFFI and CPU are the same domain.

## Data Hardware Direction

Programmable DPU and IPU hardware is a credible physical backend family for
the Data Nustar. It should enter through provider registration rather than
redefine the domain:

```text
Data YIR / GLM / clock / movement contract
  -> registered DPU, IPU, RDMA, CPU-memory, or device-memory provider
  -> backend artifact and lifecycle adapter
```

The Data Nustar remains broader than any current DPU. Placement, movement,
ownership, residency, synchronization, and future data-fabric hardware must
share the same provider-neutral contract.

## Beta-0.1 Main Target

1. keep the CFFI source and whitelist boundary strict while widening pointer,
   string, and object support only through registered contracts
2. advance the weakest runtime coordinate,
   `native-binary-system/nuis-runtime/lifecycle-context-dispatch`
3. preserve GLM, clock, dependency, replay, and final-image evidence across
   heterogeneous provider graphs
4. keep Nsld and Nsdb provider-neutral while real backends mature
5. model DPU/IPU support as an open Data provider family without vendor
   coupling
6. harden package/import and std behavior under larger CLI and heterogeneous
   programs
7. update the development tensor after each implementation tranche

## Honesty Boundary

`beta-0.1.0` should not claim:

* stable public API or package-format compatibility
* final self-hosting
* a native DPU/IPU backend already exists in the repository
* complete ELF, PE/COFF, and Mach-O parity
* production-complete GPU/NPU portability
* replacement of every host linker, debugger, runtime, or device toolchain
* mature Ns Nova framework readiness

Safe wording:

* `early beta foundation hardening`
* `registered CFFI source and ABI boundary`
* `registered heterogeneous provider closure`
* `Nsld/NSB image closure`
* `Nsdb trace/replay evidence`
* `host-assisted final-output boundary`
* `provider-neutral Data hardware direction`
* `self-hosting migration begins at beta-0.10.*`

## Version Surface Rule

`beta-0.1.0` is the project release line. Cargo package versions may remain on
their internal workspace versions until the repository adopts one explicit
package-version synchronization policy. Documentation must not infer the
project phase from an individual crate manifest.
