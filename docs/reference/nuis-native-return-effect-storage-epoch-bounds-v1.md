# Native Return Storage Distinct-Epoch Bound Records V1

This bounded local CPU follow-up extends the
[parent-preheader witnesses](nuis-native-return-effect-storage-preheader-bounds-v1.md).
Git remains beta-0.17.4 at bef7a23d. No release is made.

## Two Live Snapshot Identities

The [shared Nuis fixture adapter and independent oracle](../../tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds_cases.rs)
add four explicit leading/trailing-step and mirrored two-epoch variants.
Previous per-parent-iteration and preheader variants retain their own source
semantics. No production code dispatches on these test variant flags.

The selected region completes an older immutable `frozen` record before the
parent loop. After each reached parent update, it completes a distinct `recent`
record from the live State. The first child uses `recent`; the sibling uses
`frozen`. Both records are simultaneously live across first-child execution,
and the older record survives parent backedges. The newer record is rebuilt
only when its source declaration is reached, not reused from an earlier epoch.

For initial total 0 and trips 4, observed child-bound inputs are
`[2, 4, 4, 4]`: first-parent recent/older, then second-parent recent/older.
The oracle requires three construction effects 930, two recent-epoch markers
934 and final integer State 116 with leading steps or 102 with trailing steps.
Substituting either record for the other, or keeping the first recent record
across the backedge, does not satisfy this witness.

All computation still executes as Nuis. Rust supplies fixture generation,
input transport, independent semantic expectations and test probes.
Multi-callback rows use actual previous State while deliberately providing
different unused event State inputs. Raw bool/i64/f32/f64 checks retain the
existing negative-zero and other float-word cases.

## Reached Construction And Seven Faults

Recent construction follows print 934 and evaluates
`root * (second + 3) + 0 / (second + 2)` as its root argument.
With `second = -2`, argument evaluation fails before entering `bound_seed`;
with `second = -3`, the callee prints 930 and then fails its own root check.
Neither case reaches the first child. Both are inert in an unselected region.

The previous five faults remain: older-record construction, first-bound
division, second-bound argument evaluation, second-bound callee and MIN/-1
overflow. The overflow accepted prefix is odd for two-epoch variants, because
the first bound now consumes the newer record after the first parent update.
Preheader-only variants retain their even overflow prefix. This keeps the
intended scale-1 overflowing division instead of a huge valid first loop.

Function return and parent break/continue skip the later sibling's poisoned
argument and callee. Child-local exits retain their existing targets.
Reference sessions preserve the last accepted raw State after three successful
callbacks, enter Faulted, reject later events and refuse successful completion
after close. Native probes check exact flushed failure prefixes, not memory
after process termination.

## Distinct Full Reservations

Ordinary fixture limits remain 18 loop-work units and 192 inclusive YIR helper
entries. Production defaults and guards are unchanged.

The exact native matrix starts after three inactive callbacks, with actual
total 6 and trips 2. Normal and first-child-break paths require 8 units for
leading steps or 9 for trailing steps. One unit less rejects the second
parent's second-child reservation after print 945, before its first body effect.
The trailing break case still charges the second first-child invocation's
complete width 2 even though it exits after one iteration.

Parent continue skips both second-child invocations but must reconstruct and
reserve the reached first child on the backedge. It requires 5 units with
leading steps or 4 with trailing steps; one less rejects after the second 942.
Every exact limit succeeds. Snapshot reuse never merges distinct reservations.

## Certificates And Delivery

[Compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_epoch_bound_tests.rs)
rename both records and the carry, deny direct callee grants and preserve
immutable source. Neither read-only record becomes mutable return storage.
Five type-valid mutations are rejected: first-child use of the older record,
second-child use of the newer record, construction from the wrong State epoch,
effect reordering and recomputation from the current carry.

Independent/shared normal/reversed YIR comparisons check all ten exit modes,
seven selected faults and inactive poison. The actual maximum scoped carry
width retains a four-word logical reduction, not measured physical memory or
speedup. Existing admission, both read sets, pending identity, bounded candidate
attempts, exact fallback and ownership/GLM/time contracts remain unchanged.

[Native/reference tests](../../tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds.rs)
exercise all four new variants through existing session/probe owners.
[CLI delivery](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_sibling_bounds.rs)
runs one new representative through fresh build, cache hit and two repeated
materializations after deleting source, manifest, project cache and original
output. Each image exercises all exit modes, unreachable sibling poisons and
all seven selected faults. Binary/container, host-object, envelope, lifecycle
and packaging identities remain checked.

## Remaining Frontier

The [returned-snapshot follow-up](nuis-native-return-effect-storage-snapshot-returns-v1.md)
tracks the next bounded payload witness. The acceptance below remains specific
to this earlier bound-consumption receipt.

The tensor coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Next return an older or newer snapshot itself from selected inner-loop exits,
rather than only using it as a bound and returning the live carry. Preserve
returned identity and raw fields, payload reads before shared-storage
publication, separate pending state, skipped later sibling effects/reservations
and failure retention. Diagnose the current path before changing authority.

This local CPU evidence does not establish arbitrary resource/I/O/async capture,
stable ABI, GPU or cross-platform parity, full heterogeneous closure or
completed self-hosting.

## Validation

Focused acceptance passes 343 distinct tests, including eight new tests:
274 compiler units (462.36 seconds), ten native/reference tests (299.76 seconds),
three CLI delivery tests (369.72 seconds), three host-fault tests,
24 repository-maintenance tests, two line-budget tests and 27 tensor tests.
The new tests comprise four compiler, three native/reference and one CLI test;
the four-test calibration and subsequent reruns are not counted again.

Native acceptance covers all four new epoch variants and the preceding
preheader and per-parent-iteration variants. CLI acceptance covers 12
fresh/cache/restored images across one representative of each of those three
families, not every variant. The earlier standalone record-dependent bound
mode is not included in this run's native/CLI filters; its prior receipt
remains historical evidence.

The ordinary CLI rebuild and live tensor audit pass: all six fields match
source, all six complete predecessor hashes verify, the task card is ready,
and drift, coverage, hierarchy, lineage and inert history are clean. Static
drift validation passes 3,519 checks with 24,140 required patterns. Locked
metadata contains 29 workspace packages. Formatting, UTF-8 (5,600 files),
local links (4,067), source/test line budgets and whitespace checks pass.
The coordinate remains active/99; no release or version bump is made.

No production lowering repair, authority widening or budget increase was
needed. These are bounded local CPU results, not full-workspace, GPU,
cross-platform, physical-memory/performance or self-hosting acceptance.

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- iteration_effects control_loops scalar_helpers scalar_control state_loop --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_loop_sibling_bounds:: --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- effect_calls::state_loop_sibling_bounds:: --test-threads=1 --nocapture
```

All six tensor fields retain their complete preceding values and SHA-256.
Historical commands remain inside an inert quoted here-document.

| Predecessor Field | Complete SHA-256 |
| --- | --- |
| `evidence` | `9df215d564c2611fbefd6cd54a453f6f47e2efb64ed62563ac2f584a38b31d8a` |
| `next_step` | `4fc67027c3a260415b344fff6a52c8545380c0b04147b0188f0e5eaf2b57041f` |
| `blocker` | `38611b9a351b64e469568c5ae8155d06d6f81678bcd5c42115f93c5af5acc50f` |
| `next_action` | `a88a05da617e1c0ae417bd595d2601a218c7994066486c3551c2a91df3ee5f17` |
| `validation_command` | `b829df3af4d6de914b783dcf66e285d749a7823bb31c8a4e1595b584aa04dc3d` |
| `expected_artifact` | `eadb279fcbfbf23597c4d8d123f9967efdf37fa4f77b100d1d139765d46b7f14` |

