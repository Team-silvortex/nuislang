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
`value_snapshot_*` fields now report the current typed snapshot reference:
metadata-only for payload handoff records and opaque-payload for runtime
payload descriptors.
`value_content_*` fields expose metadata summaries today and mark opaque
runtime payloads with readable paths as safe file summaries until a dedicated
decoder exists.
`value_decoder_*` fields report the selected decoder registry entry, currently
using opaque summary decoders for Metal, SPIR-V, and CoreML-style payloads.
They also expose capability, detail level, and whether the decoder read a file
summary so callers can distinguish semantic metadata from opaque file metadata.
Registered opaque decoders now perform a lightweight format probe, such as
`MTLB` for Metal libraries or the SPIR-V magic word, before deeper decoding
exists.
The decoder registry is spec-driven internally, so new payload families can add
their decoder id, status, and optional magic probe without adding scattered
format-specific branches.
Artifact outputs may also provide `nuis.nsdb.payload-decoders.toml` with
`[[decoders]]` records for experimental payload families. The first supported
top-level fields are `protocol = "nuis-nsdb-payload-decoders-v1"` and
`schema = "nsdb-payload-decoder-manifest-v1"`. The first supported
record fields are `payload_format`, `decoder_id`, `magic_label`, and
`magic_ascii` or `magic_hex`. Hex magic accepts spaces or underscores between
byte pairs for readability.
External specs may also declare `decoder_capability` and `decoder_detail_level`;
when omitted they default to `opaque-file-summary` and `file-header`.
Nuis artifact-doctor and project-status/project-doctor surfaces mirror the
generated manifest status, record count, and first diagnostic so broken decoder
registries are visible before entering `nsdb`.
Replay checkpoints include `value_decoder_manifest_*` diagnostics so malformed
external specs, such as invalid hex magic, are visible instead of being silently
ignored.
`inspect` also reports a manifest-level summary with availability, record
counts, invalid record counts, and the first diagnostic so bad decoder specs can
be spotted before building a replay plan.
It also lists `payload_decoder_manifest_record` entries so each external
decoder declaration has its own validity and diagnostic status.
Unsupported protocol/schema values are reported in inspect summaries without
blocking built-in decoder fallback.
`run-artifact`/artifact runtime trace persistence now emits a standard decoder
manifest for observed backend payload formats, giving nsdb a generated external
decoder registry before format-specific decoders exist.
