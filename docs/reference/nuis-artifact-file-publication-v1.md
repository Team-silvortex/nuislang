# Artifact File Publication

Status: local CPU regression and publication contract, 2026-10-03.
This is current implementation evidence, not a version or ABI change.

## Reproduced Failure

The original scalar-alias artifact workflow observed one `SIGKILL` without an
exact launch phase. A diagnostic rerun passed, which did not resolve it.

A smaller regression builds two minimal LLVM programs with different exit
codes and alternately restores their cache entries to one executable path.
Before the fix, the second image was killed with signal 9 on the local macOS
aarch64 host. `codesign --verify --strict` nevertheless reported a valid image
on disk. A hard-link test independently showed that restoration mutated the
old file identity.

In-place executable replacement is an actual defect, not an LLVM callback or
alias-analysis failure. Stale host executable/signature mapping state is
consistent with these observations; a specific kernel/AMFI rejection reason
is not established. Read-only system-log approval could not complete because
the approval-review service disconnected. The exact phase/PID of the earlier
isolated failure was not retained and cannot be retrospectively matched.

## Publication Contract

The shared `nuis-artifact` helpers have no ISA, backend, provider, signing,
execution or retry responsibility.

1. Exclusively create a private sibling file on the destination filesystem.
2. Write the full payload, or stream a copy from a regular source file.
3. Apply source permissions for copies and synchronize the staged file.
4. Publish with a same-directory rename, never an existing-identity overwrite.
5. Synchronize the parent directory on Unix and clean owned staging files.

Existing leaf symlinks and directories are rejected, not followed. Failures
before rename retain the old destination. A directory-sync failure after rename
reports failure but does not undo publication. This is not protection against
arbitrary parent-directory mutation or a multi-file bundle transaction.
Staging names do not extend destination names, retaining near-limit file names.
UTF-8 names are tested locally; Unix byte-name coverage is Linux-only because
the local filesystem rejects invalid UTF-8 names.

Cache storage and restoration use streaming atomic file copies, preserving
bytes and source permissions without mutating a previously launched inode.
Cache keys and admission checks are unchanged; no execution retry is added.

`nuis materialize-artifact` uses the same publisher for its embedded host binary.
On Unix the new file is private and non-executable, regardless of previous
destination permissions. The complete restored manifest is verified before
native/session executable permissions are enabled. `nuis-self-contained-image`
remains non-executable data here. Other support files do not acquire a whole
bundle atomicity guarantee.

## Regression Coverage

Tests cover retained identities, streaming bytes/permissions, injected partial
write cleanup, missing sources, directory/symlink rejection, complete old/new
reader views, non-executable writes, UTF-8 names and near-limit names. The native
cache regression executes six alternating restorations with exact exit codes,
without retry.

A frontdoor workflow launches a session, restores its cache, removes the source
and project manifest, then materializes and launches the standalone artifact
four times at the same path. Replacements require fresh identities, byte-exact
binaries, complete matching session states and no staging-file leftovers. A
symlink destination must not modify its referent.

Scalar-alias tests retain variant, phase and input diagnostics. Abnormal
`SIGKILL` records binary SHA-256, file identity and a failure-only macOS signature
check. Explicit `NUIS_NATIVE_WORKFLOW_KEEP_FAILED=1` retains a failed test fixture;
default/successful tests clean up. Arithmetic negative tests require exactly
the open state and cannot accept host `SIGKILL` as expected failure evidence.

The local checkpoint passed 140 artifact-library unit tests (10 publication
tests), six compiler-cache tests, four source-free workflow tests, 26 tensor
tests and one host-path policy test: 177 selected tests, excluding reruns.
The rebuilt CLI reports 1693 clean drift checks and clean coverage, hierarchy
and lineage. All 49 changed/new text files pass UTF-8 and file-size limits;
2992 local documentation links resolve. This is not a full-workspace run.

Run sequentially from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuis-artifact --lib -- --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- cache:: --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuis --test native_session_workflow -- native_cache_and_repeated_materialization native_scalar_alias_copies native_post_loop_snapshots native_full_width_nested_returns --test-threads=1 --quiet
```

These are local host checks, not fresh Linux/Windows certification. The repair
does not widen callback signatures, proof budgets or the persistent-session
score; see the [scalar loop proof](nuis-native-scalar-loop-snapshots-v1.md).
