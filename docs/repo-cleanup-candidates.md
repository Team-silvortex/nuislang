# Repository Hygiene

This is the current cleanup policy and audit record, not an alpha-era deletion
wishlist. Current compiler, Nustar and application behavior remains governed by
the [mainline map](current-mainline-map.md) and the development tensor.

## 2026-09-24 Audit

The pre-cleanup workspace occupied about 7.9 GiB. Root Cargo output accounted
for 7.2 GiB; generated outputs below tools and example projects added several
hundred MiB. Source deletion is not the main disk-saving mechanism.

Retired after checking workspace, source, test and documentation references:

* the unused `strategy-ai` workspace crate, whose only compiled API returned a
  placeholder string and had no consumers
* five pre-alpha `check-0.17-*`, `check-0.18-*` and `check-0.19-*` scripts;
  current CI and the [beta checklist](versioning/nuis-beta-0.14.0-release-checklist.md)
  replace their historical gate role
* generated window/kernel bundles under `examples/bins/`, including host
  shims, LLVM/IR dumps, stale manifests and host executables
* the checked-in executable under `tools/yir-preview-macos/build/`;
  its optional preview source and build scripts remain available

The example sources remain. Generated artifacts are not ABI compatibility
fixtures unless an explicit test identifies them as such. Current snapshot,
native-session, Nsld and provider regressions remain intact.

CLI unit-test fixtures now own their temporary project and derived default
build/release outputs. Scope exit and panic unwinding reclaim those paths;
independent fixtures remain isolated. Artifact protocol and provider-runtime
tests are separated without removing cases.

The Rust module audit checked Cargo targets, ordinary modules, path attributes,
inline test scopes and macro-declared Nsld test modules. A missing ordinary
`mod` line alone is not evidence that a file is unused.

After cleanup, a cold build of all 29 workspace packages and focused regressions
left the workspace at about 1.93 GiB, including 1.81 GiB of fresh build output.
The net reduction was about 6 GiB; available volume space rose from about
11 GiB to 17 GiB. These are measured local results, not a fixed cache budget.

## Storage Boundary

Keep source, intentional fixtures, package manifests/locks, current regression
tests, minor-version history and the separately owned `vulpoya`/`yalivia`
subprojects. Do not remove tests just because their original bug is old.
Git history is retained; this cleanup does not rewrite repository history.

Rebuild generated example bundles into `target/example-builds/`, rather than
shipping host-specific binaries beside source. The
[artifact workflow](reference/nuis-native-artifact-workflow.md) uses that route.
The former `examples/bins/` area now contains only a rebuild guide.

Default cleanup removes example `.nuis/cache`, tool/crate-local `target`
directories, incremental state, optional preview output and maintenance Python
cache. It preserves other `.nuis` state, the root CLI binaries, libraries,
package downloads and all subprojects.

## Cleanup Commands

Stop builds and tests first. From any working directory, the scripts resolve
their own Git workspace; they do not search siblings, home caches, shared
temporary directories or Docker state.

```bash
bash scripts/disk-audit.sh
bash scripts/disk-clean-safe.sh --build-binaries --verbose
bash scripts/disk-clean-safe.sh --build-binaries --apply
```

`--build-binaries` also removes hashed debug/release build/test executables.
Root CLI binaries and dependency libraries remain; Cargo regenerates removed
executables when needed. Reported candidate sizes may share hardlinked storage.

For a deliberately cold rebuild, inspect `--workspace --verbose`, then apply
`--workspace --apply`. That also removes the complete root `target/`.
No option cleans other projects, shared Cargo downloads or Docker.

The cleaner defaults to dry-run and preflights every candidate before deletion.
Tracked paths, non-ignored paths and paths through symlinks fail closed. Do not
run it concurrently with builds or other writers.

## Validation

CI discovers the maintenance regressions under
[scripts/tests](../scripts/tests). They cover dry-run behavior, explicit full
cleanup, preservation of source/project state/external resources, tracked and
symlinked-path refusal, and source-only checkouts with no generated outputs.

This audit passed 72 focused CLI/tensor/workflow tests, six source-example
compilation tests and 20 maintenance tests. The rebuilt CLI reported 1,414
passing drift checks with clean coverage and hierarchy validation. This is a
focused cleanup regression result, not a claim that the entire test suite or
every hardware backend was executed.

Run the same checks locally:

```bash
python3 -m unittest discover -s scripts/tests -v
python3 scripts/check-text-encoding.py
bash scripts/check-doc-links.sh
cargo metadata --locked --no-deps --format-version 1
```

The [development tensor](reference/nuis-development-tensor.md) keeps the current
application mainline and its evidence boundaries unchanged by repository cleanup.
