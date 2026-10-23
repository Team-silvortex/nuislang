# Native Return Storage Shared Sibling Bound Records V1

This bounded CPU follow-up extends the
[record-dependent bound witnesses](nuis-native-return-effect-storage-bound-calls-v1.md).
The inspected Git identity remains beta-0.17.4 at bef7a23d. No release is made.

## One Snapshot, Separate Reached Loops

A focused [Nuis source fixture](../../tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds.ns)
updates a real bool/i64/f32/f64 carry, then constructs one completed `frozen`
record with the shared
[bound helpers](../../tools/nuisc/tests/native_application_bridge/state_loop_return_shared_bound_helpers.ns).
Two sibling loops independently consume that same immutable record through
`bound_extent`. The record is constructed once per selected parent iteration,
not once per callback; a callback can reach two parent iterations.

The first child changes the live carry's integer parity, but the second child
must still use the frozen record rather than recomputing from that carry.
With input trips 4, the selected snapshot has width 2, even when the later live
State would produce width 4. Semantic oracle controls distinguish these values,
require one `930/931` record-construction pair and two successful extent calls,
and check the resulting integer State. No computation is replaced by Rust:
Rust supplies only the independent oracle, input transport and test driver.

The [source adapter/oracle](../../tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds_cases.rs)
owns four explicit leading/trailing-step and mirrored-control variants. This
is a separate focused program, not a ninth mode of the preceding eight-mode
fixture. It reuses the existing State raw-word cases and bound helper source.

## Exit Targets And Poisoned Siblings

The first child's local `break` does not skip its sibling. Its local `continue`
only suppresses the remainder of the current child iteration. Function return
and parent-loop break/continue do suppress the later sibling and its reservation.
Both children also exercise their own return/break/continue paths.

The selected outer update has live unselected poisoned arguments. A skipped
child region also carries poisoned division and enormous input bounds.
The second sibling has both a raw argument failure (`second = -1`) and a
callee failure (`second = 0`). CLI delivery exercises both poisons after each
unreachable-sibling exit target, not merely the callee poison.

The oracle models semantic loops and actual State; it never interprets raw
numeric observer values as synthetic effects. Multi-callback rows use the
actual previous State, deliberately supplying different unused event State
inputs to detect accidental reseeding.

## Reservation And Failure Order

The ordinary fixture declares 18 loop-work units: two parent units plus at
most four units for each of four reached child invocations. The initial 160
inclusive YIR helper entries proved insufficient for the longest
callback. The new fixture explicitly declares 192; the previous bound fixture
keeps 160. A dedicated normal/reversed control retains limit 160 and requires
the entry-budget diagnostic, a trap, a proper source-effect prefix and no
callback publication. Production defaults, guards and admission are unchanged.

A separate exact-budget witness reaches two width-2 siblings after reserving
two parent units. Limit 5 rejects after the second extent/print 945 and before
any second-child effect; limit 6 succeeds for the first-parent return path.
The limit-5 rejection is also tested when the first child breaks after one
iteration: its complete width 2 must still be charged. Sharing a record does
not share or deduplicate the two reservations.

Selected failures cover record construction, first-bound division, second-bound
argument evaluation, second-bound callee and MIN/-1 overflow. Second-bound
failures occur after real first-child work, with no second-child publication.
Overflow cases preserve the actual accepted-prefix State so MIN reaches a
scale-1 division; replacing it with the input State would test another path.
Reference sessions keep the last accepted raw State after three callbacks,
enter Faulted, reject later events and refuse successful completion after close.
Flushed native trap probes check exact effects, not memory after termination.

## Certificates And Delivery

[Compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_sibling_bound_tests.rs)
rename helpers, storage and the snapshot, preserve immutable source and pending
identity, and deny direct callee grants. Type-valid current-carry substitution,
record recomputation, effect reordering and argument-check bypasses fail the
structural certificate. An earlier rejected candidate still allows a later
certified carry; loop-local `frozen` is not a dominating storage candidate.
Original-invalid inner reads remain rejected.

Independent/shared normal/reversed YIR comparisons check results, ordered
effects and failures while retaining a four-word logical scoped carry
reduction, not measured physical memory or runtime speedup. No production
lowering capability is widened; the production module edit registers tests.

[Native/reference tests](../../tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds.rs)
cover the four variants, raw float words, actual State threading, local exits,
selected failures and exact computed reservations.
[CLI delivery](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_sibling_bounds.rs)
uses one representative through fresh build, cache hit and two materializations
after removing source, manifest, project cache and original output. Each image
executes all ten exit modes, unreachable argument/callee poisons and all five
selected failure categories. Binary/container, host-object, envelope, lifecycle
and packaging identities stay intact. Buffered CLI failure output is not used
as a substitute for the flushed native prefix proof.

## Remaining Frontier

The [parent-preheader follow-up](nuis-native-return-effect-storage-preheader-bounds-v1.md)
tracks the next bounded lifetime witness. The acceptance below remains specific
to this earlier per-parent-iteration snapshot receipt.

The coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Next validate a completed immutable bound record dominating a parent loop and
surviving its backedges, rather than being reconstructed per selected parent
iteration. Reuse it across reached sibling loops while the live carry changes;
retain independent full reservations, reached-only effects, failure boundaries,
actual State bits and source-free delivery. Preserve current catalog, both read
sets, ownership/GLM/time contracts, bounded candidates and exact fallback.

Resource/I/O/async capture, wider changing backedges, stable ABI, device and
cross-platform parity, full heterogeneous closure and completed self-hosting
remain separate gates. These are bounded local CPU witnesses only.

## Validation

Focused acceptance passes 333 distinct tests, including 10 new tests:
266 compiler units (434.13 seconds), nine native/reference tests (217.25 seconds),
two CLI delivery tests (345.73 seconds), three host-fault tests,
24 repository-maintenance tests, two line-budget tests and 27 tensor tests.
Calibration probes are included in those suites, not counted again.

Native acceptance includes the four sibling-program variants and the preceding
record-dependent bound mode, not every earlier native source mode. CLI delivery
covers eight fresh/cache/restored images across one sibling representative and
one preceding bound representative, not all four sibling variants or all eight
preceding CLI modes. The new representative runs all ten control modes, both
unreachable poisons and five selected failure categories in every image.

The ordinary CLI rebuild and live tensor audit pass. All six current fields
match source, all six complete predecessor hashes verify, the selected task
card is ready, and drift, coverage, hierarchy, lineage and inert history are
clean. Static drift validation passes 3,507 checks with 24,050 required patterns.
Locked metadata contains 29 workspace packages. Formatting, UTF-8, local links,
source/test line budgets and whitespace checks pass. The coordinate remains
active/99; no release or version bump is made.

Initial native calibration rejected the fixture's declared 160-entry budget;
the final matrix uses the explicit 192-entry policy and retains a dedicated
160-entry rejection control. An independent entry probe also needed the actual
accepted-prefix State to preserve its intended MIN/-1 overflow. These are test
configuration/construction corrections, not production lowering repairs.

The focused executable routes are:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- iteration_effects control_loops scalar_helpers scalar_control state_loop --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_loop_sibling_bounds:: state_loop_return_shared::bound_tests:: --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- effect_calls::state_loop_sibling_bounds:: effect_calls::state_loop_return_shared::branch_bound_calls --test-threads=1 --nocapture
```

Each tensor field retains its complete predecessor and SHA-256. Historical
commands remain inside an inert quoted here-document.

| Predecessor Field | Complete SHA-256 |
| --- | --- |
| `evidence` | `6c5cad6ee4b5fb202b83709046af89ec182ecb9be6958562c21a11848ceb9b92` |
| `next_step` | `1bee6d16bb9566445bfc34777396e9183f75190db74f832a3ad21903bbe97dfb` |
| `blocker` | `d255677b9e182ca54a1da5cdda386ad0317d74686876d82764155353aa7a71e3` |
| `next_action` | `1eef97644f400d269339ed4f7100c6d57bbf081d76fc794d1b331333a34b39f1` |
| `validation_command` | `16df0ad0d0b602c4223d795affddeadcff347ab0a4e80ef6e627d78e01ba1ad6` |
| `expected_artifact` | `8161d55496e5c22c58982de095155fd983a43a2fc177b8f8885a42c7be369d84` |
