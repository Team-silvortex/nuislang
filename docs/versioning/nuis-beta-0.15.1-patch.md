# Beta 0.15.1 Patch

Date: 2026-09-24. Source version: `beta-0.15.1`, following the documentation
checkpoint `9ec40a40` and minor baseline `05951bef` (`beta-0.15.0`). Git history
identifies the exact revision; Cargo package and protocol versions are unchanged.
The [minor snapshot](nuis-beta-0.15.0-snapshot.md) remains a historical baseline,
not a claim about later changes or a compatibility freeze.

## Included Changes

Private native capture projection now preserves lexical identities across
same-name branch and loop locals. Scope-confined record rebindings retain their
preceding values and old aliases without merging sibling bindings. Signature and
call-site rewrites commit together only after a supported field reduction; whole
record uses and computed callers retain their conservative path.

Single-definition loop aliases may expose fields only from proven invariant,
unwritten pure-value inputs. Generated scoped helpers opt into that projection
separately, retaining induction, carry and break-control identities. The wide-loop
fixture reduces its iteration helper from 66 physical arguments to four while
leaving the 64-argument public helper unchanged. Native argument and work limits
are not widened.

Non-loop child scopes proven to return can now keep separate snapshot versions
when writing ancestor records. Parent continuations and old aliases retain their
preceding values. The exit proof is iterative and requires exact pure-value types;
break/continue and zero-trip loops cannot certify a returning scope. Selected
constructor work still executes even when its resulting record is unused.

The [value-return and capture contract](../reference/nuis-native-scalar-value-returns-v1.md)
records implementation details, direct regressions and source/native composition
separately. The shared 64-field returning-child fixture is composition evidence,
not a claim that the source pipeline previously rejected that program.

## Validation Evidence

The final implementation validation on macOS aarch64 passed 706 selected tests:

* 629 lowering unit tests
* 40 selected native application bridge tests
* eight sparse-capture CLI build/cache/restoration workflows
* two ordinary native snapshot tests
* 26 development-tensor tests
* one host-absolute-path policy test

Overlapping reruns are excluded. Earlier intermediate checkpoints, including the
broader scoped-helper test selection, remain separately recorded in the reference
document and are not added to this total. CLI workflows retain actual native
execution, cache reuse, tamper rejection, selected traps and source-free restoration
with identical LLVM and lifecycle state.

The fresh CLI reports 1429 clean drift checks and clean coverage, hierarchy and
task lineage. UTF-8, local documentation links, changed-source formatting and
changed-file line limits also pass. These are selected local results, not a
full-workspace run, fresh Linux/GPU acceptance, formal proof or measured speedup.
Use the [validation checklist](nuis-beta-0.15.0-release-checklist.md) for broader
follow-up checks rather than treating its commands as already executed evidence.

## Remaining Work

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session` at `active/86`.
The next task is to project field demand through fallthrough record joins while
preserving branch-selected values and loop backedges. General loop-written
snapshots, resource-bearing state, provider dispatch and self-contained application
packaging remain separate boundaries. This patch does not claim complete native
application closure, stable ABI or completed compiler self-hosting.

The [mainline map](../current-mainline-map.md) and live development tensor remain
the current task-selection surfaces; a version increment does not raise scores.
