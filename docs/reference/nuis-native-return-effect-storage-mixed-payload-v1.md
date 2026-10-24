# Native Return Storage Mixed Payload V1

This bounded local CPU follow-up extends the
[field-payload witnesses](nuis-native-return-effect-storage-field-payload-v1.md).
It starts from Git beta-0.17.5 at c3e6a14f without a version bump.

## Boolean Projection Admission

The mixed fixture exposed an admission gap: loop temporary validation accepted
boolean variables and calls but rejected `carry.result != true`. A typed record
field was missing from the boolean-atom profile. The
[validator](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs)
now recognizes projection chains rooted in an existing atom, after the full
expression has passed type, read-readiness, depth and call-authority checks.
This does not admit compound boolean trees compared against true, grant a
missing callee, or permit reading an unavailable sibling carry.

[Focused controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/bool_projection_tests.rs)
cover equality/inequality in both operand orders, direct/nested fields, permitted
self-reads, unavailable/available sibling reads, record-valued call authority,
wrong types, missing names/fields and the retained compound-expression boundary.
Nested and call-rooted projections here have admission evidence only; the full
mixed-payload execution chain below uses fields rooted in local records.

## Mixed Origins And Publication

The [source adapter and independent oracle](../../tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds_cases.rs)
construct a single State from the mutable live carry and both immutable snapshot
epochs. Its source field order is marker/result/gain/total, not layout order.
The result field inverts the live boolean; the later total calculation must
still read the original boolean through the
[Nuis bias helper](../../tools/nuisc/tests/native_application_bridge/state_loop_return_mixed_helpers.ns).
Prematurely publishing the result field changes the bias by 1,000.
The integer seed combines live total, 100 times older total and 10 times newer
total. The two floating fields come from opposite snapshot epochs, with their
roles reversed in the second child. Wrong-origin reads therefore have explicit
value witnesses, rather than merely checking successful execution.

Four leading/trailing-step and mirrored variants cover first-parent and
post-backedge returns. Eight hand-derived totals independently check the
oracle. Successful field work ends with exactly
`980, 981, 982, 984, 970, 971, 972, 973`. Eight selected argument/callee failure
modes stop before complete publication; seven earlier bound/construction fault
categories stay distinct. Unselected and zero-width paths must skip poisoned
return expressions.

[Compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_mixed_payload_tests.rs)
reuse the seven field-order/publication mutations and add three type-valid
changes: replacing the live total with a snapshot, forwarding the new boolean
into a later self-read, and confusing older/newer weighted totals. The structural
certificate must reject all ten classes. Every reached helper needs its own
catalog grant; both snapshots remain ineligible as mutable return storage.
Independent/shared read sets, bounded candidates, separate pending identity
and exact independent fallback remain required.

Independent/shared normal/reversed YIR is compared with the semantic oracle.
The observed reduction is four logical YIR carry words, not measured RAM or
speed. No loop-work or helper-entry limits are raised: they remain 18 and 192.

## Native And Delivery Gates

[Reference/native coverage](../../tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds.rs)
compares actual State and raw bool/i64/f32/f64 words, including signed zero,
subnormals and NaNs. Reference failures after three accepted callbacks must
retain the last complete State, enter Faulted, reject further events and refuse
successful completion. Native probes compare accepted states and the exact
flushed failure-effect prefix; they do not inspect a terminated process's memory
or undo effects already emitted before failure.

Reservation probes use exactly the required budget and one less: 4/8 for
first-parent returns, 6/7 or 8/9 after the backedge. An insufficient reservation
fails before the selected child body and return-field effects. Early returns
do not refund entered work, and skipped loops reserve nothing.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_sibling_bounds.rs)
selects one representative per family for fresh build, cache hit and two
source-free materializations after deleting the source, manifest, project cache
and original output. Each image checks healthy sessions, all fifteen failure
categories and recovery through a new healthy session. Container/binary,
host-object, envelope, lifecycle and packaging identities remain checked.
CLI fault checks require no new accepted State or completion marker; exact
flushed failure prefixes belong to the separate native probe.

## Remaining Frontier

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. Next exercise an effectful record-valued helper as the base of a
returned field projection, with checked argument/callee failures and observable
source occurrences. Do not infer once-only projected-call execution from local
record projection or from admission-only controls. The
[projected-call follow-up](nuis-native-return-effect-storage-projected-payload-v1.md)
records the subsequent execution gate without broadening this receipt's original evidence.
Preserve ordered evaluation,
complete publication, call/read authority and exact fallback.

Resource/I/O/async capture, compound boolean comparisons against true, stable
ABI, GPU/cross-platform parity, full heterogeneous closure and self-hosting
remain separate gates. This is not full-workspace acceptance.

## Validation

Bounded local M2 CPU acceptance passed 78 distinct tests, including ten new
tests. The initial rejected fixture and interrupted run are not acceptance
evidence, and repeated runs are not counted twice:

- Compiler: ten mixed/field-payload and boolean-projection tests passed in
  338.83 seconds; four adjacent boolean-carry/word-temporary controls also passed.
- Native/reference: six mixed/field-payload tests passed in 348.21 seconds,
  exercising all four source variants and the unchanged reservation limits.
- CLI delivery: two tests passed in 646.02 seconds, covering eight images.
  This selects one representative per family, not all four source variants.
- Host fault-state controls: three passed; repository maintenance: 24 passed.
- File-line limits: two passed; status tensor: 27 passed.

The ordinary CLI build, formatting, whitespace, UTF-8 across 5,616 files,
4,116 local documentation links and locked offline metadata for 29 workspace
packages passed. All 3,548 drift checks and 24,343 required patterns matched.
Complete predecessor hashes and inert historical commands passed validation;
the rebuilt CLI matches all six source fields. Task-card readiness, coverage,
hierarchy and lineage checks are clean; the coordinate remains active/99.

Other historical fixture families were not rerun here. No full-workspace,
GPU/cross-platform, physical-performance or self-hosting completion is asserted.

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- loop_bool_projections bool_carries loop_word_temporaries effect_return_storage_mixed_payload effect_return_storage_field_payload --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_loop_sibling_bounds::mixed_payload_ state_loop_sibling_bounds::field_payload_ --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- effect_calls::state_loop_sibling_bounds::mixed_payload_ effect_calls::state_loop_sibling_bounds::field_payload_ --test-threads=1 --nocapture
```

The six tensor fields retain complete predecessor values and SHA-256 hashes;
historical commands stay inert in a quoted here-document.

| Predecessor Field | Complete SHA-256 |
| --- | --- |
| `evidence` | `dcd48f385cc690eb8aa8a5d34d97b7e71c3f997a44f5fd0444fb8d7c5598ad99` |
| `next_step` | `0a3e14c2f11d944bbc797238044b68b52c65a94e8b0e532eac532bacf6a6501f` |
| `blocker` | `a8d1ac0bc81006942e77e2a13e5156e1a5cebb5078c11b37ad8131cb90f02547` |
| `next_action` | `f394c95e8ee578dcd27d10cbfc3ffd200e0f9d09393e8292e9631057031f4a48` |
| `validation_command` | `e64f523fcf3438652d4eadbfaf054172df7a9745b20db0af6d56cc3ceb8c7d03` |
| `expected_artifact` | `2f4c8334db0e9bc43c268e511e4ed6095acc948ef05cd90332164977fd538b84` |
