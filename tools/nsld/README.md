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

## Core Responsibilities

Future `nsld-core` or equivalent galaxy-style capability should own:

* deterministic link graphs and link-unit registration
* lifecycle hook ordering and global clock metadata
* deterministic section and data-segment layout
* object-plan target identity metadata before platform object bytes are emitted
* native object output verification against deterministic image bytes
* unified heterogeneous container metadata
* static C-world compatibility wrapper policy

The CLI should remain a human and script entry point for those capabilities.

## Current Alpha Rule

Do not add new linker semantics only as formatted command output. If a command
needs to expose new information, add or preserve a structured representation so
`nuisc`, future IDE tools, `yalivia`, Nuis OS surfaces, and test/benchmark
flows can consume the same contract without shelling through text.
