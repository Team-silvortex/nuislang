# Native Return Storage Returned Snapshot Records V1

This bounded local CPU follow-up extends the
[distinct-epoch bound witnesses](nuis-native-return-effect-storage-epoch-bounds-v1.md).
Git remains beta-0.17.4 at bef7a23d. No release is made.

## Return The Selected Snapshot

The [Nuis fixture adapter and semantic oracle](../../tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds_cases.rs)
add four leading/trailing-step and mirrored returned-snapshot variants.
The previous three source families retain their own behavior. Test flags do
not add production dispatch cases.

The first child returns the complete `recent` record in mode 1; the second
returns the complete older `frozen` record in mode 6. Modes 10 and 11 select
those same returns only on the second parent iteration. The existing return
conditions are extended with pure compound predicates, not extra side effects.
All returns follow a real live-carry body update.

These are completed bound records: their integer field is the previously
computed bound-seed value, not the original State integer or current carry.
Their bool/f32/f64 fields retain the bits from their capture point.
With the ordinary negative-zero seed, returning `recent` preserves positive
zero and the toggled flag even after the live carry has toggled back.
Returning `frozen` preserves the original negative zeros and flag.

Late-return rows use trips 5 and divisor 2. After the parent backedge, the newer
record has integer 2 with leading steps or 5 with trailing steps. The older
record has integer 5 in both cases, but different bool and float signs.
Thus comparing only integers would miss the trailing-step wrong-record case.

The oracle now retains complete snapshot records rather than only their
integer bound values. Prior variants still consume the same integer bounds.
Raw-word rows cover all four returned-snapshot exits with zero, signed zero,
subnormal and NaN payload cases, including deliberately different unused event
State inputs and actual previous-State threading.

## Publication And Exit Order

[Compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_snapshot_return_tests.rs)
rename storage and both snapshots, preserve immutable source and deny direct
callee grants. Neither read-only snapshot becomes mutable return storage.
Type-valid older/newer swaps, live-carry substitution and pending-before-payload
publication are rejected by the existing structural certificate.

Independent/shared normal/reversed YIR comparisons check all twelve modes,
inactive poison and seven selected construction/bound faults. The actual
maximum scoped carry width retains a four-word logical reduction, not measured
physical memory or speedup. Both read sets, separate pending identity, bounded
candidate attempts, exact fallback and ownership/GLM/time contracts remain gates.

Successful snapshot returns stop after the final body update effect 960.
They skip the rest of that child, any later sibling work or reservation, later
parent iterations and fallback effect 949. An early first-child return also
keeps both later sibling argument/callee poisons inert.

The seven existing fault categories are exercised in callbacks that otherwise
would return snapshots: modes 1 or 6 rather than mode 9. These remain older/newer
construction and reached-bound failures, not failing bare payload reads.
Reference sessions keep the last accepted raw State after three successful
callbacks, enter Faulted, reject later events and cannot complete successfully
after close. Native probes compare exact flushed prefixes, not memory after
process termination.

## Full Entered Reservations

Ordinary fixture limits remain 18 loop-work units and 192 inclusive YIR helper
entries. Production defaults and guards are unchanged.

The exact native matrix starts after three inactive callbacks with actual
total 6. Mode 1 requires 4 units: parent reservation 2 plus the first child's
complete width 2, even though its first body iteration returns. Mode 6 requires
8 units, including the second child's complete width 4 before its first return.

Late modes use trips 5 and divisor 2. Mode 10 needs 6 units with leading steps
or 7 with trailing steps. Mode 11 needs 8 or 9 respectively. One unit less
rejects at print 942 or 945 before entering the selected returning child;
the exact limit succeeds. Return does not refund an entered loop's full bound
or charge siblings and iterations that are never reached.

## Native And Source-Free Delivery

[Native/reference tests](../../tools/nuisc/tests/native_application_bridge/state_loop_sibling_bounds.rs)
exercise all four returned-snapshot variants, normal/reversed modules, raw State,
seven faults and the exact-budget matrix through existing shared probe owners.
No Nuis computation is replaced by Rust; Rust supplies input transport,
independent semantic expectations and test drivers.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_sibling_bounds.rs)
uses one new representative through fresh build, cache hit and two repeated
materializations after deleting source, manifest, project cache and original
output. Each image runs all twelve modes, unreachable sibling poisons and all
seven selected fault categories. Binary/container, host-object, envelope,
lifecycle and packaging identities remain checked.

## Remaining Frontier

The [effectful payload-call follow-up](nuis-native-return-effect-storage-payload-calls-v1.md)
tracks the next boundary described below; this receipt retains its own acceptance.

The tensor coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Next compose a returned snapshot with effectful finalization in the return
expression. Distinguish ordered argument failure from callee failure after
successful body work, evaluate selected payloads once, and publish neither
pending state nor partial record fields until the full payload is complete.

Direct immutable record reads here do not themselves fault. The seven current
construction/bound failures must not be cited as coverage of failing payload
evaluation. Resource/I/O/async captures, stable ABI, GPU/cross-platform parity,
full heterogeneous closure and self-hosting remain separate gates.

## Validation

The bounded local M2 CPU acceptance passed 351 distinct tests, including
eight new returned-snapshot tests. Calibration and repeated runs are not
counted twice:

- Compiler units: 278 passed in 524.96 seconds.
- Native/reference sibling-bound tests: 13 passed in 426.99 seconds.
- CLI delivery tests: 4 passed in 469.78 seconds.
- Host fault-state tests: 3 passed.
- Repository maintenance tests: 24 passed.
- File-line limits: 2 passed.
- Status tensor tests: 27 passed.

The native filter covers all four new variants and the preceding three
sibling-bound families. The four CLI tests cover 16 images, one representative
from each of the four families; they do not deliver every source variant.
Earlier standalone record-dependent bound tests were not included in these
native and CLI filters and retain their own historical receipts.

The ordinary CLI build, formatting, UTF-8 verification across 5,603 files,
4,078 local documentation links and whitespace checks passed. Locked metadata
contains 29 workspace packages. All 3,525 drift checks and their 24,187 required
patterns matched. The rebuilt CLI agrees with all six source tensor fields;
their complete predecessor hashes verify, historical commands remain inert,
and task-card readiness, coverage, hierarchy and lineage checks are clean.

No production lowering repair, authority widening or budget increase was
required. This is not full-workspace, GPU, cross-platform, self-hosting or
physical performance acceptance. The coordinate stays active/99; neither a
release nor a version bump is made.

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- iteration_effects control_loops scalar_helpers scalar_control state_loop --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_loop_sibling_bounds:: --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- effect_calls::state_loop_sibling_bounds:: --test-threads=1 --nocapture
```

All six tensor fields retain their complete preceding values and SHA-256.
Historical commands remain inside an inert quoted here-document.

| Predecessor Field | Complete SHA-256 |
| --- | --- |
| `evidence` | `d3e5110f3d8f027747ff7e38af40a150b7e0373c0512deb4dbc18bb1d1411830` |
| `next_step` | `003cb033d66e51443e1f72d247f3599b9eaee5bc40c0ba615bddd6da30907a86` |
| `blocker` | `c57710b99beb1f4893fa777c1092a7e9f79b2a1ba70d19ba68bec6c975b6450a` |
| `next_action` | `3c6d90623d407b2a76ca2ca12e8bdbb1656e9a13942859ee98d44c44591d55bd` |
| `validation_command` | `c0a2464d43350b9443ddf7fd7b4275985ae99107aa57e9b48a762a2974263853` |
| `expected_artifact` | `0365913a72b6f546a2b62d979d35a8b55aef728477501b1b1bf03595b3c7d4ee` |

