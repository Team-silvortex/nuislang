# Toolchain Galaxy Core Boundary

This note defines the intended boundary for toolchain members such as `nsld`,
`nsdb`, and `nsbdr`.

The short rule is:

```text
core galaxy capability -> CLI adapter
```

The CLI is a front door, not the long-lived capability boundary.

## Why This Exists

`nsld`, `nsdb`, and `nsbdr` are toolchain members, but they are also future
reusable Nuis capabilities:

* `nsld` owns linker graph, lifecycle hook, deterministic section/container,
  and artifact manifest behavior.
* `nsdb` owns YIR-level debug metadata, trace, replay, frame/slot mapping, and
  semantic inspection behavior.
* `nsbdr` owns OS bundle and distribution adaptation over already-linked Nuis
  final outputs, such as `.app`, `.dmg`, and future installer/package formats.

If these capabilities only exist as shell commands, then `nuisc`, `yalivia`,
future IDE surfaces, CI, self-hosting flows, and Nuis OS-facing runtimes would
have to communicate through command-line text. That would make the real
protocol harder to stabilize.

Instead, each toolchain member should be read as:

```text
stable core package / galaxy interface
  -> library-style API
  -> CLI adapter
  -> human and script entry points
```

## Nsld Shape

`nsld` should eventually split conceptually into:

* `nsld-core`: link graph, global clock ordering, lifecycle hooks, section
  layout, container metadata, static link-unit registration, and C-world
  wrapper policy.
* `nsld-cli`: commands such as `status`, `prepare`, `check`, `container`, and
  `verify-*`.

`nuisc` should consume the `nsld-core` capability boundary directly when that
boundary becomes stable enough. It should not be forced to shell out to the CLI
for ordinary linker graph work.

## Nsdb Shape

`nsdb` should eventually split conceptually into:

* `nsdb-core`: YIR debug metadata, timestamped event views, YIR frame/slot
  maps, GLM state views, heterogeneous node traces, and semantic replay
  queries.
* `nsdb-cli`: human-facing commands such as `status`, `inspect`, and future
  stepping or query commands.

Native debuggers can still inspect the host shell binary. `nsdb-core` owns the
Nuis semantic debug view.

## Contract Rules

Use these rules when evolving linker/debugger code:

* The CLI may expose every important operation, but it must not be the only
  durable protocol.
* Core metadata should be structured and versioned before it is formatted for
  terminal output.
* `nuisc` should depend on core capability contracts, not command-line text.
* `yalivia`, future IDE tools, and future Nuis OS surfaces should be able to
  consume the same core metadata without pretending to be shell users.
* External C-world compatibility may be wrapped by these capabilities, but it
  should not receive special linker/debugger semantics outside the core
  contract.

## Current Alpha Boundary

Today this is a design boundary, not a full implementation split.

Current reality:

* `nsld` is still a CLI/frontdoor over repository-owned linker contract logic.
* `nsdb` is still a CLI/frontdoor over YIR metadata inspection.
* `nuisc` still owns much of the practical compiler/linker implementation
  surface.

The important alpha rule is that new linker/debugger features should be shaped
as future core capability data first and CLI output second.

## Related

Tool-local boundary notes:

* [tools/nsld/README.md](../../tools/nsld/README.md)
* [tools/nsdb/README.md](../../tools/nsdb/README.md)
