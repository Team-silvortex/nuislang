# Native Return Storage Conditional Payload V1

This bounded local CPU follow-up extends the
[unprojected-field witnesses](nuis-native-return-effect-storage-unprojected-payload-v1.md).
It starts from Git beta-0.17.5 at c3e6a14f, without a version bump.

This receipt records the earlier return-focused checkpoint. The
[ready-value follow-up](nuis-native-return-ready-values-v1.md) tracks removal of
its full-fixture helper-count obstacle and the replacement of focused tests.

## Lowering Repair

The initial executable witness failed with division by zero on an unselected
right operand. Iteration helpers already guarded logical RHS work, but scalar
control extraction copied an admitted compound condition into an eager binding.
Type/call admission alone did not preserve its short-circuit semantics.

[Shared logical-condition lowering](../../tools/nuisc/src/lowering/buffer_loop_outline/logical_conditions.rs)
now serves both scoped iterations and
[scalar control extraction](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_control.rs).
The existing implementation is moved, not duplicated: one guarded helper per
logical edge keeps the complete RHS behind the once-evaluated left predicate.
OR retains its neutral-false complemented encoding. Both routes retain their
original admission rules and registered guard boundaries.

## Lazy Callee Fields

The [Nuis fixture](../../tools/nuisc/tests/native_application_bridge/state_loop_return_conditional_helpers.ns)
retains the same complete record construction and four direct call projections.
Its marker field uses `record_gate(choice) && record_rhs(selected_root)`;
its total field uses `record_gate(choice) || record_rhs(selected_root)`.
The left predicate prints once. A reached right predicate prints before and
after its checked division and returns a runtime-dependent boolean. Each `if`
arm has a distinct print. A skipped right operand receives a zero divisor:
eager evaluation would trap even though the original call should succeed.

All six lazy truth paths are required: false-left AND, true-left AND with both
right results, true-left OR, and false-left OR with both right results. The
predicate input comes from the original live/older/newer record, not the outer
literal's already computed inverted boolean. All returned raw fields remain
identical to the preceding family, making the effects independently observable.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/state_loop_return_conditional_cases.rs)
models field source order, selected predicate calls, branch prints and failure
cuts separately from generated source. The existing 31 failure categories are
retained, including first-parent and post-backedge faults before and after the
desired gain field. Selected RHS faults must stop before its post-check print;
skipping that RHS must not skip the field's subsequent unconditional check.
Whole-return unselected and zero-width controls remain in place.

## Authority And Controls

[Compiler tests](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_conditional_payload_tests.rs)
recollect the complete call catalog after removing a predicate or selector leaf.
Both field helpers, the record helper and the enclosing loop must lose admission.
Effectful predicates stay outside the pure catalog. The conditional-return
selector remains pure; purity is not borrowed for its effectful callers.

The previous fifteen type-valid caller mutation controls are retained. Four
alternative source programs eagerly move the RHS before its guard or prune it,
for both operators. Eager versions trap; pruned versions keep the final integer
but change the effect trace. These are oracle-sensitivity controls, not claims
that the caller certificate proves arbitrary callee-body equivalence.

Independent/shared and normal/reversed YIR are compared across four source
variants. Four fewer logical YIR carry words are not a physical-memory or speed
measurement. Loop-work/helper-entry limits remain 18/192; no backend-specific
admission shortcut or enlarged execution budget is introduced.

## Native And Delivery Gates

[Reference/native probes](../../tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds.rs)
use four return-focused variants and compare raw bool/i64/f32/f64 state, signed
zeros, subnormals and NaNs. Reference
failure after three accepted callbacks retains the last complete State, enters
Faulted, rejects further events and refuses successful completion. Native probes
check accepted State and the exact flushed failure prefix, not memory in a
terminated process or rollback of earlier emitted effects. Reservation checks
retain exact-required and one-less bounds, charged before selected field work.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_sibling_bounds.rs)
selects one representative for fresh build, cache hit and two source-free
materializations. Source, manifest, project cache and original output are
removed before restoration. Each image runs healthy sessions, all 31 failure
categories and another healthy session; binary/container, host-object, envelope,
lifecycle and packaging identities are checked. CLI failures require no new
accepted State or completion; flushed failure-prefix evidence belongs to the
separate native probes.

The full compiler stress fixture exceeds the native profile's existing 64
reachable-function cap. A negative admission test preserves that veto; the cap
is not raised. Native/reference and CLI delivery use `conditional_native_variants`,
which remove unrelated local/parent break, continue and mode-9 return branches
before mirroring/loop adaptation. Modes 2-5 and 7-9 are explicitly unavailable
in this focused family, not silently assigned different expectations. First-parent
and post-backedge returns, all 31 failure categories, skipped/zero-width work,
both record epochs and the same raw-value/effect oracle remain. The full family
retains those exit modes in the independent/shared compiler matrix. Thus this
is not native acceptance of the entire accumulated stress fixture.

## Remaining Frontier

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. This fixture uses a conditional-return helper to select the RHS
argument. A typed `let selected_root: i64 = if ...` instead produces
scalar branch assignments that the isolated iteration-call catalog still rejects.
A regression pins that conservative rejection and checks that collection does
not mutate the source module. This is a specific admission boundary, not a claim
that all Nuis `if` expressions are unsupported.

Next reduce generated private-helper overhead for the complete stress closure,
without raising the 64-function cap or duplicating/speculating effects. Keep
focused and full fixtures distinct until the latter can pass native admission.
Conditional scalar value bindings also need a separate admission/lowering proof,
preserving branch scope, original reads and atomic fallback; do not globally
permit arbitrary scalar reassignment to accept a witness.
Resource/I/O/async captures, compound boolean comparisons,
stable ABI, GPU/cross-platform parity, full heterogeneous closure and self-hosting
remain separate gates.

## Validation

Bounded local M2 CPU acceptance passed 90 distinct tests, including eleven new
tests. Repeated checks are not counted twice:

- Compiler: 29 distinct checks. The final 28 focused/adjacent controls passed in
  13.59 seconds; the full independent/shared execution matrix and six loop/call
  controls passed in 427.24 seconds. Those six controls overlap the final set.
- Native/reference: four passed in 238.20 seconds, including four return-focused
  variants, unchanged reservations and the full-closure admission veto.
- CLI delivery: one passed in 827.04 seconds, covering fresh/cache and two
  source-free images with all 31 failure categories on one focused representative.
- Host failure-state controls: three passed; repository maintenance: 24 passed.
- File-line limits: two passed; status tensor: 27 passed in 5.22 seconds.

The ordinary CLI build passed in 8.51 seconds. Formatting, whitespace, UTF-8
across 5,631 files, 4,154 local documentation links and locked offline metadata
for 29 packages passed. All 3,581 drift checks and 24,539 required patterns match.
The rebuilt CLI matches all six source fields; complete predecessor hashes and
inert historical commands are verified. Task readiness, coverage, hierarchy and
lineage are clean; the coordinate remains active/99.

The full native stress closure remains rejected at the unchanged 64-function
cap. Historical native/delivery families and the full workspace were not rerun.
No GPU/cross-platform or physical-performance acceptance is asserted.

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effect_return_storage_conditional_payload --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effect_return_storage_unprojected_payload --skip matches_independent_execution_and_carry_width --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- conditional_payload --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- conditional_payload --test-threads=1 --nocapture
```

All six complete predecessor fields and hashes remain in the tensor. Historical
validation commands remain inert inside a quoted here-document.

| Predecessor Field | Complete SHA-256 |
| --- | --- |
| `evidence` | `9f00a8151def836a3b82648830f45b427d08292e109b6148c3ace7b6f3b3b58d` |
| `next_step` | `95470b4505a2ddaee5a86fa8c5fcf46729e78237181ecf7e72ebc72f60f2ddb1` |
| `blocker` | `08d2ffe1331404d1eec5dd78b72566d88a51c7041fc04925b8673f81700e4647` |
| `next_action` | `19df3f6461a245a87110a8afc1fa67f0bd56e6e7a4845445c7f3fb10061ebcfa` |
| `validation_command` | `9f010ced8b86a548ac8a7a740e5bd0e808ee8e8f3ba35288c4d6367b2511ab6b` |
| `expected_artifact` | `746c36ce022216693d466fd3563aca0e8427f79b869c8ae90f2ab85afcd76e77` |
