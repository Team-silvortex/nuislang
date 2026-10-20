# Native Return Storage Candidate Attempts V1

This bounded CPU follow-up extends the
[ordered initialization-prefix proof](nuis-native-return-effect-storage-prefix-v1.md).
Git identity remains beta-0.17.3 at 81dc1669; this is not a release.
The later [ordered reassignment proof](nuis-native-return-effect-storage-reassignment-v1.md)
advances the repeated-prefix-binding boundary below; historical acceptance
counts and the eight-attempt budget retain their own scope.

## Bounded Candidate Attempts

[Discovery](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_storage.rs)
returns eligible record names in declaration order, filtering actual source loop
carries before limiting effectful attempts to at most eight. Unrelated records
do not consume that attempt budget. It does not sort names or use backend,
function-name or record-name conventions. The pure entry still selects only
its established first candidate and rejects interleaved prints.

[Return preparation](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/returns.rs)
keeps the independent payload/pending baseline and its original admission checks.
Each attempt rewrites the unchanged source into its own local plan, then requires
the existing independent storage certificate, including original and candidate
ordered-read/type checks. A failed rewrite or certificate continues to the next
candidate, without modifying the source, baseline, pending identity or publishing
partial helpers. Only a complete passing plan is selected.

The first successful certificate wins. If all attempted candidates fail, or a
usable candidate lies after the eight-attempt limit, preparation retains the
exact independent plan. This is an optimization budget, not a source rejection
or a public ABI limit. Existing preflight node/depth limits still apply to every
certificate. Literal-print discovery keeps initialization dominance; branches,
loops, dynamic prints and other non-binding statements remain barriers.

## Executable Controls

The [source adapter](../../tools/nuisc/tests/native_application_bridge/state_loop_return_shared_cases.rs)
adds actual outer record carries before the established usable record. Those
extra records are read-only captures in the inner loop. Reusing one for a child
return introduces an inner write and fails that loop's ordered-read admission;
reusing the later record passes. Original work, State and print traces retain
the independent source oracle, including caller print 5 and work prints 701/702.

[Candidate units](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_candidates_tests.rs)
check eight step/guard variants and three usable-record names, failed-first and
successful-later certificates, immutable source/baseline, first-success ordering
with deliberately reverse-sorted names, and reversed function storage.
Seven rejected candidates followed by a usable eighth succeed. Eight rejected
candidates followed by a usable ninth retain exact fallback, despite the ninth
having its own passing certificate. Source-valid barriers still block discovery,
the pure first-candidate policy is unchanged, and shared self-read authority
cannot make original-invalid source admissible.

[Execution comparisons](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_storage_execution_tests.rs)
lower both optimized and explicitly independent NIR for all eight variants.
The later-candidate route reduces maximum scoped YIR carry width by four
transport words, comparing real entry values and ordered effects under normal
and reversed YIR storage, including selected failures.
Logical carry reduction is not physical-memory or speedup evidence.

[Native/reference probes](../../tools/nuisc/tests/native_application_bridge/state_loop_return_shared.rs)
extend the existing raw-word, actual-State, local-exit, fault-retention and full
reached-reservation matrices with the failed-first/later-successful source.
Native failures observe flushed prefixes, not memory after abort. Reference
faults retain the last accepted State, enter Faulted and reject further work.
The existing printed-prefix initializer-failure route remains registered.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/effect_call_state_loop_return_shared.rs)
adds one alternative-candidate representative alongside the two preceding
representatives. Each has fresh/cache/two source-free images, unchanged
binary/container/host-object/envelope/lifecycle identity and healthy sequences
around selected fatal events. Original source, project manifest/cache and
original output are removed before restoration. No failed State/completion
publication is accepted; buffered abort stdout is not exact-prefix evidence.

## Remaining Frontier

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
The certificate still rejects repeated same-name bindings in the initialization
prefix, even when independently admitted source reassigns an already initialized
mutable record before entering its loop. Next distinguish an ordered same-record
prefix reassignment from a fresh declaration, constant or parameter write,
preserving exact type, initialization dominance, all fallible effects and
per-candidate read authority. Unproved cases must retain independent fallback.

The eight-attempt cap is deliberately conservative; do not expand it or cross
discovery barriers merely to force an optimization. General resource/I/O/async
capture, wider changing backedges, stable ABI, full heterogeneous closure,
cross-platform/device parity and completed self-hosting remain separate.

## Scoped Acceptance

This follow-up passes 306 distinct focused tests, 10 new: 236 compiler units,
11 native/reference/reservation/initializer-failure/preceding-break compatibility
tests, three fresh/cache/source-free CLI tests, three host fault tests,
24 repository-maintenance tests, two line-budget tests and 27 tensor tests.
The initial six-test candidate probe is included in the final 236-unit result,
not added again. Preceding receipts retain their own historical counts.

The native suite passes on this M2 in 219.22 seconds, including all three new
alternative-candidate matrices and the preceding no-prefix, printed-prefix and
fallible-initializer controls. Actual State handoff, raw float words, local exits,
selected failures and full reached reservations retain their source semantics.
Native abort observations are exact effect-prefix evidence, not post-abort
memory inspection. Per-candidate units separately establish failed-first and
successful-later certificates, stable first-success order, attempt-eight/nine
boundaries and exact fallback without source/baseline mutation.

Three CLI representatives pass in 240.21 seconds, covering 12 delivery images:
each representative has fresh/cache/two source-free images. This is not an
eight-variant source-free matrix. Each representative's own binary/container,
host objects, envelope, lifecycle and packaging identity stay unchanged through
cache/materialization; different source programs need not have equal binaries.
Healthy sequences precede and follow both selected fatal cases, with no failed
State or completion publication. These timings describe regression scope, not
a runtime performance or speedup comparison.

All 3,451 drift checks, locked metadata for 29 packages, workspace Rust
formatting, UTF-8, relative documentation links and source/test/document budgets
pass. The ordinary CLI was rebuilt and matches all six selected tensor fields
and their complete predecessor hashes, with clean coverage, hierarchy and
task-card lineage. The coordinate remains active/99, with a ready task card for
ordered same-record prefix reassignment, not completed application closure.
Complete historical values are retained; shell syntax and an inert-history
probe confirm that preceding validation commands are not executed again.
The older candidate-scope drift check now requires actual-carry filtering and
the pure first-candidate selection rather than obsolete first-only code text.

| Predecessor Field | Complete SHA-256 |
| --- | --- |
| `evidence` | `f9ea382fee22f4acd5641b1ff231f3853b044ac21240154c34fb60240ab94013` |
| `next_step` | `3d2b1ad2f2480a2e29c0888456fdfc3c0ef7ee6b614e13321712b66bcaf2173c` |
| `blocker` | `5d0da4175268deaf3f0a4ba88af7cf3a7f376f2efb51dab66aa69fe9d3667233` |
| `next_action` | `81b8a929928614b12bd66d00b1c7a5700bb449cfe92ff04d8e2b9cccd1ad239e` |
| `validation_command` | `ccfe003a0b52c3665521f28063535c1b52e1a94a5181bdad622c24b39106b22d` |
| `expected_artifact` | `7baeb3931acaabd302c2e2a8817a9fbd0d26aa54c226c20f6a9e518d444ebb8a` |

No Git commit, push, tag or version bump is performed. Existing uncommitted work
and prior regression registrations are preserved. No new Linux, Windows, GPU,
full-workspace semantic, physical-memory/performance or completed self-hosting
acceptance is claimed.
