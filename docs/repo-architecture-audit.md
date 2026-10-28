# Repository Architecture And Storage Audit

Audit date: 2026-10-28. Git baseline: beta-0.18.3 at `2c5b4796`.
The pending [i64 XOR follow-up](reference/nuis-native-i64-xor-argument-print-v1.md)
is preserved, not committed, removed or revalidated as a new runtime feature.
This audit separates dependency ownership, source organization and generated
storage. It is not whole-workspace semantic or physical-device acceptance.

## Actual Workspace Graph

The offline Cargo metadata graph contains 29 workspace packages, six
`yir-domain-*` implementation crates and 74 distinct normal/build edges.
Dev-only edges are reported separately and do not define production layering.
Optional and target-specific declared normal/build dependencies are included.
The number of Rust domain crates is not the number of registered Nustars,
hardware backends or executable provider capabilities.

Arrows below mean **depends on**, not execution direction:

```text
yir-domain-*      -> yir-core
yir-syntax        -> yir-core
yir-verify        -> yir-core + built-in domain implementations
yir-exec          -> yir-core + yir-verify
yir-lower-llvm    -> yir-core + CPU implementation + yir-verify
nuisc library    -> semantics/artifact/runtime + syntax/verify/LLVM + shader helpers
nsdb library     -> nuisc + shared contracts + shader helpers
nsld library     -> nuisc + nsdb + artifact/runtime contracts
nuis CLI         -> compiler/debugger/runtime libraries
```

The [repository layout](repo-layout.md) maps directories. The
[mainline map](current-mainline-map.md) owns current task selection and separates
embedded-YIR, native CPU callbacks and private Nsld image routes.

| Owner | Current Responsibility | Boundary |
| --- | --- | --- |
| `yir-core`, `nuis-artifact`, `nuis-semantics` | Shared contracts and semantic/artifact data | Must not reach concrete domain implementations or tool frontdoors. |
| `yir-domain-*` | Domain definitions, verification/execution support | All six currently depend only on `yir-core`; no direct or indirect cross-domain dependency is admitted. |
| `yir-verify` | Built-in registry composition and verification | Linking known domains here is a composition-root role, not a reason to let domains depend on each other. |
| `yir-syntax`, `yir-exec`, lowering and host libraries | Representation, execution, lowering and host boundaries | Implementation crates must not reach tool packages. |
| `tools/nuisc` | Compiler library plus compiler frontdoor | Its location under `tools/` does not mean it is only a CLI. |
| `tools/nsld`, `tools/nsdb` | Linker/debugger libraries and frontdoors | Their compiler dependencies remain explicit debt, not full isolation. |
| `nustar-packages`, `stdlib`, `subprojects` | Registration metadata, Nuis library sources, separately owned future projects | Rust crate counts alone do not certify these surfaces or their backend parity. |

## Enforced Boundaries

[check-workspace-architecture.py](../scripts/check-workspace-architecture.py)
reads locked offline metadata without building. It follows transitive
normal/build paths within workspace members, resolves dependency identity by manifest rather than local
alias spelling, and uses the open-ended `yir-domain-` prefix rather than a
finite list of hardware combinations.

It rejects implementation/domain paths reaching tool packages, foundations
reaching domain implementations, and a domain reaching another domain. Shared
helpers cannot hide those paths. Registry composition, tool-to-library edges
and dev-only test dependencies are not falsely forbidden.

```sh
python3 scripts/check-workspace-architecture.py
python3 scripts/check-workspace-architecture.py --json
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s scripts/tests -p test_workspace_architecture.py -v
```

JSON manifest paths are repository-relative. The CI regression tests use a
fake Cargo entrypoint and do not need an installed Rust toolchain. The real
architecture check runs after Rust installation and locked dependency fetch,
before workspace compilation. This preserves the source-only cold-start gate.
The audit does not prove Rust module-level ownership, runtime registry coverage,
ABI compatibility, GLM correctness or complete ecosystem decoupling.
External registry/path-package internals are outside `--no-deps` metadata and
require separate dependency review; this is a workspace boundary check.

## Remaining Organization Debt

1. **Compiler/linker/debugger aggregation.** `nsld` and `nsdb` depend on the
   `nuisc` library in normal and build dependencies; `nsld` also depends on
   `nsdb`. Compiler, debugger and CLI packages directly use shader helpers.
   Extract reusable artifact/debug/lowering contracts by responsibility when
   their owners are clear, rather than moving code or deleting APIs blindly.
2. **Concentrated tensor ancestry.** At this audit,
   `tools/nuis/src/dev_tensor_data.rs` is 1,744,588 bytes across 778 lines.
   Its long complete checkpoint strings evade a line-count-only view. A future
   versioned migration should separate compact current data from immutable
   historical payloads while retaining complete values, hashes and inert
   command history. Update readers and drift ownership before moving data;
   truncation or line-budget tricks are not an acceptable migration.
3. **Legacy line exceptions.** At the initial audit, seven Rust test files and
   six Nuis recipes remained in the exact ratchet table. The subsequent
   [shader/ns-nova split](repo-cleanup-candidates.md#2026-10-28-shadernova-contract-suite-split)
   removes its 2,115-line exception, leaving six Rust tests and six Nuis recipes.
   These are live regression/source surfaces,
   not unused files. Split by responsibility and lower/remove each exception
   only after preserving registrations, bodies and focused acceptance.
4. **Repeated proof harnesses.** Scalar print families repeat transport and
   delivery mechanics. Shared harness extraction should keep mathematical
   oracles independent from the compiler and preserve per-family source
   permissions, raw words, fault prefixes, entry counts and budgets.

No live source, test, historical receipt, subproject or dependency source is
deleted by this maintenance. No legacy exception is silently widened.

## Storage And Build Policy

The initial workspace footprint was 5,399,936 KiB. Root Cargo output accounted
for 5,302,536 KiB; the source/fixture tree excluding Git and target was about
59 MiB, and Git occupied about 37 MiB. Source deletion is not the primary
storage remedy.

The existing safe cleaner selected 28 ignored outputs totaling 1,404,064 KiB:
incremental state and hashed debug build/test executables. After cleanup,
`target/` measured 4,139,800 KiB, an observed reduction of 1,162,736 KiB
(about 1.11 GiB). Hardlinks mean selected bytes and freed bytes are not equal.
All six root CLI hashes and all 21 pre-existing dirty-file hashes matched
immediately after cleanup. Libraries, Cargo downloads, source, project settings,
Git history, shared temporary storage and other projects were not removed.

Filesystem availability rose from about 5.3 GiB to about 42 GiB during this
interval. That change exceeds the removed target footprint and must not all be
attributed to this cleanup; OS reclaim and unrelated activity are not measured
by this workspace audit. Validation can recreate selected build products.

After validation, a second conservative cleanup retired six recreated hashed
executables. The final root target footprint is 4,172,992 KiB (about 3.98 GiB),
net 1,129,544 KiB below the initial target footprint (about 1.08 GiB). The safe
build-binary dry run now selects zero outputs, all six latest root CLIs remain
byte-identical across that cleanup, and incremental entries remain zero.
Filesystem availability at the final sample is about 40 GiB; the same attribution
limit applies to that system-wide figure.

Developer and test profiles now default to `incremental = false`, matching
the preceding focused validation environment. Existing debug information,
optimization levels and release configuration are unchanged. Repeated edits
can rebuild more work; developers may explicitly opt in with
`CARGO_INCREMENTAL=1` when that tradeoff is useful. This is a build-storage
policy, not an implementation of the planned Nuis-rc collector.

Use [disk-audit.sh](../scripts/disk-audit.sh) and inspect
[disk-clean-safe.sh](../scripts/disk-clean-safe.sh) before deletion. Stop builds
and tests first. The default is read-only; `--build-binaries --apply` retains
root CLIs and libraries, while `--workspace --apply` explicitly removes all root
build output. Docker/CUDA-heavy work remains a remote Linux preference.

## Maintenance Acceptance

67 distinct structural/maintenance checks passed: 37 Python maintenance tests
(including 11 new architecture/profile/CI regressions), 27 tensor tests, two
line-budget tests and one exact host-path integration. The real workspace audit
reports 29 packages, six domain crates, 74 normal/build edges, five dev-only edges
and no policy violations. Empty-Cargo-home offline metadata also succeeded; this
is not a rerun of the full Linux cold-build gate.

Formatting, whitespace, UTF-8 across 5,970 text files, 4,639 local documentation
links and locked offline metadata pass. The ordinary CLI was rebuilt with
incremental environment overrides removed; no incremental entries were created.
It reports 4,135/4,135 drift checks passing, clean coverage/manifest/milestone/
hierarchy/task lineage and a ready selected active/99 task. Source/runtime values
and the complete predecessor chain remain identical.

| Functional Field | Unchanged SHA-256 |
| --- | --- |
| `evidence` | `84f97fe616c0ead3324a28bcb31fa7446183e2b93d80fa7d49dc8312964507fa` |
| `next_step` | `df2148170098b26bf5950c763a4843923a75ae98ffc87eb761212612f3a4b7e7` |
| `blocker` | `50ef761c1a24e7775a06543b1e273739d68da83f679c9b8a3b95ecc88a29dbce` |
| `next_action` | `adfcbff1688b38fff828dfe9e8ecdeca4e9f75db1aafb8b815dea79d34c493cc` |
| `validation_command` | `3a0534058a7b8e4a79d823aaa6e96f0f59e973e692c5896b85c63a726f2588e0` |
| `expected_artifact` | `d376881a392b624e3a48ea64cf099e04c84ba454eb892cb54c1b66deb722b602` |

The six functional tensor fields are retained byte-for-byte with their hashes.
The pending XOR receipt keeps its own executed totals; maintenance does not
rerun or claim its full native delivery suite. Functional selection remains
active/99 with the source-local temporary audit as the next task. New architecture
guards update diagnostics, not capability scores. No commit or version bump is
performed.
