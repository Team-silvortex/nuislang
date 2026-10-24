# Native Return Storage Unprojected Payload V1

This bounded local CPU follow-up extends the
[projected-call witnesses](nuis-native-return-effect-storage-projected-payload-v1.md).
It starts from Git beta-0.17.5 at c3e6a14f without a version bump. It exercises
complete record construction through the existing general lowering path.

## Complete Callee Fields

The [Nuis helper](../../tools/nuisc/tests/native_application_bridge/state_loop_return_unprojected_helpers.ns)
performs observable work in every field of its returned literal:

```nuis
return State {
    marker: record_marker(-seed.marker, before),
    result: record_result(seed.result),
    gain: record_gain(-seed.gain),
    total: record_total(seed.total + delta + checked, after)
};
```

The caller directly projects `.marker`, `.result`, `.gain` or `.total` from
separate calls, preserving the previous mixed-origin return construction.
Projection must wait for every field, including unused fields after the desired
one. In the `.gain` call, the marker helper can fail before gain is calculated;
the total helper can fail after gain is calculated. Neither failure may reach
the enclosing `field_gain`, start the next record call, or publish a new State.
The marker helper's checked division has an unused result, so dead-value
elimination must not remove its failure behavior.

[Source adaptation and the semantic oracle](../../tools/nuisc/tests/native_application_bridge/state_loop_return_unprojected_cases.rs)
remain separate from lowering. Successful callee work has the exact prefix
`986, 987, 991, 992, 993, 994, 995, 996`, repeated once per reached source call.
The independent projected-value model supplies raw fields and totals; explicit
effect expectations account for all literal fields in source order, not record
layout order. Eight new first-parent/post-backedge failures extend the family
to 31 categories, preserving the previous 23 argument/callee/bound failures.
Unselected and zero-width paths skip poisoned return work.

## Authority And Negative Controls

[Compiler tests](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_unprojected_payload_tests.rs)
require the completed transitive callee closure. Removing any field helper from
the source module prevents both the record helper and its caller from entering
the admitted catalog. Effectful leaves remain absent from the pure catalog.
This recollects authority from source, rather than fabricating an inconsistent
catalog by deleting a leaf while retaining its already-admitted parents.

Fifteen type-valid caller mutations are rejected: the previous thirteen plus
bypassing either new field-failure guard. The caller certificate preserves
callee identity and arguments; it is not a proof of arbitrary callee-body edits.
Four separate executable negative controls prune the marker or total helper,
reorder fields, or repeat the total helper. These are intentionally different,
valid programs: their final healthy integer agrees, but their effect traces
must differ from the oracle. Pruning a checked field also turns its selected
failure into success, proving the failure witness detects that mistake.

Independent/shared normal/reversed YIR is compared across four leading/trailing
and mirrored variants. Independent/shared read sets, immutable snapshots,
bounded candidates, separate pending identity and exact independent fallback
remain required. Four fewer logical YIR carry words are not a physical-memory
or speed measurement. Loop-work/helper-entry limits remain 18/192.

## Native And Delivery Gates

[Reference/native coverage](../../tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds.rs)
checks raw bool/i64/f32/f64 state, including signed zero, subnormals and NaNs.
Reference failures after three accepted callbacks retain the last complete
State, become Faulted, reject subsequent events and refuse successful completion.
Native probes compare accepted states and exact flushed failure prefixes, not
terminated-process memory or rollback of already emitted effects.

Reservation probes retain the exact required budget and one-less controls:
4/8 for first-parent returns and 6/7 or 8/9 after the backedge. Reservation
failure precedes selected child work and callee-field effects; early return
does not refund entered work.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_sibling_bounds.rs)
uses one representative for fresh build, cache hit and two source-free
materializations. Source, manifest, project cache and original output are
removed before restoration. Every image runs healthy sessions, all 31 failures
and a new healthy session, checking binary/container, host-object, envelope,
lifecycle and packaging identities. CLI failures require no new accepted State
or completion; exact flushed prefixes belong to the separate native probes.

## Remaining Frontier

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. Next exercise conditional and short-circuit work inside an unprojected
field: complete construction must retain reached work without speculating
effects or checked failures from an unselected branch. Current whole-return
skip controls do not establish that internal branch boundary.

The [conditional-field follow-up](nuis-native-return-effect-storage-conditional-payload-v1.md)
now exercises that boundary and repairs eager RHS work in scalar control.
Its acceptance and remaining typed-binding limitation are recorded separately;
the historical validation below is unchanged.

General resource/I/O/async captures, compound boolean comparisons against true,
stable ABI, GPU/cross-platform parity, full heterogeneous closure and self-hosting
remain separate gates. This is not full-workspace acceptance.

## Validation

Bounded local M2 CPU acceptance passed 68 distinct tests, including nine new
tests. Repeated checks are not counted twice:

- Compiler: seven focused checks passed in 6.60 seconds, comprising four new
  admission/oracle/negative-control tests and three adjacent projected-call
  controls. The new independent/shared execution test passed in 382.83 seconds.
- Native/reference: three tests passed in 264.16 seconds, exercising all four
  source variants and the unchanged reservation limits.
- CLI delivery: one test passed in 704.42 seconds, covering four images and
  all 31 failure categories. This selects one representative, not all variants.
- Host fault-state controls: three passed; repository maintenance: 24 passed.
- File-line limits: two passed; status tensor: 27 passed.

The ordinary CLI build, formatting, whitespace, UTF-8 across 5,626 files,
4,140 local documentation links and locked offline metadata for 29 workspace
packages passed. All 3,568 drift checks and 24,464 required patterns matched.
Complete predecessor hashes and inert historical commands passed validation;
the rebuilt CLI matches all six source fields. Task-card readiness, coverage,
hierarchy and lineage checks are clean; the coordinate remains active/99.

Historical native/delivery families and the full workspace were not rerun.
No GPU/cross-platform, physical-performance or self-hosting completion is asserted.

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effect_return_storage_unprojected_payload --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effect_return_storage_projected_payload --skip matches_independent_execution_and_carry_width --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- unprojected_payload --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- unprojected_payload --test-threads=1 --nocapture
```

All six complete predecessor fields and their hashes remain in the tensor;
historical commands stay inert in a quoted here-document.

| Predecessor Field | Complete SHA-256 |
| --- | --- |
| `evidence` | `636fb1f41a8e6a211c085a263e8813970408fe70eff4db0b5b53fc09afb17bc2` |
| `next_step` | `1afe5c1367bfd8bb1c0b5809ccd732ade1b1d8f38c01a28131f276121c6aaade` |
| `blocker` | `14d11d397db17403232eb5d4f75e954f6025f4a0bf9895d9f4c4fa15c2ef124d` |
| `next_action` | `5a41b96614b9450de1836baeb8e45a6fcdbc5ac7bfd684088f7f2735e5728e3e` |
| `validation_command` | `d48d602ec67e2d0efeb5950d9ce6bccd18e0b9873382bf9267c0aae011a19e53` |
| `expected_artifact` | `512615d240e356e832c25ffadd44f46e386de32a4d8f354623b1e82198a6914b` |
