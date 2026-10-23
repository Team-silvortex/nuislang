# Native Return Storage Branch-Selected Helper Updates V1

This bounded CPU follow-up extends the
[reached-loop witnesses](nuis-native-return-effect-storage-loop-calls-v1.md).
The inspected Git identity remains beta-0.17.4 at bef7a23d. No release is made.

## Branch-Selected Multi-Field Updates

The seventh source mode, `BranchHelperCalls`, replaces three complete numeric/
bool/f32/f64 update groups with runtime-selected Nuis helper paths. The outer
and unselected-child sites choose by outer parity; the inner site chooses by
the active child index. Parent-local break/continue updates retain the previous
loop helpers, so the new calls compose with established local-exit handling.

The [source adapter](../../tools/nuisc/tests/native_application_bridge/state_loop_return_shared_branch_calls.rs)
inserts real [Nuis helper declarations](../../tools/nuisc/tests/native_application_bridge/state_loop_return_shared_branch_helpers.ns).
Even updates capture an unchanged record, then toggle bool, update the integer
and negate both floats in the final callee. Odd updates toggle bool and negate
f32 in the aggregate argument helper, then update the integer and negate f64 in
the final callee. The odd result lists fields in a different source order.
Both compositions match the established source oracle without extra integer
offsets or floating arithmetic that would change signed-zero/NaN semantics.
Successful even effects are `[910, 911, 914, 912, 913]`; odd effects are
`[920, 921, 924, 922, 923]`. These calculations are Nuis code, not foreign-language
application wrappers. Rust is only the independent test oracle/harness.

Existing typing, call catalog, read authority, storage certificates, candidate
budget and exact independent fallback already admit the route. No production
call, purity, ownership, GLM or reservation authority is expanded. The production
module edit only registers tests.

## Unselected Poison And Semantic Roles

Each even branch has raw `0 / (1 - index % 2)`-style arguments; the odd branch
uses `0 / (index % 2)`, with the appropriate active selector at each site.
The chosen branch is safe, while evaluating the unchosen branch's arguments
would divide by zero. These are live argument expressions, not unreachable
dummy helper declarations. Mirrored guards reverse both predicate polarity and
branch placement while preserving the same selected calculation.

The adapter identifies each semantic site by its original update expression,
not the textual `flipped_N` index: mirrored source can place the unselected
branch before the child loop. A regression checks both arrangements explicitly.
The source oracle continues using semantic assignment hooks and never expands
flat numeric traces, including when observed raw words collide with markers.
The unselected-child hook now supplies its actual outer index; the preceding
loop mode ignores that index at this site and keeps its prior behavior.

[Certificate controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_branch_call_tests.rs)
rename helpers/storage across eight variants, preserve immutable source and
independent pending identity, and deny removal of any of the eight branch
callee grants. Pure admission stays separate. Type-valid swapped branches,
duplicate calls, hoisted calls and changed arguments are checked against the
independent structural certificate. Candidate fallback and original-invalid
read rejection remain required.

[Execution comparisons](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_execution_tests.rs)
compare independent/shared entry values, exact selected effects and failures
in normal/reversed YIR storage for eight variants. The four-word maximum scoped
carry reduction remains logical YIR evidence, not measured physical memory or runtime speedup.

## Selected Later Failures

The inner path passes `root + 21 - index` to its second helper and
`after + 22 - index` to its final callee. Selected even failures occur at index 2,
odd failures at index 3, after successful prior inner work in both step
placements. Each path has separate argument and callee failures.

[Native/reference tests](../../tools/nuisc/tests/native_application_bridge/state_loop_return_shared.rs)
compare full flushed native traces, whose terminal suffixes are:

```text
even argument: [..., 10, 910, 911]
even callee:   [..., 10, 910, 911, 914, 912]
odd argument:  [..., 10, 920, 921]
odd callee:    [..., 10, 920, 921, 924, 922]
```

No later argument/callee/completion effect or normal callback publication may
appear after its selected failure. Normal/reversed reference sessions retain
the last accepted actual State after three prior callbacks, enter Faulted,
reject later work and refuse successful completion after close. Native trap
observations do not inspect memory after process termination.
Full raw-word, selected local-exit, inactive-bound and reached-reservation
matrices include the new mode, alongside the preceding loop mode as regression
coverage. These remain bounded local CPU witnesses, not a global memory-safety
or complete heterogeneous-closure proof.

## Source-Free Delivery

### Inclusive Entry Budget

The branch fixture explicitly declares 128 inclusive YIR function entries per
callback; preceding modes retain 100. This counts generated control helpers,
not only source-level calls, and remains independent of loop-work reservations.
The production [default and rejection guard](../../crates/yir-lower-llvm/src/native_session/helper_entries.rs)
are unchanged. A long healthy branch
callback exhausts the previous test fixture's 100-entry budget. A dedicated
normal/reversed native control keeps that exact limit and requires a budget
diagnostic, trap and proper source-effect prefix before any callback result.
The full branch raw-word matrix separately succeeds with the declared 128.
No unselected poison is removed and no unlimited allowance is introduced.

The shared [native probe](../../tools/nuisc/tests/native_application_bridge/word_session_probe.rs)
now reports status and a bounded stdout tail on failure,
and identifies entry-budget rejection on stderr only. Source stdout is unchanged.
This distinguishes a configured budget failure from an illegal eager branch,
wrong payload or arithmetic trap instead of masking them behind a bare signal.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_shared.rs)
adds one branch-helper representative. Focused acceptance runs that new source
and the preceding loop-helper source through fresh build, cache hit and two
restorations after removing source, manifest, project cache and original output.
Each program retains its own binary/container, host objects, envelope,
lifecycle and packaging identity. The new program selects all four branch-
failure cases, preceding prefix-helper failures and selected loop/fallback
failures in each image, without failed State or successful completion output.
Dedicated tail failures remain in the full native/reference case matrix,
not an extra CLI `tail = 0` selection. Exact fatal stdout belongs to flushed
native probes, not buffered CLI output. This is one new source-free
representative, not all eight variants or a rerun of all seven CLI modes.

## Remaining Frontier

The [record-dependent bound follow-up](nuis-native-return-effect-storage-bound-calls-v1.md)
now owns executable checks for the next boundary below; its own frontier
advances to sibling reached-loop reuse and separate reservations.

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Next validate a completed selected record-helper result determining a reached
child-loop bound: evaluate the bound once and only on the reached path, preserve
full reservation order, and retain argument/callee/bound failure boundaries,
actual previous-State bits and native/source-free replay. Diagnose the current
lowering before widening any profile; preserve original/shared read authority,
the current catalog, pending identity, bounded candidates and exact fallback.

Resource/I/O/async capture, wider changing backedges, stable ABI,
device/cross-platform parity and completed self-hosting remain separate gates.

## Validation

Focused acceptance passes 326 distinct tests, including 12 new tests:
256 compiler units (239.55 seconds), 12 native/reference tests (226.04 seconds),
two CLI delivery tests (342.77 seconds), three host-fault tests,
24 repository-maintenance tests, two line-budget tests and 27 tensor tests.
The delivery tests cover eight fresh/cache/restored images across two source
representatives. Native acceptance includes the new branch mode, the preceding
loop-helper mode and original parent-break compatibility, not all seven modes.

The ordinary CLI rebuild and live tensor audit pass: all six current fields
match source, all six complete predecessor hashes verify, the selected task card
is ready, and coverage, lineage and inert validation history are clean.
Static drift validation passes 3,486 checks covering 23,909 required patterns.
Locked metadata contains 29 packages. Formatting, UTF-8 validation, local
documentation links, source/test line budgets and whitespace checks pass.

The first probes exposed a mirrored-fixture semantic-site mapping mistake and
the explicitly configured 100-entry test-budget exhaustion. Both are covered
by regression controls above. Neither is reported as a production lowering
repair: admission authority, production entry guards and defaults are unchanged.

The focused executable routes are:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- iteration_effects control_loops scalar_helpers scalar_control state_loop --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_loop_return_shared::shared_return_storage_branch_helper_calls state_loop_return_shared::shared_return_storage_loop_helper_calls state_loop_return_parent_mixed_keeps_preceding_parent_break_native_profiles_unchanged --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- effect_calls::state_loop_return_shared::shared_return_storage_branch_helper_calls effect_calls::state_loop_return_shared::shared_return_storage_loop_helper_calls --test-threads=1 --nocapture
```

Each current tensor field retains its complete predecessor with a separate
SHA-256 and explicit checkpoint marker. Historical validation commands remain
inside an inert here-document rather than being executed again.

| Predecessor Field | Complete SHA-256 |
| --- | --- |
| `evidence` | `f1843500fe8dd4f06116f532224b510736e76d89acd4c0972e020246306f0c3b` |
| `next_step` | `14f7aef5c3eb74926a1c1e4d4101b1ece9783fd4568f25c7ca8319a35ffd28e1` |
| `blocker` | `865a8df34dfefb79e317449dc31bc3d8c97d75148f92c0a38e0843328cfcf5aa` |
| `next_action` | `3dc3c1a52bdd00c51cedbd98c55a855cf6f9628e7bb9bf469211bc3228b07213` |
| `validation_command` | `1756fc5ebb7f496ed9b98e10e48da5257a3fca369354371de60326a72cf03393` |
| `expected_artifact` | `486cde1c972409ddd033c560e3c2388c37ee631deb4d7f9963f1d02656166088` |
