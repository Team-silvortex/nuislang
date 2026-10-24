# Native Return Storage Projected Payload V1

This bounded local CPU follow-up extends the
[mixed-origin payload witnesses](nuis-native-return-effect-storage-mixed-payload-v1.md).
It starts from Git beta-0.17.5 at c3e6a14f without a version bump. It exercises
the existing general lowering path, not a backend-specific projection shortcut.

## Call Then Project

The [Nuis helper](../../tools/nuisc/tests/native_application_bridge/state_loop_return_projected_helpers.ns)
returns a complete State after observable work and a checked division. The
[source adapter](../../tools/nuisc/tests/native_application_bridge/state_loop_return_projected_cases.rs)
uses it directly beneath four returned-field expressions, without introducing
an explicit source-level temporary:

```nuis
marker: field_marker(project_record(frozen, project_marker_arg(1), 1).marker),
result: field_result(project_record(carry, project_result_arg(2), 1).result != true),
```

The other two occurrences project gain and total. Each argument helper has its
own effect marker. Successful evaluation of a selected return enters the record
callee exactly four times: once per source occurrence, not once per record field and not
once for all occurrences combined. The gain projection can fail during argument
evaluation or inside the callee; neither the projection nor its enclosing field
helper may bypass that failure.

Source field order is marker/result/gain/total, distinct from record layout.
The result inverts the live boolean while the later total still reads its
original value. Older/newer snapshots remain immutable. Sign-bit changes and
the additional total delta distinguish actual record-call results from bypassed
calls. The semantic oracle models these operations independently, with eight
hand-derived totals and exact effect traces.

## Authority And Failure Gates

[Compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_projected_payload_tests.rs)
require separate grants for the record function and every argument helper;
effectful helpers remain absent from the pure catalog. Seven inherited
field-order/publication mutations and six projected-call mutations must be
rejected even when still type-valid. The new mutations change record origin or
argument-helper identity, duplicate a call, bypass either call layer, or remove
the selected callee failure check.

Four leading/trailing-step and mirrored variants exercise first-parent and
post-backedge returns. There are 23 failure categories: seven earlier
bound/construction failures, eight earlier payload failures and eight new
projected-argument/callee failures. Unselected and zero-width paths skip all
poisoned return expressions. The new argument failures stop before the argument
helper; callee failures include its entry effect but not its post-check effect.

Independent/shared normal/reversed YIR must match the oracle. The reduction is
four logical YIR carry words, not a measured memory or speed improvement.
Independent/shared read sets, bounded candidates, separate pending identity,
catalog authority and exact independent fallback remain unchanged. Loop-work
and helper-entry limits stay at 18 and 192.

## Native And Delivery Gates

[Reference/native tests](../../tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds.rs)
compare raw bool/i64/f32/f64 words, including signed zero, subnormals and NaNs.
After three accepted callbacks, a reference failure must retain the last
complete State, become Faulted, reject further events and refuse completion.
Native probes require the exact flushed effect prefix and previously accepted
states; they do not inspect terminated-process memory or roll back effects.

Exact-budget and one-less reservation probes retain the existing 4/8 and
6/7 or 8/9 limits. An insufficient reservation fails before the selected child
and projected-call effects; early return does not refund entered work.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_sibling_bounds.rs)
covers fresh build, cache hit and two source-free materializations, with one
representative per family. It removes source, manifest, project cache and
original output before restoring. Each image checks healthy sessions, every
failure category and a new healthy session, together with container/binary,
host-object, envelope, lifecycle and packaging identities. CLI failures assert
no new accepted State or completion; exact flushed prefixes belong to the
separate native probe.

## Remaining Frontier

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. Next put effects or checked faults inside an unprojected field of the
callee's returned literal. The current callee check precedes that literal;
it does not by itself prove that projecting one field preserves evaluation of
all other literal fields. Keep full callee evaluation, source order, once-only
occurrences and complete publication explicit rather than inferring them.
The [unprojected-field follow-up](nuis-native-return-effect-storage-unprojected-payload-v1.md)
records that subsequent gate without broadening this receipt's original evidence.

Resource/I/O/async capture, compound boolean comparisons against true, stable
ABI, GPU/cross-platform parity, full heterogeneous closure and self-hosting
remain separate gates. This is not full-workspace acceptance.

## Validation

Bounded local M2 CPU acceptance passed 72 distinct tests, including eight new
tests. Repeated focused runs are not counted twice:

- Compiler: four projected-payload tests passed in 252.28 seconds; four adjacent
  mixed-payload tests passed in 144.68 seconds.
- Native/reference: six projected/mixed tests passed in 439.81 seconds,
  exercising all four source variants and unchanged reservation limits.
- CLI delivery: two tests passed in 919.68 seconds, covering eight images.
  This selects one representative per family, not all four source variants.
- Host fault-state controls: three passed; repository maintenance: 24 passed.
- File-line limits: two passed; status tensor: 27 passed.

The ordinary CLI build, formatting, whitespace, UTF-8 across 5,621 files,
4,128 local documentation links and locked offline metadata for 29 workspace
packages passed. All 3,558 drift checks and 24,404 required patterns matched.
Complete predecessor hashes and inert historical commands passed validation;
the rebuilt CLI matches all six source fields. Task-card readiness, coverage,
hierarchy and lineage checks are clean; the coordinate remains active/99.
An intermediate maintenance failure for the initially unlinked receipt was
resolved by adding its proof-index link; the complete maintenance rerun passed.

Other historical fixture families were not rerun here. No full-workspace,
GPU/cross-platform, physical-performance or self-hosting completion is asserted.

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effect_return_storage_projected_payload effect_return_storage_mixed_payload --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- projected_payload mixed_payload --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- projected_payload mixed_payload --test-threads=1 --nocapture
```

The six tensor fields retain complete predecessor values and SHA-256 hashes;
historical commands stay inert in a quoted here-document.

| Predecessor Field | Complete SHA-256 |
| --- | --- |
| `evidence` | `07768eee21fd298bdc02f6cd3c8a28bb6748806da8d9b74eadeae4effc68e035` |
| `next_step` | `c3525f9a82590688ef7fbb73ca251c663cab240e9aa56dc08cd45b7ae33caba8` |
| `blocker` | `86a6fd3dcd73b3f1438171a83ef771b7a01dab28996dfdc83c9746095c86c3b7` |
| `next_action` | `d31fdefab245e6e3a8b257e723484c5733918fea5c037133487a9c46d212089b` |
| `validation_command` | `455e520f63519bb73323fdaec4a626a9d6f835001c98dbaf2ddcd67c955de5ed` |
| `expected_artifact` | `0f026eb98e76bc3b7c575a097363c15210d40915657aaffd233afe6c7d711f18` |
