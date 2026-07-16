# Nsdb YIR Debugger Frontdoor

`Nsdb` is the Nuis debugger frontdoor for YIR-layer debugging.

It is intentionally not an `lldb` clone. Native debuggers can still attach to
the host executable shell when the final binary is Mach-O, PE, or ELF, but that
view is expected to be low-level and incomplete for Nuis semantics.

Longer-term, `Nsdb` should be read as a CLI adapter over a future reusable
YIR-debug core / galaxy capability boundary, not as a CLI-only tool. See
[toolchain-galaxy-core-boundary.md](toolchain-galaxy-core-boundary.md).

`Nsdb` should instead consume the metadata that `Nsld` organizes:

* domain units
* clock protocol edges
* deterministic data segments
* artifact lowering units
* lowering IR sidecars

## Current Commands

```sh
cargo run -p nsdb -- status
cargo run -p nsdb -- inspect <artifact-output-dir>
cargo run -p nsdb -- inspect <artifact-output-dir> --json
cargo run -p nsdb -- inspect <artifact-output-dir> --event-status blocked
cargo run -p nsdb -- inspect <artifact-output-dir> --event-phase device-dispatch
cargo run -p nsdb -- inspect <artifact-output-dir> --trace-id payload-trace:...
cargo run -p nsdb -- events <artifact-output-dir>
cargo run -p nsdb -- events <artifact-output-dir> --json --event-status ready
cargo run -p nsdb -- replay-plan <artifact-output-dir>
cargo run -p nsdb -- replay-plan <artifact-output-dir> --json --event-status blocked
```

When given an output directory, `Nsdb` resolves
`nuis.build.manifest.toml` inside that directory.

## Current Debug Model

Current `Nsdb` output is an inspectable metadata view, not an interactive
debugger yet.

It reports:

* `debug_model = yir-metadata`
* `native_debugger_visibility = host-shell-only`
* `nsdb_visibility = domains+clock+segments+lowering-units`
* `sidecar_count`
* `sidecars`
* `payload_execution_handoff_available`
* `payload_execution_handoff_protocol`
* `payload_execution_handoff_debugger_contract`
* `payload_execution_handoff_status`
* `payload_execution_handoff_record_count`
* `payload_execution_handoff_first_trace_id`
* `payload_execution_handoff_first_entry_symbol`
* `payload_execution_handoff_first_execution_phase`
* `payload_execution_event_filter_active`
* `payload_execution_event_filter_status`
* `payload_execution_event_filter_phase`
* `payload_execution_event_filter_trace_id`
* `payload_execution_event_count`
* `payload_execution_events`
* `replay_protocol = nsdb-payload-execution-replay-plan-v1` for
  `replay-plan`
* `replay_checkpoint_count`
* `replayable_checkpoint_count`
* `replay_checkpoints`
* `frame_id`, `slot_scope`, and `value_state_status` inside each replay
  checkpoint
* `value_sample_contract`, `value_sample_ref`, and `value_sample_source` inside
  each replay checkpoint
* `value_sample_resolution_status` and `value_sample_resolution_detail` inside
  each replay checkpoint
* `debug_readiness = yir-debug-ready` when the linker graph, clock protocol,
  hetero calculate plan, lowering units, referenced lowering IR sidecars, and
  persisted payload execution handoff metadata are all readable

`Nsdb` can now expose sidecar capability metadata such as the owning Nustar,
frontend IR, native IR, backend lowering model, validation contracts, and entry
symbol. The same view is used across current heterogeneous domains:

* shader sidecars can describe
  `shader-nustar -> nuis-yir.shader -> msl2.4` plus Metal/Vulkan/DirectX style
  dispatch and resource lowering contracts
* kernel sidecars can describe
  `kernel-nustar -> nuis-yir.kernel -> coreml-program` or SPIR-V/host-SIMD
  tensor dispatch contracts
* network sidecars can describe
  `network-nustar -> nuis-yir.network -> foundation-url-request`,
  `posix-socket`, or `winsock-overlapped` transport lowering contracts

This means `Nsdb` is pointed at the right layer and can see real lowering
capability metadata, but source-level stepping, breakpoints, value inspection,
and replay still need dedicated debug sidecars.

`Nsdb` also consumes `nuis.nsdb.payload-execution-handoff.toml` from the
artifact output directory. The current handoff consumer validates
`nuis-nsdb-payload-execution-handoff-v1` and
`nsdb-yir-payload-execution-trace-v1`, then exposes each `[[records]]` row as a
payload execution event with trace id, status, execution phase, target, entry
symbol, entry kind, entry section id, first blocker, and next action.
`inspect` and the event-focused `events` command can filter those events with
`--event-status`, `--event-phase`, and `--trace-id`; filters are ANDed when
multiple flags are present. This is still an inspect-time event view rather
than full replay, but it proves that `run-artifact` metadata can cross into the
debugger frontdoor without asking `nsdb` to rerun the host probe.

`replay-plan` maps the filtered payload execution events into read-only
checkpoints. `container-loader-handoff` becomes a loader checkpoint,
`device-dispatch` becomes a device-dispatch checkpoint, and blocked events
carry their first blocker into the plan. Each checkpoint also exposes a stable
`frame_id`, `slot_scope`, and `value_state_status` so nsdb can attach future
YIR frame/value samples without changing the checkpoint identity. The
`value_sample_*` fields are references into later payload-execution or
heterogeneous runtime trace records, not inline values. This keeps replay-plan
as a deterministic debug plan while leaving actual value materialization to the
runtime/device trace resolver. `value_sample_resolution_*` reports whether the
current nsdb metadata can resolve that reference through payload handoff
metadata, a visible domain, or a readable sidecar. This is not execution or
time-travel debugging yet; it is the stable checkpoint skeleton that later YIR
frame/value state can attach to.

`run-artifact` persists the device/runtime sample source as
`nuis.nsdb.hetero-runtime-trace.toml` using
`nuis-nsdb-hetero-runtime-trace-v1` and
`nsdb-yir-hetero-runtime-trace-v1`. `Nsdb` inspect reads this file as
`hetero_runtime_trace_*` metadata, and replay planning uses its records before
falling back to sidecar/domain metadata. The trace still carries metadata
references rather than inline checkpoint values.

Replay checkpoints now expose `value_sample_materialization_*` plus
`value_sample_payload_format`, `value_sample_payload_path`, and
`value_sample_bridge_stub_path`. These fields materialize a stable sample
descriptor from the hetero trace record; they do not decode the payload into
typed YIR values yet.

Replay checkpoints also expose `value_slot_id`, `value_slot_scope`,
`value_schema_contract`, `value_schema_status`, and `value_schema_hint`. These
fields give future decoders a stable typed-slot target while keeping opaque
runtime payloads opaque until a real decoder is available.

`value_snapshot_*` fields provide the current typed snapshot reference. Metadata
snapshots are marked as metadata-only, while runtime payloads are represented as
opaque payload snapshots until a concrete payload decoder exists.

`value_content_*` fields expose the first concrete content layer. Payload
execution metadata can produce a readable summary immediately; opaque runtime
payloads produce a safe file summary when the payload path is readable, and
remain marked as awaiting a dedicated decoder for format-specific contents.
`value_decoder_*` identifies the decoder selected by the registry; current
Metal/SPIR-V/CoreML entries are intentionally opaque summary decoders. Decoder
capability and detail-level fields make that boundary explicit for replay tools.
Registered opaque decoders also expose a lightweight format probe status so
replay can tell whether a payload file matches the expected container header
before full format-specific decoding exists.
The current registry keeps decoder ids, statuses, and optional magic probes in
one payload-family spec rather than scattering per-format decisions through the
replay planner.
Artifact outputs can extend that registry with
`nuis.nsdb.payload-decoders.toml`; its initial `[[decoders]]` records support
`payload_format`, `decoder_id`, `magic_label`, ASCII magic probes, and hex
magic probes for binary payload headers.
The manifest itself carries `protocol = "nuis-nsdb-payload-decoders-v1"` and
`schema = "nsdb-payload-decoder-manifest-v1"` so nsdb can report unsupported
future versions while still falling back to built-in decoders.
Artifact runtime trace persistence emits a generated decoder manifest for
observed backend payload formats, so inspect/replay can consume a produced
registry instead of relying only on hand-authored manifests.
Nuis artifact-doctor and project-status/project-doctor mirror that generated
manifest summary, including status, record counts, and the first diagnostic,
before control moves into `nsdb`.
External decoder records may also declare `decoder_capability` and
`decoder_detail_level`, allowing experimental payload families to advertise a
more specific interpretation boundary while still remaining replay-safe.
Replay checkpoints surface `value_decoder_manifest_*` diagnostics so invalid
external specs can be inspected without changing the replay planner.
The inspect report also includes `payload_decoder_manifest_*` summary fields to
catch malformed decoder manifests before drilling into checkpoint replay.
Per-record manifest diagnostics are exposed as
`payload_decoder_manifest_records`, preserving the aggregate summary while
making multi-decoder manifests inspectable.

## Relationship To Nsld

`Nsld` freezes and validates the linker-facing contract graph.

`Nsdb` consumes that graph to answer debugger questions at the semantic layer:

```text
Nsld link metadata
  -> domain / clock / segment / lowering-unit map
  -> Nsdb YIR debug view
  -> future stepping / replay / value inspection
```

The debugger should not depend on LLDB being able to reconstruct Nuis
semantics from lowered LLVM code. LLDB remains useful for host crashes and
native wrapper failures; `Nsdb` is responsible for the YIR-level truth.
