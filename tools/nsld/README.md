# Nsld

`nsld` is the Nuis linker front-door.

In the current alpha line it is still a CLI wrapper over repository-owned
linker contract logic, including `nuisc::linker` helpers. That is intentional:
the tool exists before the final self-owned linker core so the toolchain can
start exercising linker plans, clock ordering, section/container metadata, and
heterogeneous binary contracts early.

## Boundary

The long-term shape is:

```text
nsld core capability -> CLI adapter
```

The CLI should never become the only durable protocol. New behavior should be
modeled as structured linker data first, then rendered for terminal output.

Nsld's native linker contract is not required to be a traditional `.o`-first
pipeline. Object files, Mach-O, ELF, PE/COFF, and host-native executable formats
belong to compatibility and finalization backends. The core linker should be
able to consume Nuis-owned link graphs, lifecycle/clock metadata, section
manifests, heterogeneous payloads, and container metadata directly, then choose
whether to emit a Nuis container, a host-native wrapper, or a compatibility
object/executable format.

The same rule applies to the larger C world: C ABI, libc, native object files,
and the classic von-Neumann host stack should be modeled as a CFFI /
host-compat capability domain inside Nuis, not as the implicit substrate that
defines all linker semantics.

## Core Responsibilities

Future `nsld-core` or equivalent galaxy-style capability should own:

* deterministic link graphs and link-unit registration
* lifecycle hook ordering and global clock metadata
* deterministic section and data-segment layout
* object-plan target identity metadata for optional platform compatibility bytes
* native object output verification when a compatibility object is emitted
* unified heterogeneous container metadata
* final executable layout and internal image dry-run metadata
* lifecycle-scoped native-object lanes for CFFI/host compatibility payloads
* static C-world compatibility wrapper policy
* host-compat domain metadata that keeps C/von-Neumann execution explicit,
  scheduled, and verifiable

Current `prepare`, `check`, `closure`, `container`, and `verify-container`
reports expose this host-compat domain metadata as compatibility-domain
summary fields. JSON output keeps flat fields for alpha compatibility and also
provides object-shaped summaries such as `compatibility_domain_summary`,
`container_compatibility_domain_summary`, and verify-container
expected/actual summaries. Treat those fields as linker protocol, not as
cosmetic CLI output.

The CLI should remain a human and script entry point for those capabilities.

## Current Alpha Rule

Do not add new linker semantics only as formatted command output. If a command
needs to expose new information, add or preserve a structured representation so
`nuisc`, future IDE tools, `yalivia`, Nuis OS surfaces, and test/benchmark
flows can consume the same contract without shelling through text.
