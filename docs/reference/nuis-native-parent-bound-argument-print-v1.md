# Native Parent Bound Argument Print V1

Audit date: 2026-11-04. Checkout baseline: `dc87ec43`; latest version-label
commit: beta-0.18.4, `ceaa2caa`. This worktree follows the
[child-progress receipt](nuis-native-child-progress-argument-print-v1.md).
No release, version bump or commit is created here.

## Parent-Dependent Child Bounds

The [Nuis witness](../../tools/nuisc/tests/native_application_bridge/parent_bound_argument_print.ns)
computes an immutable child bound on every parent entry. The pure noinline
`child_count` helper returns zero on even parent indices and a growing bound
on odd ones. Base three over four parents gives `0, 3, 0, 5`; base zero gives
`0, 0, 0, 2`. A skipped child loop is not evidence that later entries can be
skipped or that its bound-producing helper can be omitted.

Each reached parent still performs its identity update and constructs a fresh
child record. Resumed work starts with the left field, not the preceding active
parent's final right field. Guards vary by parent and child index. Poison at a
skipped parent's would-be first child stays inert, while later reached
initializer/lazy-use/update faults retain their ordered prefixes.

No production lowering permission changes. The bound is an ordinary ready
local, the induction value remains ready only within its own iteration, and
the record projection keeps its existing epoch ownership. Source controls retain
unready-record, alias and branch refusals and additionally reject a child body
that mutates its own loop bound.

## Independent Model And Execution

The [model](../../tools/nuisc/tests/native_application_bridge/parent_bound_argument_print_cases.rs)
alternates an active flag and grows the next active count by two, independently
of the source's modulo expression. Parent guard bases advance by sixteen and
child offsets advance one at a time. The existing independent bit-by-bit
single-child oracle supplies values and faults; source arithmetic is not
reimplemented by invoking generated code.

Each f32/f64 kind has 24 raw sequences, eight finite sequences and
nine fault rows. The failure set reaches initializer/lazy-use/update after four or five
completed children, eager-false and zero-word-lazy faults after seven, and a
tail failure after eight. The zero-work gap before resumed work is observable
through separate parent and child completion markers.

The [shared native/reference driver](../../tools/nuisc/tests/native_application_bridge/parent_reentry_argument_print.rs)
checks raw State, output order, normal/reversed YIR and faulted reference
sessions. Native marker 602 observes actual bound-helper arguments, including
zero-work parents, without replacing source computation. Reference counts use
the sum of actual child counts; bound calls equal reached parents. Existing
fixed-bound profiles now use the same count API and are rerun.

The callback ABI still has eleven input words; the profile selector and bound
model are test-only data. Successful native probes check State words and output
canaries. Fault probes check exact flushed prefixes and absence of callback
success publication; reference sessions additionally retain their last accepted
State through a rejected further event and faulted close. This does not claim
rollback of already performed output effects or a general memory-safety proof.

## Exact Budget Boundary

Four parent iterations plus three and five children require exactly 12 units
of shared loop work. The normal matrix succeeds at that existing fixture limit.
At 11 units, the fourth parent's bound helper has returned, but its five-child
reservation fails before the first child marker. The expected prefix ends
`76, 602, 3, 3`, with no successful callback publication afterward.

The [shared probe](../../tools/nuisc/tests/native_application_bridge/word_session_probe.rs)
distinguishes loop-work exhaustion from helper-entry exhaustion in stderr and
requires the exact expected work-rejection prefix. Entry limits eight and 64
retain their separate refusal controls; healthy runs keep 192 entries. Neither
the producer's global limits nor its accounting rules change. A pre-existing
literal-print budget test also reruns to cover the non-computed policy branch.

The [CLI profile](../../tools/nuis/tests/native_session_workflow/parent_reentry_argument_print.rs)
uses fresh/cache images and two restores after deleting source, manifest and
project cache. Healthy runs surround faults; binary/container, host-object,
envelope, lifecycle and packaging identities remain checked. Trapping CLI
stdout can be buffered, so its prefix check does not substitute for the exact
direct native trace.

## Validation

The final selected suite passed 124 distinct tests/checks:

| Selected Set | Passed |
| --- | ---: |
| Parent-bound, parent/root/child epoch and ready-local compiler regressions | 29 |
| Native/reference profiles and literal-print entry-budget regression | 22 |
| Three f32/f64 fresh/cache/two source-free delivery matrices | 3 |
| Python CI-script regressions | 40 |
| Repository file budgets | 2 |
| Host-absolute-path policy | 1 |
| Development tensor tests | 27 |

Seven controls are new: two source tests, four native/reference/oracle controls
and one CLI matrix. The existing literal-print budget regression is additionally
selected for the shared probe change. Matrix rows, preliminary documentation
drift failures, repeated runs and earlier receipts are not added to this total.

The three CLI matrices completed in 582.38 seconds. Structural checks passed for
6,019 UTF-8 text files, 4,718 local Markdown links, locked offline metadata,
formatting and whitespace. Architecture remains 29 packages, six domains and
74 normal/build edges. The proof index stays at 300 lines and the drift registry
at 790, within the existing limits. No broad unselected library suite or
Linux/GPU execution is claimed.

The rebuilt ordinary CLI passes all 4,199 drift checks, retaining all 4,190
previous IDs and adding nine parent-bound checks. All six source fields match
the runtime report and retain complete predecessor values and SHA-256 hashes.
Coverage, hierarchy and task lineage are clean, with selection still active/99.
Historical command data passes syntax and inertness checks through stdin
without executing its contents.

## Scope And Next Gate

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. This is bounded native host CPU evidence, not Linux/GPU parity,
arbitrary resource/I/O/async captures, stable ABI or complete native-engine
closure.

Next is failure in the bound-producing helper itself, including a parent that
would otherwise have zero child work. This witness's bound producer is total
for its tested inputs. A zero-trip result cannot justify skipping evaluation
that must happen before that result exists.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- parent_bound_argument_print child_progress_argument_print parent_reentry_argument_print epoch_local_ child_print_epoch local_scalar_argument_print --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- parent_reentry_argument_print:: epoch_local_argument_print:: sibling_bounds_native_ready_updates_fit_160_but_entry_limit_100_still_rejects --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- parent_reentry_argument_print:: --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete child-progress fields remain lossless with these hashes.
Historical validation is quoted no-op here-document data, checked through stdin
for syntax and inertness rather than replayed.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `710b812db071a7c825b42b9cca2ba7fdfdf8e9f874eb91009f011c0cba61e1ae` |
| `next_step` | `e33eec24c45131db176ba69728f7229b53286c3d7783fa0da7c2aadd96874f85` |
| `blocker` | `e7065949ba400a5caea8b79f3378ec582c8984796e2f8b58303a24907febcb0c` |
| `next_action` | `ad02b7c60278e69d72347067e39c6060b9c5f3d896ae9ad1aec97a306e290c54` |
| `validation_command` | `fa050613f81693b25dc9064f7d70967775703f6985a65a2fbd5fe4811e93edca` |
| `expected_artifact` | `1217356b33e9f6e48dfab90413c29d090d30fabef83958224ba2f75c8d249e78` |
