# Native Return Storage Field Payload V1

This bounded local CPU follow-up extends the
[payload-call witnesses](nuis-native-return-effect-storage-payload-calls-v1.md).
The work starts from Git beta-0.17.5 at c3e6a14f; it does not bump the version.

## Source Field Order

The [fixture and independent oracle](../../tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds_cases.rs)
construct the returned State directly inside the selected child-loop return.
The literal evaluates marker, result, gain, total, deliberately different from
the declaration/layout order result, total, gain, marker. The first three
[Nuis field helpers](../../tools/nuisc/tests/native_application_bridge/state_loop_return_field_helpers.ns)
print 980, 981 and 982 and preserve their f64/bool/f32 words. The total field
evaluates the existing argument helpers (970, 971), then field_total (972,
checked division, 973), producing the selected snapshot total plus ten.

Four leading/trailing-step and mirrored variants return either the newer or
older immutable snapshot, on the first parent iteration or after its backedge.
All successful literal evaluations must end with exactly
`980, 981, 982, 970, 971, 972, 973`. Eight argument/callee failure modes leave
the earlier fields observable but stop before 973. Seven existing bound and
snapshot-construction fault categories remain separate. Unselected branches
and zero-width children must not evaluate poisoned fields.

An initial native probe rejected an unselected literal-print grant: the test
adapter had retained the now-unused payload_finish helper and authorized all
fixture print nodes. The field variant now includes only the argument helpers
from that older helper source. The native policy's rejection remains intact;
this correction does not authorize effects outside the selected call closure.

## Complete Publication

[Compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_field_payload_tests.rs)
require every reached field/argument helper grant and keep both snapshots
read-only. Type-valid mutations reorder whole field pairs or nested arguments,
duplicate construction, bypass early field work or finalization, substitute an
older snapshot field or publish the pending flag before construction. The
structural certificate must reject them without relying only on type validity.

Independent/shared normal/reversed YIR is compared against the independent
oracle. The carry difference remains four logical YIR words; this is not a
physical-memory or speed measurement. Original/shared read sets, bounded
candidates, separate pending identity and exact independent fallback remain
required. No production call authority is widened by the fixture.

[Reference/native checks](../../tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds.rs)
cover complete raw State, including signed zero, subnormal and NaN words.
After three successful callbacks, a failed event must retain the actual last
accepted reference State, enter Faulted, reject later events and refuse
successful completion after close. Native checks compare the exact flushed
effect prefix and previously accepted State words; they do not inspect a
terminated process's memory or claim to roll back already emitted effects.

Loop-work and helper-entry limits remain 18 and 192. Exact reached reservations
are 4/8 for first-parent exits and 6/7 or 8/9 after the backedge. One unit less
must fail before the chosen child body and all field effects. An early return
does not refund the entered loop reservation; skipped loops reserve nothing.

## Source-Free Delivery

[CLI coverage](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_sibling_bounds.rs)
selects one representative for fresh build, cache hit and two materializations
after deleting source, manifest, project cache and original output. Every
image runs healthy callbacks, all fifteen fault categories and a new healthy
session afterward. Failed callbacks publish no new State or completion marker.
Container/binary, host-object, envelope, lifecycle and packaging identities
remain checked. The CLI fault checks do not assert flushed stdout; the native
probe separately owns exact failure-effect-prefix evidence.

## Remaining Frontier

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. At this checkpoint, all fields read one chosen immutable snapshot;
this is not evidence for alias-sensitive mixed-origin construction while sharing
the live return storage. The subsequent
[mixed-origin follow-up](nuis-native-return-effect-storage-mixed-payload-v1.md)
adds the live mutable carry and both immutable snapshot epochs, and repairs the
boolean field-projection admission gap exposed by its original self-read witness.

Resource/I/O/async captures, compound boolean comparisons against true, stable
ABI, GPU/cross-platform parity, general heterogeneous closure and self-hosting
remain separate gates. This receipt is not full-workspace acceptance.

## Validation

The bounded local M2 CPU acceptance passed 72 distinct tests, including eight
new tests. Initial failed probes and repeated runs are not counted twice:

- Compiler: eight field-payload/payload-call tests passed on the final source
  in 289.93 seconds, including seven type-valid mutation classes.
- Native/reference: six distinct tests passed across the unchanged payload-call
  baseline and final field-payload rerun. The three updated field tests passed
  in 168.00 seconds after the unselected-grant fixture correction described above.
- CLI delivery: two tests passed in 738.26 seconds, covering eight images from
  the new field-payload family and the preceding payload-call family.
- Host fault-state checks: three passed; repository maintenance: 24 passed.
- File-line limits: two passed; status tensor: 27 passed.

Native coverage includes all four source variants. CLI delivery selects one
representative per family, not all four. Other historical return-storage and
sibling-bound families were not rerun in this step; their receipts retain
their earlier evidence. This is not full-workspace or GPU/cross-platform acceptance.

Formatting, whitespace, UTF-8 across 5,611 files, 4,102 local documentation
links and locked offline metadata for 29 workspace packages passed. The
ordinary CLI build passed. All 3,539 drift checks and 24,282 required patterns
matched. Complete predecessor hashes and inert historical commands passed
validation. The rebuilt CLI matches all six source fields; task-card readiness,
coverage, hierarchy and lineage checks are clean.

Existing production lowering supports this fixture without new call authority
or higher fixture limits. No release, physical-performance result or
self-hosting completion is asserted; the selected coordinate remains active/99.

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effect_return_storage_field_payload effect_return_storage_payload_calls --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_loop_sibling_bounds::field_payload_ state_loop_sibling_bounds::payload_calls_ --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- effect_calls::state_loop_sibling_bounds::field_payload_ effect_calls::state_loop_sibling_bounds::payload_calls_ --test-threads=1 --nocapture
```

The six updated tensor fields must retain their complete predecessor values
and SHA-256 hashes. Historical commands stay inert in a quoted here-document.

| Predecessor Field | Complete SHA-256 |
| --- | --- |
| `evidence` | `f4e63ebcae845bef6f0851f7e0268efd767c4786ae5d6ec56a386a61f99ab58e` |
| `next_step` | `a05a2272fbb68328be086445bf4125942e6ef6c5f37423c25a99641ee52ec031` |
| `blocker` | `6f8ba783e2251efcd3973b163afd6810291c3d06fb2ea38a33275d1cdd3e9415` |
| `next_action` | `7d334b15f0dd8bad731df82ed11ce6014075d1cbd28319ca9f743b0304ba374f` |
| `validation_command` | `f7027b8545944de814bd1806ba663ff9652804a809babae4766f1d0c70ee9204` |
| `expected_artifact` | `48cb50ff21a65f138f4b5eefdd441d93c3b4afffa8208b5a3dc1673a1b9cd256` |
