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
cargo run -p nsdb -- replay <artifact-output-dir>
cargo run -p nsdb -- replay <artifact-output-dir> --json
```

When given an output directory, `Nsdb` resolves
`nuis.build.manifest.toml` inside that directory.

## Current Debug Model

Current `Nsdb` output is an inspectable metadata and deterministic replay
transcript view, not an interactive debugger yet.

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
* `payload_execution_event_query_contract =
  nsdb-payload-execution-event-query-v1` for `events`
* `payload_execution_event_source = payload-execution-handoff-events`
* `payload_execution_event_query_result_count`
* `payload_execution_events`
* `replay_protocol = nsdb-payload-execution-replay-plan-v1` for
  `replay-plan`
* `replay_event_query_contract = nsdb-payload-execution-event-query-v1`
* `replay_checkpoint_source = payload-execution-handoff-events`
* `replay_event_query_result_count`
* `replay_checkpoint_count`
* `replayable_checkpoint_count`
* `replay_checkpoints`
* `debugger_transcript_contract = nsdb-yir-replay-transcript-v1` for `replay`
* `debugger_transcript_status`
* `debugger_transcript_checkpoint_count`
* `debugger_transcript_replayed_checkpoint_count`
* `debugger_transcript_frames`
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
multiple flags are present. `events` reports
`nsdb-payload-execution-event-query-v1`, the handoff source protocol, the
debugger contract, and the filtered result count so scripts can treat the event
view as a stable query surface rather than an inspect-only printout.

`replay-plan` maps the filtered payload execution events into read-only
checkpoints. `container-loader-handoff` becomes a loader checkpoint,
`device-dispatch` becomes a device-dispatch checkpoint, and blocked events
carry their first blocker into the plan. Each checkpoint also exposes a stable
`frame_id`, `slot_scope`, and `value_state_status` so nsdb can attach future
YIR frame/value samples without changing the checkpoint identity. The
top-level `replay_checkpoint_source` explicitly states that checkpoints are
derived from `payload-execution-handoff-events`, keeping replay deterministic
and auditable for nsld/nsdb integration.
`value_sample_*` fields are references into later payload-execution or
heterogeneous runtime trace records, not inline values. This keeps replay-plan
as a deterministic debug plan while leaving actual value materialization to the
runtime/device trace resolver. `value_sample_resolution_*` reports whether the
current nsdb metadata can resolve that reference through payload handoff
metadata, a visible domain, or a readable sidecar. This is not execution or
time-travel debugging yet; it is the stable checkpoint skeleton that later YIR
frame/value state can attach to.

`replay` is the first consumer of that skeleton. It emits
`nsdb-yir-replay-transcript-v1` only after the complete filtered checkpoint set
is replayable and the heterogeneous execution closure is ready. Consumption is
all-or-nothing: a blocked checkpoint produces `transcript-blocked` and zero
consumed frames instead of a misleading partial replay. A ready transcript
preserves checkpoint order and exposes each consumed YIR frame, value slot,
snapshot summary, and next action. It is deterministic transcript consumption,
not native instruction execution or interactive stepping.

The first deterministic replay-control layer is
`nsdb-yir-replay-control-v1`:

```bash
nsdb replay <artifact-output-dir> --frame <index|frame-id> --json
nsdb replay <artifact-output-dir> --break-at <index|frame-id> --json
nsdb replay <artifact-output-dir> --break-phase <phase> --break-entry <symbol> --json
```

`--frame` consumes exactly the selected replayable YIR frame and reports
`frame-selected`. `--break-at` consumes the ordered prefix through the selected
frame and reports `breakpoint-hit`. The selectors are mutually exclusive and
match either the numeric checkpoint index or the exact stable `frame_id`.
Missing or ambiguous targets fail closed with zero consumed frames and an
explicit `replay-control:*` blocker. These are transcript-level breakpoint
semantics; they do not yet pause native execution or provide interactive
continue/step commands.

Typed predicates use `nsdb-yir-breakpoint-predicate-v1`. `--break-phase` and
`--break-entry` may be supplied independently or together; together they match
with AND semantics and stop at the first ordered frame satisfying both fields.
They are mutually exclusive with exact `--frame` and `--break-at` controls.
Every successful stop also emits `nsdb-yir-replay-resume-cursor-v1`. A cursor
records the stopped `after_frame_id` and, when another frame exists, its stable
`next_frame_id` and numeric index with `resume-ready`. A terminal stop reports
`end-of-transcript` and `resume_cursor_ready = false`. The cursor is currently
consumed by replay through a strict pair:

```bash
nsdb replay <artifact-output-dir> \
  --resume-after <stopped-frame-id> \
  --resume-next <next-frame-id> \
  --json
```

This input uses `nsdb-yir-replay-resume-input-v1`. Nsdb resolves the stopped
frame, verifies that the supplied next frame is its immediate ordered
successor, and only then consumes the suffix beginning at that next frame. A
missing half of the pair, unknown stopped frame, terminal stopped frame, or
mismatched next frame returns `cursor-rejected`, zero consumed frames, and an
explicit `replay-resume:*` blocker. Resume may be combined with an exact or
typed breakpoint to stop again later; `--frame` remains mutually exclusive
because single-frame inspection is not continuation.

A non-terminal stop may persist the validated cursor directly and load it in a
later invocation:

```bash
nsdb replay <artifact-output-dir> \
  --break-at <frame-id> \
  --save-cursor <cursor.toml> \
  --json

nsdb replay <artifact-output-dir> \
  --resume-cursor <cursor.toml> \
  --break-at <later-frame-id> \
  --json
```

The file uses `nsdb-yir-replay-cursor-record-v1` and records the transcript and
replay-source contracts, manifest, stopped frame, immediate next frame index,
and next frame id. Nsdb refuses to persist blocked or terminal cursors. Loading
fails closed on malformed or unknown fields, incompatible contracts, non-ready
status, or a different manifest. `--resume-cursor` is mutually exclusive with
manual resume fields and `--frame`, but may be combined with a later breakpoint.
Nuis mirrors this public artifact through `nuis-debugger-cursor-handoff-v1` as
path/readiness/status metadata in final-output and closure summaries. This is an
adapter boundary: Nuis does not import Nsdb implementation types, and absence
of an optional cursor does not block binary readiness. A ready mirror also
publishes a cursor-specific `next_command` through final-output and closure
summaries; unavailable or invalid mirrors publish no continuation command.
That command targets `nuis debug-resume`, whose frontdoor validates the handoff
again before launching Nsdb with structured arguments. The initial route
may continue through the remaining suffix or forward exact/typed breakpoint
controls and an optional cursor output. The official PixelMagic path now emits
data, kernel, and shader checkpoints: it stops and saves at the first frame,
resumes and replaces the cursor at the second, then consumes that replacement
and stops at the third exclusively through the Nuis frontdoor. Cursor-file
replacement now writes and syncs a same-directory temporary file, reloads it
through the public cursor validator, atomically renames it into place, and syncs
the containing directory on Unix. Validation failures preserve the previous
cursor and remove the temporary file. Each successful save also best-effort
updates the sibling
`nuis.nsdb.replay-cursor.lineage.toml` sidecar under
`nsdb-yir-replay-cursor-lineage-v1`. It retains at most eight entries containing
only a monotonic sequence, previous/current FNV-1a content hashes, and public
after/next frame ids. Existing lineage must parse, match the cursor path, and
continue the hash chain before replacement; a damaged sidecar is preserved and
does not invalidate the already-installed authoritative cursor.

Nuis consumes that sidecar through its own
`nuis-debugger-cursor-lineage-mirror-v1` artifact adapter rather than importing
Nsdb types. Final-output and closure summaries expose the source protocol,
path, readiness, status, bounded entry count, and latest cursor hash. Readiness
also requires the latest lineage hash to match the authoritative cursor bytes.
Missing lineage remains optional; stale or malformed lineage is reported as
`lineage-invalid` without blocking cursor continuation.

Invalid mirrors also expose a stable first-blocker code plus
`repair-cursor-lineage` and a concrete
`nuis debug-lineage-repair ... --json` command. This first-class Nuis frontdoor
validates the artifact and rejects missing lineage before dispatching structured
arguments through the same Nsdb executable boundary as `nuis debug-resume`.
Nsdb remains the repair owner and first validates the authoritative cursor. Healthy
lineage returns `already-ready` without mutation; invalid lineage is renamed to
a content-hash-qualified `.invalid-<hash>.toml` archive before a one-entry
lineage is rebuilt atomically from the current cursor. Repeating repair is
idempotent. Actual rebuilds also append an atomically validated, bounded audit
entry to `nuis.nsdb.replay-cursor.lineage-repairs.toml` under
`nsdb-yir-replay-cursor-lineage-repair-journal-v2`; healthy `already-ready`
probes do not modify this journal. Each entry records the archived path and
content hash plus the rebuilt cursor hash.

Nuis validates this journal independently through
`nuis-debugger-cursor-lineage-repair-mirror-v1`. Final-output and closure
summaries retain the journal status/count, latest mutation flag, archived
path/hash, and rebuilt hash after command stdout has disappeared. Readiness
requires the archived bytes to match their recorded hash and the latest rebuilt
hash to match the active lineage. The official three-domain smoke covers
stale-hash diagnosis, archive/rebuild, persistent repair evidence, restored
Nuis readiness, and continued cursor resume.

Before any lineage archive or rebuild, Nsdb preflights an existing repair
journal. An invalid journal is content-hash archived first; if no archive slot
can be reserved, the command fails while preserving the lineage bytes exactly.
The repair result exposes both the active journal path and any archived journal
path. After successful lineage recovery, a fresh validated journal is installed
and becomes visible to the next independent Nuis report.

When lineage is already healthy but the journal alone is invalid, the same
frontdoor performs a journal-only recovery. Nsdb emits
`repair-history-recovered`, preserves lineage bytes, and reports
`lineage_mutated = false` separately from `repair_journal_mutated = true`.
Ordinary healthy checks keep both flags false; lineage rebuilds set both true.

Replay source selection remains deterministic. Payload-execution handoff
events are preferred whenever present. When that list is empty, Nsdb projects
`hetero_runtime_trace.records` into ordered metadata/device-dispatch events.
`metadata-only` and `trace-ready` records become debugger checkpoints, while
device-dispatch replayability still depends on provider-sample validation.
This gives pre-final-output heterogeneous artifacts a real replay route without
inventing a fake native payload handoff.

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
