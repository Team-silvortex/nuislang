# Repository Hygiene

This is the current cleanup policy and audit record, not an alpha-era deletion
wishlist. Current compiler, Nustar and application behavior remains governed by
the [mainline map](current-mainline-map.md) and the development tensor.

## 2026-10-25 Documentation Consolidation

The Git baseline is `2db5c82c` (beta-0.17.8). This documentation-only maintenance
preserves the pending child-epoch implementation and its separate acceptance
receipt; it does not create a release, change compiler behavior or rerun that
receipt's native/device suites.

### Routing And Current Truth

The [dated snapshot](versioning/nuis-beta-0.17.0-snapshot.md) owns exact committed
patch identity. README and the other undated routers link there rather than
copying a patch hash. The snapshot now distinguishes committed root-carry work
from the child-epoch follow-up that was uncommitted at this audit.

The mainline map owns current task selection, the operating checklist owns
validation choices, and the proof index owns the full bounded native history.
Repeated proof-by-proof narratives were replaced with scoped tables and links;
historical receipts and minor-version records were not deleted or rewritten as
fresh acceptance.

| Entry | Before | After |
| --- | --- | --- |
| Repository README | 300 lines | 253 lines |
| Documentation index | 111 lines | 114 lines |
| Current mainline map | 203 lines | 130 lines |
| Reference index | 263 lines | 162 lines |
| Native proof index | 300 lines | 280 lines |
| Beta-0.17 snapshot | 163 lines | 94 lines |
| Beta-0.17 checklist | 144 lines | 111 lines |
| Standard-library index | 193 lines | 85 lines |
| Examples router | 95 lines | 49 lines |

All 224 original proof-index targets, including section anchors, remain linked.
The reference index separates contract families from scoped receipts. Standard
library import policies now match the five package manifests: core/std/
PixelMagic/WitSage use `project-auto`; ns-nova uses `manual-only`. Canonical
project resolution locks/caches are distinguished from the planned resident RC
collector. Examples start with current host CLI/application routes instead of
the beta-0.3 phase anchor or supposed shipped executable bundles.

Two source-only regressions protect dated snapshot ownership and manifest-backed
import-policy documentation. Doc drift checks now guard those roles instead of
pinning the obsolete beta-0.17.3 hash in current routers.

### Validation Scope

This pass passed 56 distinct structural checks: 26 maintenance tests, including
two new regressions, one host-path check, two line-budget checks and 27 tensor
checks. Repeated focused runs are counted once. The final maintenance run took
18.170 seconds; tensor tests took 6.16 seconds and the ordinary CLI rebuild took
8.87 seconds, without new warnings.

Formatting, whitespace, UTF-8 across 5,780 source text files and locked offline
metadata for 29 packages passed. A repository-wide scan of 412 Markdown files
checked 5,819 local file targets without failures; all 224 original proof-index
targets remain linked. These are local-path checks, not external URL or general
Markdown-renderer certification.

The actual rebuilt CLI reports all 3,780 drift checks and 25,626 required patterns
matching, with clean coverage, hierarchy and task-card lineage. All six functional
checkpoint values match both source and their pre-maintenance SHA-256 identities.
The selected coordinate stays active/99 with the same next task. Documentation
routing is not a new runtime or self-hosting milestone; native delivery, physical
device execution, performance and whole-workspace semantic suites were not rerun.

## 2026-10-18 AOT Packer Modularization

This follow-up keeps Git identity at `81dc1669` (beta-0.17.3) and preserves the
existing dirty mainline work. It does not create a release or remove executable
regressions in the name of tidiness.

### Private Ownership Boundaries

The largest remaining packaging frontdoor,
[tools/yir-pack-aot/src/main.rs](../tools/yir-pack-aot/src/main.rs), shrinks from
3,888 to 403 lines. Thirteen private modules now own host planning, backend
artifact/descriptor generation, shader source/expression/texture normalization,
inherited host compilation/frame support, CFFI validation and AppKit source.
The largest extracted module is 676 lines. No replacement line exception is
added; the repository now has 13 remaining historical exceptions, not zero.

The CLI retains orchestration, validation order, profile dispatch and bundle
publication. All thirteen extracted source bodies and the entrypoint bodies
match their predecessors after visibility removal and Rust formatting.
Dependencies use explicit imports and narrowly scoped visibility rather than a
new public facade. Existing C/Objective-C compatibility generation is moved,
not introduced; no new backend grant, ABI or CFFI signature is registered.
The [tool reference](reference/yir-tools-reference.md) maps the new owners.

Four new unit tests freeze seven generated-content fingerprints captured on the
original monolith before extraction. They cover shader variants, kernel
stage/graph artifacts, host sources/fabric plans and shader expressions.
These are change-detection fingerprints, not cryptographic security claims,
proof of byte-identical final linked binaries or new device-execution evidence.

### Focused Acceptance

The complete packager suite passes 28 tests: 25 units, two new CLI integrations
and the existing headless session test. The CPU integration builds and runs an
actual native executable, checks the existing bundle layout/manifest and
observes `12\n` with exit zero. The other integration rejects malformed source
and incomplete/invalid profiles before bundle publication. New subprocesses
are bounded to 60 seconds and temporary project directories have scope cleanup.

The original headless test initially hit the sandbox restriction on binding a
local Unix socket, including before extraction. With local-socket permission,
both the original and refactored test pass Finish/Drain against the existing
mock provider; this is not physical GPU or external-network acceptance.

This follow-up passes 87 distinct focused tests, seven new: 28 packager,
24 repository-maintenance, two line-budget, six current example/host-path and
27 tensor tests. It also passes packager all-target compilation without new
warnings, workspace formatting, UTF-8, relative documentation links, locked
metadata for 29 packages and 3,421 drift checks. The rebuilt ordinary CLI
matches all six tensor fields and complete predecessor hashes; coverage and
task lineage remain clean. Counts from the earlier 72-test organization audit
and 256-test mixed-exit receipt are historical and are not added to this total.

The active/99 coordinate remains
`standard-library/ns-nova/persistent-application-session`. The next functional
task is still an independent effect-prefix proof before typed return-snapshot
coalescing. All six complete preceding values and their SHA-256 identities are
retained, and recorded prior commands remain inert. No new Linux/Windows/GPU,
full-workspace semantic, performance, formal-memory-safety or completed
self-hosting acceptance is claimed. No Git commit, push or version bump is
performed.

## 2026-10-18 Organization Audit

Git records the current series as beta-0.17, with committed patch
`81dc1669` (beta-0.17.3). This maintenance does not create a release, discard
uncommitted mainline work or change compiler semantics.

### Navigation And Evidence

The repository README, documentation index and mainline map had accumulated
duplicated proof-by-proof narratives. Their new sizes are:

| Router | Before | After |
| --- | --- | --- |
| Repository README | 972 lines | 251 lines |
| Documentation index | 326 lines | 111 lines |
| Current mainline map | 786 lines | 125 lines |

The [native session proof index](reference/nuis-native-session-proof-index.md)
retains all 184 focused reference targets from the former routers, including
their section anchors. All 98 existing native documents remain reachable.
Original receipts, source fixtures, regression registrations and historical
acceptance counts stay in place. This removes duplicate navigation prose,
not validation evidence.

The [beta-0.17 snapshot](versioning/nuis-beta-0.17.0-snapshot.md) and
[current checklist](versioning/nuis-beta-0.17.0-release-checklist.md) distinguish
committed Git identity from uncommitted follow-ups. Five current indexes now
agree on those entrypoints; beta-0.16 remains historical.

Three source-only regressions enforce short routers (300 lines), native receipt
reachability and consistent current-series routes. They do not impose that
router cap on reference/history Markdown, which retains its 2000-line policy.

### Source Budgets And Storage

The line-budget table contained 56 legacy entries. Forty-two files no longer
needed an exception and now use the standard source/test limits. Fourteen
oversized files remain, with current exact ceilings. A new ratchet test rejects
obsolete entries and requires budgets to decrease when files shrink.

The remaining debt includes the 3,888-line AOT packaging frontdoor, 2,115-line
shader/ns-nova contract tests and std CLI/network recipes. They remain explicit
refactoring work, not unused modules to delete. Four existing Rust formatting
discrepancies were fixed without changing behavior.

The source/fixture checkout was about 43 MB; generated Cargo output was about
5.1 GiB. Default safe cleanup selected 1,275,584 KiB of incremental/Python
cache paths, reducing the measured root target footprint to about 4.4 GiB.
Hardlinks can share storage, so selected sizes are not a claim of that much new
filesystem free space. Root CLI/test binaries, libraries, locks, downloads,
source and subprojects were preserved. No other repository or system cache was
touched, and no blanket `cargo clean` was used.

### Focused Acceptance

This audit passes 72 distinct focused tests, four new: 23 maintenance tests,
two line-budget tests, fourteen affected compiler units, six current example/
host-path tests and 27 tensor tests. It also passes full-workspace Rust
formatting, UTF-8, documentation links, locked metadata for 29 packages and
3,416 drift checks. The ordinary CLI was rebuilt and matched all six tensor
fields and complete predecessor hashes, with clean coverage and task lineage.

The selected tensor coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Its next functionality task remains the independent effect-prefix proof before
typed return-snapshot coalescing. All six prior values/hashes and inert command
history are retained; maintenance does not increase capability scores.

The earlier mixed-exit receipt's 256-test/native/source-free acceptance remains
that receipt's own evidence, not a fresh full native delivery run for this
cleanup. No Linux/GPU/Windows rerun, performance, formal/global-memory-safety,
completed self-hosting or whole-workspace semantic acceptance is claimed.

## 2026-09-24 Audit

The cleanup is recorded in `05951bef` (`beta-0.15.0`). The
[minor snapshot](versioning/nuis-beta-0.15.0-snapshot.md) separates this repository
maintenance result from native application and provider-execution evidence.

The pre-cleanup workspace occupied about 7.9 GiB. Root Cargo output accounted
for 7.2 GiB; generated outputs below tools and example projects added several
hundred MiB. Source deletion is not the main disk-saving mechanism.

Retired after checking workspace, source, test and documentation references:

* the unused `strategy-ai` workspace crate, whose only compiled API returned a
  placeholder string and had no consumers
* five pre-alpha `check-0.17-*`, `check-0.18-*` and `check-0.19-*` scripts;
  current CI and the [beta checklist](versioning/nuis-beta-0.15.0-release-checklist.md)
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

## Historical Cleanup Validation

CI discovers the maintenance regressions under
[scripts/tests](../scripts/tests). They cover dry-run behavior, explicit full
cleanup, preservation of source/project state/external resources, tracked and
symlinked-path refusal, and source-only checkouts with no generated outputs.

The earlier cleanup audit passed 72 focused CLI/tensor/workflow tests, six source-example
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
