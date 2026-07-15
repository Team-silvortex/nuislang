# Nsdb

`nsdb` is the Nuis YIR-level debugger metadata front-door.

It is currently a compact CLI prototype over linker/YIR metadata inspection.
Native debuggers may still attach to the host shell binary, but `nsdb` owns the
Nuis semantic debug view: YIR domains, clock edges, data segments, lowering
units, sidecars, persisted payload execution handoff metadata, and future
GLM/runtime state views.

## Boundary

The long-term shape is:

```text
nsdb core capability -> CLI adapter
```

The CLI is a front door, not the durable debugger protocol.

## Core Responsibilities

Future `nsdb-core` or equivalent galaxy-style capability should own:

* YIR debug metadata loading and validation
* timestamped event and clock-edge views
* YIR frame, slot, symbol, and value mapping
* heterogeneous node traces and semantic replay queries
* persisted payload execution handoff consumption
* GLM and memory-state inspection surfaces when those contracts are ready

The CLI should format those capabilities for humans and scripts, but it should
not be the only way to access them.

## Current Alpha Rule

When adding debugger behavior, prefer structured metadata and query-shaped
helpers before terminal formatting. This keeps `nsdb` usable later from IDE
surfaces, `yalivia`, future Nuis OS debugging shells, and automated verifier
flows without pretending that command-line text is the protocol.

Current `inspect` output consumes `nuis.nsdb.payload-execution-handoff.toml`
when it is present beside `nuis.build.manifest.toml`. The fields are exposed as
`payload_execution_handoff_*` JSON/text metadata plus
`payload_execution_events`, so container-loader and future device-dispatch
handoff traces can move from `run-artifact` into the debugger layer.
Use `--event-status`, `--event-phase`, and `--trace-id` on `inspect` or
`events` to narrow that event view without changing the persisted handoff file.
`events` is the focused surface for scripts that only need payload execution
event metadata.
`replay-plan` turns the same filtered events into read-only
`nsdb-payload-execution-replay-plan-v1` checkpoints with stable frame/value
anchors, `nsdb-yir-value-sample-ref-v1` sample references, and metadata-backed
sample resolution status; it does not execute, inspect real values, or
time-travel yet.
`run-artifact` also writes `nuis.nsdb.hetero-runtime-trace.toml`; `nsdb`
consumes that trace file for replay-plan sample resolution rather than
embedding runtime values in replay plans.
Replay checkpoints surface materialized sample descriptors via
`value_sample_materialization_*`, payload format/path, and bridge stub path;
typed value decoding is still a later layer.
They also expose `value_slot_*` and `value_schema_*` fields so future decoders
can attach typed YIR value snapshots without changing checkpoint identity.
