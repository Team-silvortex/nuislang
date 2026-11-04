# Native Fallible Bound Argument Print V1

Audit date: 2026-11-04. Checkout baseline: `dc87ec43`; latest version-label
commit: beta-0.18.4, `ceaa2caa`. This extends the
[parent-bound receipt](nuis-native-parent-bound-argument-print-v1.md).
No release, version bump or commit is created here.

## Fallible Bound Producers

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/fallible_bound_argument_print.ns)
adds an independent bound guard to the existing zero/positive/zero/resumed
child schedule. The noinline producer checks division by its guard before
returning a child count, even when that count would be zero. Its guard varies
by parent index, independently of child-initializer, lazy-use, update and tail
faults. No production lowering authority is widened.

A parent that is not entered does not call the producer. Every reached parent
does, including an all-zero child schedule. Negative nonzero guards remain
valid: the guard is not a work budget. Source admission keeps both float kinds,
reversed declarations, unready-record/alias/branch refusals and rejection of a
child body that mutates its own bound.

## Model And Execution

Each f32/f64 kind has 24 raw sequences, eight finite sequences and 14 fault rows.
The [profile](../../tools/nuisc/tests/native_application_bridge/fallible_bound_argument_print_cases.rs)
retains all nine earlier child/tail faults and adds five producer failures:
parents zero through three of the 0/3/0/5 schedule, and parent one of an all-zero
schedule. Two later producer failures follow three completed children.

The shared [independent model](../../tools/nuisc/tests/native_application_bridge/parent_bound_argument_print_cases.rs)
alternates activity and increments counts instead of reusing source modulo
arithmetic. It checks the producer guard before entering the modeled child.
Native entry marker 602 observes real base, parent and guard arguments; marker
601 still observes real child helper arguments. Instrumentation never replaces
source calculation. The new fixture has twelve callback input words; older
profiles retain eleven, with explicit isolation checks.

The [native/reference controls](../../tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print.rs)
compare raw State and ordered output under normal/reversed YIR. Successful
native probes read State and canaries. Failed native probes require exact
flushed output and absence of success publication; this is not a failed-buffer memory snapshot.
Reference sessions directly check their last accepted State after failure,
a rejected subsequent event and faulted close. Prior output is not rolled back.

## Producer And Budget Ordering

The healthy 0/3/0/5 schedule still uses exactly twelve work units. At 11 units,
a successful fourth producer is followed by child-reservation rejection,
before any fourth-parent child body. The native prefix ends
`76, 602, 3, 3, 2`. Changing only that producer's guard to zero instead causes
the source arithmetic failure before reservation, ending `76, 602, 3, 3, 0`.

The shared probe rejects either budget diagnostic for expected source faults;
separate entry-budget controls retain limits eight and 64, with 192 healthy
entries. This distinguishes source failure from reservation failure rather
than treating every trap as equivalent. The collision probe first accepts a
smaller callback under the same eleven-unit limit.

The [CLI matrix](../../tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs)
checks fresh compilation, a confirmed cache hit and two source-free restores.
Healthy runs surround failures. State reports and binary/container/host-object,
envelope, lifecycle and packaging identities stay checked. CLI trap output may
be buffered and is checked as a prefix, not substituted for the flushed probe.

## Validation

The selected suite passed 131 distinct tests/checks:

| Selected Set | Passed |
| --- | ---: |
| Fallible/parent/child/root bound and ready-local compiler regressions | 31 |
| Native/reference/oracle profiles and literal-print entry-budget regression | 26 |
| Four f32/f64 fresh/cache/two source-free delivery matrices | 4 |
| Python CI-script regressions | 40 |
| Repository file budgets | 2 |
| Host-absolute-path policy | 1 |
| Development tensor tests | 27 |

Seven controls are new: two source tests, four native/reference/oracle controls
and one CLI matrix. Matrix rows, repeated checks and predecessor receipts are
not added to this total. The four CLI matrices completed in 879.58 seconds.

Structural checks passed for 6,023 UTF-8 text files, 4,727 local Markdown links,
locked offline metadata, formatting and whitespace. Architecture remains 29
packages, six domains and 74 normal/build edges. The proof index stays at 300
lines and the drift registry at 793, within existing limits. No broad unselected
library suite or Linux/GPU execution is claimed.

The rebuilt ordinary CLI passes all 4,209 drift checks, retaining every one of
the 4,199 preceding IDs and adding ten checks. All six source fields match the
runtime report and preserve complete predecessor values and SHA-256 hashes.
Coverage, hierarchy and task lineage are clean, with selection still active/99.
Historical commands pass syntax and inertness checks through stdin without
executing their contents.

## Scope And Next Gate

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. This is bounded native host CPU evidence, not Linux/GPU parity,
arbitrary resource/I/O/async capture, stable ABI or complete native-engine closure.

Next connect the bound producer to parent-carried State changed by completed
children, rather than only callback inputs and parent induction. Test stale
snapshot avoidance, ordered failures, exact reservations and source-free delivery
without granting ancestor-owned or ordinary record-projection permissions.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- fallible_bound_argument_print parent_bound_argument_print child_progress_argument_print parent_reentry_argument_print epoch_local_ child_print_epoch local_scalar_argument_print --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- parent_reentry_argument_print:: epoch_local_argument_print:: sibling_bounds_native_ready_updates_fit_160_but_entry_limit_100_still_rejects --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- parent_reentry_argument_print:: --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete parent-bound fields are retained with the hashes below.
Historical validation commands remain quoted no-op here-document data. Check
syntax and inertness through stdin; do not replay the historic commands.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `d62cc4d0df793e2b689e0bb13c2e412687b6323081ec2960591be580d1b639d1` |
| `next_step` | `9058b9d28bf42ad7dfb668919656f3fb2b8cf378361d99161765c1c6ff6243a6` |
| `blocker` | `b110bc04bc7f5d4c396165cc1b3cdf723603086e8e0f25e21647512b07d95ac7` |
| `next_action` | `8e6230c99dd830a9031128ba2fae28bd0b8ff6189aa7908fb635ea9afc30da17` |
| `validation_command` | `35327b915148ac69069c393d9efbd23a0a2f91bfcc1f5fd0c202823c48c204a9` |
| `expected_artifact` | `923df5b94db3ffcff2e404af4768c897d97d5304e509aff39cc27c1e631e1605` |
