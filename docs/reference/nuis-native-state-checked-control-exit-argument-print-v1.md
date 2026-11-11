# Native State Checked Control Exit Argument Print V1

Audit date: 2026-11-11. Checkout baseline: beta-0.18.7, `a270a99e`.
The preceding [persistent-control checkpoint](nuis-native-state-control-exit-argument-print-v1.md)
is now committed; its original worktree receipt is unchanged. This follow-up
creates no version bump, commit, tag or release.

## Independently Checked Control Updates

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/state_checked_control_exit_argument_print.ns)
replaces the total control increment with an independent `advance_control`
producer. Its i64 fault input is checked before it returns the wrapping `value + 1`.
The existing float-update check stays before record construction; the producer
runs before the completed-record marker and every exit argument.

A failed float check never enters the control producer. A failed control producer
can follow float computation, but prevents complete record construction and skips
all subsequent confirmation work. A successful producer does not by itself
publish callback State: later exit, tail and callback completion checks still
own that boundary. Both accepted State words remain separate from pending work.

The callback retains `State { total: f32/f64, control: i64 }`. One new i64 fault
input appends to the unchanged 37-word prefix. Open/event/close input widths are
38/40/2, with two State output words and unchanged scalar transport limits.
The earlier total-update and one-word profiles remain independent.

[Source controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/state_checked_control_exit_argument_print_tests.rs)
retain the control carry's independent parent-entry readiness, original field
epochs, exit stepping and exact i64 producer signature. Float, bool and i32
arguments and a bool result are rejected. No new read or effect authority is
inferred from the control producer.

## Structured Observation And Fault Models

The [checked-control model](../../tools/nuisc/tests/native_application_bridge/state_checked_control_exit_argument_print_cases.rs)
reuses the predecessor's successful State schedule because healthy advancement
is still wrapping `+1`. A separate fault model inserts producer entry before
each completed record, stopping at the first zero update guard. It decodes the
predecessor trace by declared observation widths rather than scanning every
integer as a marker: operand payloads can equal marker values.

The matrix covers 33 raw sequences, eleven finite sequences and 19 fault rows
per width. Raw controls deliberately include 71, 75, 76, 80 and 617, as well as
i64 boundary wrapping and MIN/MAX fault inputs. Marker-shaped payloads must not
create phantom parents, child iterations or producer entries. Native probes
record the actual producer input and fault word; reference execution counts
exactly one producer call per completed child.

Eight retained faults cover bound production, float update, both callback tails
and first/later continue/break confirmation checks. Seven new control failures
cover every child in the first and later active parents. Two collisions make
float failure win before control entry, and two make control failure win over a
later exit check. Accepted-State decoys remain different in both fields; failed
events and faulted close retain the complete accepted State. Wrong-type update
fault input is rejected while the reference session remains Open and can still
accept a valid event.

Zero-parent/zero-child rows carry zero update-fault inputs without entering the
producer. An early-break row poisons the skipped second child but continues later parents
with fresh offsets. Control updates count completed work, never reserved work,
and reservations are not refunded.

## Independent Native Controls

[Native controls](../../tools/nuisc/tests/native_application_bridge/state_checked_control_exit_argument_print.rs)
move the float check after record construction in a source-only contrast. Healthy
results stay the same, but colliding float/control faults now enter and fail the
control producer first. Both first-parent and later-parent prefixes distinguish
this order from the original source. Declaration-order reversal is also covered.

A separate return-value control XORs bit 32 into the source producer input and
decodes it in the producer before adding one. The observed input changes, but
the returned control and full State must remain exact. Finite, MIN/MAX and
marker-valued controls plus reached producer failures prevent substituting input
words for returned values or truncating to 32 bits.

Adding a print to the update producer requires that exact native site grant.
The full selected-site policy permits emission; withholding the new site while
keeping all other grants rejects it. This is an admission check, not a broader
execution grant for arbitrary transitive effects. Native traps are process
failures; no test reads a pending result after trapping to claim recoverability.

## Quota-Preserving Source Shapes

The extra producer initially exhausted the unchanged 192-entry limit. The final
witness removes only unnecessary surrounding helper entries: parent float
identity uses two sign-bit negations, and the bound helper selects already-ready
total integer values. The bound's checked prefix remains first.

For an integer round, the squared remainder modulo two is zero for even rounds
and one for odd rounds. Multiplying the base or its wrapping increment by that
value preserves the old bound result, including signed rounds; no fallible branch
work is speculated. Raw zero-child controls pass i64 MIN/MAX bases through an even
parent, validating wrapping intermediates without manufacturing child work.
The native IEEE matrix retains raw float words through the sign-flip identity.

Restoring both predecessor source shapes explicitly rejects at 192 entries.
Healthy paths retain 12 loop work and 192 helper entries; limits eight and 64
remain negative controls. Exact work limit/limit-minus-one checks and a poisoned
later update prove reservation failure still wins before the rejected body.
The expected prefix uses the actual changed update guard on earlier producer
entries, not the healthy guard's trace. No runtime accounting, production
compiler rule or allowed work/entry bound is changed.

## Artifact Delivery

The [independent CLI driver](../../tools/nuis/tests/native_session_workflow/state_checked_control_exit_argument_print.rs)
runs eleven healthy sequences before and after 19 faults on fresh builds, cache
hits and two independent source-free restorations in both widths: 328 artifact
runs. Source, manifest and project cache are removed before restoration. Complete
State, stdout, completion status and binary/container/host-object/envelope/
lifecycle/packaging identities are checked.

Direct native probes own exact trapping prefixes; CLI output may be buffered.
Non-finite payload evidence belongs to raw word probes, not finite CLI argument
serialization. Earlier accepted State lines cannot be replaced with partially
computed results from a failed callback.

## Validation

Fresh selected runs on 2026-11-11, including the committed persistent-control
predecessor where named below. Earlier receipts and historical test totals are
not counted again.

| Selected Suite | Result |
| --- | --- |
| Checked/persistent control source and refusal tests | 4 passed |
| Checked/persistent control native and reference tests | 14 passed |
| Checked-control artifact delivery | 1 passed |
| Tensor tests, including persistent lineage | 34 passed |
| Script unit tests | 40 passed |
| Repository file budgets | 2 passed |
| Host absolute-path policy | 1 passed |

All 96 distinct selected tests passed. The artifact-delivery test completed in
672.79 seconds and covers the 328 runs described above, including both independent
source-free restorations for each width. Repeated validation runs are not added
to this total.

The rebuilt ordinary CLI reports 4364 passing drift checks and zero failures.
All 4352 committed predecessor IDs are retained, the existing registration
order is unchanged, and the checked-control group adds twelve checks. All six
source/runtime fields agree and retain the complete committed predecessor
values and SHA-256 hashes. Coverage, hierarchy and task lineage remain clean.
Historical validation commands were syntax-checked through stdin; only their
quoted inert suffix was executed, not the historical command chains.

Workspace architecture remains 29 packages, six domains and 74 normal/build
edges. Maintenance checks pass for 6103 UTF-8 files and 4867 documentation links,
with formatting and patch whitespace checks clean. These are selected host
checks, not a full workspace, CI or Linux/GPU acceptance run.

## Scope And Next Gate

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Next make the checked control producer select State-dependent return branches,
rather than always adding one. Retain correct current/updated value provenance,
selected/skipped checks, complete accepted two-field State, source-free delivery
and existing read/effect/work/entry boundaries.

This is bounded host CPU evidence, not arbitrary resource/I/O/async capture,
pre-update ordinary reads, Linux/GPU parity, a stable ABI, arithmetic NaN payload
guarantees, full native-engine closure or completed self-hosting.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- state_checked_control_exit_argument_print state_control_exit_argument_print --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_checked_control_exit state_control_exit --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- state_checked_control_exit --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test file_line_limit --locked -j1 -- --test-threads=1
python3 -m unittest discover -s scripts/tests
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-host-absolute-paths.sh
python3 scripts/check-text-encoding.py
bash scripts/check-doc-links.sh
cargo fmt --all -- --check
git diff --check
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --locked -j1
target/debug/nuis dev-tensor --json
```

## Predecessor Lineage

All six complete persistent-control values are retained. The
[persistent lineage regression](../../tools/nuis/src/dev_tensor_native_argument_lineage_tests.rs)
checks their full SHA-256 values; earlier checkpoint guards remain intact.
Historical validation commands stay quoted no-op here-document data. Check
syntax through stdin and execute only the inert suffix, never old command chains.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `aa05ce5c46222c478d014a3b79f9664f71cd21c2239c500769735c4c84eb844d` |
| `next_step` | `6e43c43c2fa77931d5c3ffbb0d9e045912f4d0bc0d9d70814d4e42ab4e85fb30` |
| `blocker` | `4b8043312f6d2a366aa82281638cc6c37d54e7f3461b0b430450e28b35264810` |
| `next_action` | `183df07ea7ae339c8ba653770a890821fd8498f31b2e522078f7b6a37fb4f8c1` |
| `validation_command` | `c4d94b5d842624556b3dee91152a02f743d04f899f8f0b180218cfd1647d8bf2` |
| `expected_artifact` | `b58763494f2f72e4587961a17c5489fd866eba6af8ce2c5575a3eefac606d780` |
