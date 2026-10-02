# Beta 0.15.6 Patch

Date: 2026-10-02. Source version: `beta-0.15.6`, following `1b97ccb1`
(`beta-0.15.5`) and minor baseline `05951bef` (`beta-0.15.0`). Git history
identifies the exact revision. Cargo package versions, protocol identifiers,
public function signatures and capability scores are not bumped for this release.
The [minor snapshot](nuis-beta-0.15.0-snapshot.md) and
[preceding patch](nuis-beta-0.15.5-patch.md) retain their historical evidence.

## Included Changes

Generated scoped helpers admit private typed read-only records when every caller
proves an unwritten input root. Plain, single-carry and multi-carry descriptors
share exact layout, dependency and GLM reads with reference and LLVM execution.
Read-only transport grants no seed-elision or backedge authority. Ordinary owned
and resource captures retain their separate modes; the native scalar profile
still rejects resource-bearing records.

Return invariant proof now covers ordinary child loops without borrowing return
signal authority. Assignment own-read order, registered control identities and
exact compact seed types survive rewriting. Unread total input reconstructions
can disappear, and partly observed copies can project typed fields, but selected
calls, checked fields, codecs, exit indices and evaluated RHS work remain.
Cached nominal layouts and field ranges avoid repeated whole-layout expansion
within the original proof budget; cached shapes do not cache value origins.

Versioned preheader record, subrecord and field snapshots retain evaluated
identities. Opaque results receive independent versions, not assumed identity
semantics. Transactional branch joins preserve only entry-visible, same-typed
relations proved on every arm through ordered origin pairs. Unknown leaves,
swapped arms, separate evaluations and old versions do not establish equality.
Branch-local names cannot escape, and failed joins publish no partial facts.

Per-field parent-entry summaries include zero trips, every intermediate write
and all later trips. Stable leaves retain their origins; varying leaves get
independent entry identities per binding and leaf. First-trip equality cannot
authorize later trips. The 65536-work/64-depth bounds and exact nominal checks
remain; failed nested proofs retain an admitted inner-only plan or the original
body. Original source admission and whole-function revalidation remain required.

The nested-return fixture now uses inner N-4/outer N-1 carried words, or 60/63 at
width 64. Changed-tag and joined-preheader variants use 60/64. Changing both tag
and count or introducing one opaque count arm still rejects at 65 outer words;
the native 64-word limit and return/control slot are not relaxed. Whole private
record input transport begins at width 61 in this fixture, with N-7 mutable
inner fields. This is a private layout choice, not a public ABI threshold change.

Private argument counts remain separate from carried-word widths. Baseline
9/64-word iteration arities are 9/12 and 6/11; the immutable-parameter parent-entry
fixture uses 7/12 at full width. These reductions do not establish a speedup.
Native probes preserve scalar bits, selected traps, shared budgets, immutable
snapshots, overlapping outputs, failure sentinels and zero aggregate allocation
and drop counters in the admitted pure-value profile.

The [value-return contract](../reference/nuis-native-scalar-value-returns-v1.md)
records dated implementation checkpoints. The
[session bridge](../reference/nuis-native-scalar-session-bridge-v1.md) and
[mainline map](../current-mainline-map.md) keep CPU lowering separate from
provider dispatch, resource-bearing state and full application closure.

## Validation Evidence

The latest implementation worktree on local macOS aarch64 passed 1922 distinct
selected tests before release preparation:

* 1774 selected frontend, lowering, walker and NIR-verifier unit tests
* 87 native/reference bridge tests
* 26 ordinary native execution tests
* three source-free CLI workflows covering four artifact variants
* five reference image/window session tests
* 26 development-tensor tests and one host-path policy case

Focused and overlapping reruns are excluded from this total. Source-free checks
cover full-width parent-entry snapshots, narrow nested returns and checked child
break/continue. They preserve repeated events, cache reuse, pre-open tamper
rejection and byte-identical LLVM/state after deleting source, manifest and
original build directories. The rebuilt CLI reports 1668 clean drift checks,
with clean coverage, hierarchy and selected-task lineage at `active/86`.

This is selected local CPU evidence, not a full-workspace test run, fresh
Linux/GPU acceptance, formal-safety certification or measured performance.
The [validation checklist](nuis-beta-0.15.0-release-checklist.md) lists broader
follow-up suites; a listed command does not imply it ran for this patch.
Remote CI acceptance must be checked for the pushed revision separately.

Release preparation reran 26 tensor tests, 20 maintenance-script tests and the
host-path policy case. The live CLI retained 1668 clean drift checks and clean
coverage/hierarchy/lineage. Formatting, diff whitespace, 2974 local documentation
links, 4606 UTF-8 text files and line caps for all 76 changed/new source and
documentation files passed. These checks are separate from the implementation
total; full native suites were not repeated solely for the version/documentation
update.

## Remaining Work

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session` at `active/86`.
Next is stable post-loop snapshot proof and further changing backedges, retaining
checked constructors, opaque calls, observed exits and bounded fallback.
Post-loop writes remain conservatively invalidated. Genuinely incompressible
private state cannot bypass the native limit.

Resource-bearing state, deferred tasks, native provider dispatch, general
interactive cancellation and self-contained application packaging remain
separate work. This patch does not freeze ABI, complete self-hosting or claim
general native application closure. Independent Nustars and Galaxies continue
to consume shared contracts without new implementation dependencies.
