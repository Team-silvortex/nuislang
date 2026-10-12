# Native Private I32 Ancestor Continuations V1

This is the frozen i32-ancestor checkpoint. The subsequent
[i64-ancestor follow-up](nuis-native-private-i64-ancestor-continuations-v1.md)
extends its exact integer-owner boundary with typed owned full-width fields;
the acceptance numbers and next-task record below remain historical.

The persistent application-session CPU path now supports one exact `i32`
prefix-owned ancestor beside one private `bool` child through bounded terminal
mixed selections and sequential continuations. This extends the
[event-budget checkpoint](nuis-native-private-continuation-event-budget-v1.md)
and reuses the closed [i32-pair codec](nuis-native-private-i32-pair-continuations-v1.md)
with its semantic roles reversed.

The surrounding source helper is still **bool-returning**. An internal selected
region exports one typed `i32` result to its surrounding return prefix, which
then computes the source bool return. This is not a second public result, a
general numeric-return pipeline, or permission to export the private child.

## Single Outward Target

Only exact unqualified `bool` and `i32` owner types are admitted. Root discovery
still requires a proven original prefix write. Delegation carries one target's
write authority, not evidence that a source version has already been defined.
The first RHS captures the entering version; later RHSs use proven current
versions. Retention keeps the incoming owner without inventing source work.

The `i32` orientation admits only one prefix-defined private `bool` child absent
from the entering scope. Existing bindings remain readonly except the nominated
owner. Third writes, extra entering targets, qualifier/type mismatches, const
seals, hidden logical work in ordinary leaves, private-scope leaks and partial
exits remain atomic vetoes. Both source orders remain meaningful: a child RHS
after an owner write observes that new owner, not its pre-branch value.

## Exact Private Transport

The internal carrier stores the signed owner with bias `2147483648` and the bool
child with flag `4294967296`, giving the exact range `0..=8589934591`.
Installation restores the private bool and then the exact signed `i32` owner.
The codec uses widening, canonical bool conversion, arithmetic and comparisons;
it adds no division, remainder, fallible source work or heap allocation.

The original forest is preflighted before installation. Shared continuations are
not copied into each arm, and original statements are charged once under the
existing 64-node / 256-expression / 32-logical-edge limits. Local body/suffix
limits remain 16, selection depth remains eight, and the private selection
capture limit remains 31 plus its gate. The new exact budget test checks
seven nodes / seven expressions / zero logical edges and adjacent rejection,
16/17 statements, 31/32 captures and the depth boundary. Existing whole-forest
and logical-budget tests remain shared regressions.

## Source And Probes

[The Nuis source](../../tools/nuisc/tests/native_application_bridge/return_i32_ancestor_private_bool.ns)
contains typed initialization, signed inversion, branch-current owner/child
updates in both orders, nested selection, a shared suffix, a const seal, unused
effectful work, a bool return and caller continuation. Inactive proof operands
are deliberately zero, so speculative evaluation would fail.

[The independent oracle](../../tools/nuisc/tests/native_application_bridge/return_i32_ancestor_oracle.rs)
replays source order imperatively using signed integers, not the compiler codec.
Five seeds including both signed extremes cross both bool gates, giving 20
cases. The native-only observation reads the actual compiled `answer_work`
argument. It does not replace source computations, helpers, joins or results.
Only observations are flushed before the original fatal trap.

The production literal-print bridge retains explicit constant-site grants and
effect-order validation. Computed prints are not admitted by this extension.
Each callback feeds its actual output into the next callback, with canaries
around the output buffer. Reference failures retain the last accepted state and
allow explicit fault cleanup; native process traps are not recoverable errors.

## Executed Acceptance

The 2026-10-11 serial scoped cohort contains **288 distinct tests**:
227 compiler unit/artifact tests, 24 native bridge tests, seven CLI tests, three
existing host failure-policy tests and 27 tensor tests. Nine units are new.
Repeated or repaired runs do not add units, and the predecessor's 57-unit
event-budget checkpoint is not added to the current count.

The new native matrix runs 40 successful native processes and
18 fatal native processes across normal/reversed inventories, proving 156 accepted callbacks,
exact signed callee arguments, complete source print order and output canaries.
A bad proof traps before callee entry, including before the answer observation
itself. No computation or fatal trap is replaced.

The new CLI matrix checks four images: fresh build, cache hit and two consecutive
source-free restores. It proves 36 CLI event traps and 420 accepted callbacks,
including healthy reentry after each failure. The last open/event state remains
the only publication; later event/close requests and completion are absent after
fatal failure. Executable, policy, envelope, lifecycle and native object identity
are unchanged. Diagnostic-bearing container bytes may legitimately differ
between fresh and cached builds; each image and the restoration input remain
unchanged by their subsequent use.

The ordinary rebuilt CLI verifies 2494 clean registered drift checks / 17354
required patterns, clean coverage/hierarchy/task-card lineage, six source-equal
fields and all six complete predecessor field hashes. The selected cell remains
`active/99`, because broader persistent application-session closure is separate.

Reproduce from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- effectful_selections effectful_scalar_selections return_i32_ancestor_private_bool --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- return_i32_ancestor_private_bool private_event_budget private_event_failure helper_entries_share_exact_budget helper_entries_reject_before_body helper_entries_skip_unselected_paths helper_entries_use_unsigned_limits helper_entries_count_loop_free_roots helper_entries_include_scoped_iterations --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- return_i32_ancestor_private_bool private_event_budget native_effectful_selected_calls_build_cache_restore_and_execute_only_selected_arguments --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-runtime-host --lib --locked -j1 -- native_event_failure_retains_last_state malformed_native_output_and_failed_close native_host_failure_still_requires --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
```

The receipt is macOS CPU execution, not a new device or platform acceptance.
Formatting, whitespace, UTF-8, local Markdown links and the source/test/document
caps are separate checks. No full-workspace or strict-Clippy acceptance is claimed.

## Remaining Boundary

The next selected task is an exact **i64 outward ancestor** beside a private bool
child, using full-width typed transport without lossy scalar packing. Wider or
floating owners, resources, extra public results, arbitrary effects and
partial-exit transport remain separate. This CPU work is not new GPU, Linux or
Windows execution, a performance benchmark, formal memory safety or self-hosting
acceptance. ABI stabilization and full-workspace/strict-Clippy gates remain
independent.
