# Native Return Storage Effectful Payload Calls V1

This local CPU follow-up extends the
[returned-snapshot witnesses](nuis-native-return-effect-storage-snapshot-returns-v1.md).
Git remains beta-0.17.4 at bef7a23d.

## Selected Return Evaluation

The [Nuis helpers](../../tools/nuisc/tests/native_application_bridge/state_loop_return_payload_helpers.ns)
finalize a complete older or newer snapshot in the return expression. Two
argument helpers print 970 and 971. The finalizer prints 972, performs a checked
division, prints 973 and returns the snapshot with its integer field increased
by ten. Its bool/f32/f64 fields retain their original bits.

The [fixture and independent oracle](../../tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds_cases.rs)
add four leading/trailing-step and mirrored variants. Existing fixture families
retain their behavior. Modes 1 and 6 return from the first reached child body;
modes 10 and 11 select those returns on the second parent iteration. Each
return follows actual live-carry work, then evaluates the selected expression
once. Subsequent child, sibling and parent work remains skipped.

Modes 12 through 17 distinguish first-argument, second-argument and finalizer
failure for each child. Modes 18 and 19 fail inside the finalizer only after
the parent backedge. These eight payload faults complement the seven existing
construction/bound faults. On payload failure, the trace ends respectively at
960, 970 or 972; 973 and fallback effect 949 are absent. Inactive branches and
zero-width children keep all eight poisoned payload expressions unevaluated.

The initial mirrored fixture used a compound boolean comparison against true.
Current loop admission only accepts boolean atoms in that comparison position;
the final fixture uses equivalent negated comparisons joined with AND/OR.
This preserves the existing condition profile and is not evidence that the
compound boolean comparison restriction has been removed.

The initial native run also reached the 192-entry fixture guard on the ordinary
non-returning path: the expanded mode predicates repeated inside every child
iteration. The final fixture computes those total, mode-only classifications
once before the parent loop. No payload call or checked expression is hoisted;
loop and helper-entry limits remain unchanged. The unoptimized predicate form
is not claimed to fit the same entry budget.

## Publication And Reservations

[Compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_payload_call_tests.rs)
deny each direct callee grant and keep both snapshots read-only. Type-valid
wrong-record substitution, argument reordering, repeated finalization, direct
payload bypass, skipped argument work and pending-before-payload publication
must be rejected by the structural certificate.

Independent/shared normal/reversed YIR execution is compared with the semantic
oracle. Both original and rewritten read sets, bounded candidate attempts,
separate pending identity and exact independent fallback remain requirements.
The measured scoped carry difference is four logical YIR words, not physical
memory consumption or execution speed.

[Native/reference checks](../../tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds.rs)
exercise complete raw State, including signed zero, subnormal and NaN words,
and actual state from accepted callbacks. After three successful callbacks,
each selected failure retains the last accepted reference State and enters
Faulted; later events are rejected and close cannot report successful completion.
Native traps are checked through exact flushed effect prefixes and accepted
State words, not by reading process memory after termination.

The exact reservation matrix still requires 4 or 8 units for first-parent
returns, and 6/7 or 8/9 for post-backedge returns. One unit less rejects before
the selected child body and all payload-call effects. The exact amount reaches
the complete finalization suffix. Fixture limits remain 18 loop-work units and
192 helper entries. Entered loops retain full reservations even when returning
early; skipped work receives no reservation.

## Source-Free Delivery

[CLI coverage](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_sibling_bounds.rs)
uses one new representative through fresh build, cache hit and two repeated
materializations after removal of source, manifest, project cache and original
output. Every image runs healthy callbacks, all fifteen fault categories and a
fresh healthy session afterward. Container/binary, host-object, envelope,
lifecycle and packaging identity checks remain in place.

## Remaining Frontier

The [field-payload follow-up](nuis-native-return-effect-storage-field-payload-v1.md)
now owns the late-field construction gate below; this historical receipt retains
the narrower call-finalization boundary and its original validation scope.

The coordinate stays `standard-library/ns-nova/persistent-application-session`,
active/99. Next exercise field-by-field payload construction: successful
effectful earlier fields followed by a failing later field, inside the returned
record literal. Verify source-field order, once-only nested calls and complete
record publication across first-parent and post-backedge exits.

The current finalizer checks its division before constructing the result
literal. It does not establish late-field construction failure behavior.
Resource/I/O/async captures, stable ABI, device/cross-platform parity, full
heterogeneous closure and self-hosting retain separate acceptance gates.

## Validation

The bounded local M2 CPU acceptance passed 140 distinct tests, including eight
new tests. Initial failed probes and repeated runs are not counted twice:

- Compiler return-storage tests: 66 distinct tests passed. After moving the
  mode-only classifications, the four affected tests passed again in 174.62 seconds.
- Native/reference tests: 16 distinct tests passed across the unchanged baseline
  and final affected-test rerun. The initial batch exposed the entry-budget
  failure described above; all three updated tests then passed in 163.84 seconds.
- CLI delivery: 2 tests passed in 608.05 seconds, covering eight images from the
  new payload-call family and the preceding direct-snapshot family.
- Host fault-state checks: 3 passed; repository maintenance: 24 passed.
- File-line limits: 2 passed; status tensor: 27 passed.

Native coverage includes all four new source variants. CLI delivery uses one
representative per selected family, not every source variant. The other three
older sibling-bound CLI families were not rerun in this step; their earlier
receipts retain that evidence. This is not full-workspace or GPU/cross-platform
acceptance.

The ordinary CLI build, formatting, whitespace, UTF-8 verification across 5,607
files and 4,090 local documentation links passed. Locked metadata contains 29
workspace packages. All 3,532 drift checks and their 24,236 required patterns
matched. The rebuilt CLI agrees with all six source tensor fields; complete
predecessor hashes, inert command history, task-card readiness, coverage,
hierarchy and lineage checks are clean.

Existing production lowering handles the final fixture without widening call
authority, changing production defaults or increasing fixture limits. No
release, version bump, physical-performance result or self-hosting completion
is asserted. The selected coordinate remains active/99.

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effect_return_storage --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_loop_sibling_bounds:: --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- effect_calls::state_loop_sibling_bounds::payload_calls_ effect_calls::state_loop_sibling_bounds::snapshot_returns_ --test-threads=1 --nocapture
```

The six current tensor fields retain their complete preceding values and
SHA-256 hashes. Historical commands remain inert inside a quoted here-document.

| Predecessor Field | Complete SHA-256 |
| --- | --- |
| `evidence` | `18f757a428ccfa7c9884b093ab316efc232b196072e2ac72f0fcbd20b170b54a` |
| `next_step` | `4940f32ea51c6cc8077461d87f1c253e42a213e1fda1cfd4447a0f618efcf9c2` |
| `blocker` | `7406a376d4cbfc89bb14891c9fa2f32ff4220bb6b1c2fd52a82967ec0fa5d6ab` |
| `next_action` | `3c23cd4907de2a248306c6fcee703b4ce1959157a30787af130dd8f9d88af8f7` |
| `validation_command` | `ed8e75b609f6fdcbe4bf8aa90c0ae1310552a27e785eee830358b5f51036dbd7` |
| `expected_artifact` | `632430c390ed82b322e0f6e5ac1641b888b40f350661bcbde7aeaf972369fad2` |
