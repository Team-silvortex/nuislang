# Beta 0.15 Validation Checklist

This is the operational companion to the
[beta-0.15 snapshot](nuis-beta-0.15.0-snapshot.md), anchored to
`05951befc70d6a145e4978a3ff8909737e5c4fbb` (`beta-0.15.0`, 2026-09-24).
Commands below are validation instructions, not blanket claims that every suite
was rerun during documentation synchronization. Record the actual revision and
worktree changes, platform, prerequisites, command, exit status and skip counts.

## Documentation And Version Alignment

- [ ] Route README, documentation/reference indexes, the mainline map and
  versioning index to beta-0.15; keep beta-0.14 and beta-0.12 anchors historical.
- [ ] Keep the existing Git version authoritative. Do not change Cargo package
  versions, protocol versions or capability scores merely to match the release.
- [ ] Keep documentation drift guards aligned with current-line references and
  retain `standard-library/ns-nova/persistent-application-session` at `active/86`.
- [ ] Describe private native value returns and capture projection separately
  from ordinary owned/resource ABI, public signatures and native provider dispatch.
- [ ] Retain the distinction between source coverage, local execution, remote CI
  and hardware evidence. Historical success is not acceptance for a new revision.

## Repository And Documentation Checks

From the repository root, with locked dependencies already available:

```sh
export CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=1
python3 -m unittest discover -s scripts/tests -v
python3 scripts/check-text-encoding.py
bash scripts/check-doc-links.sh
cargo fmt --all -- --check
git diff --check
cargo test --locked -p nuis --bin nuis -j 1 -- tensor --test-threads=1
cargo run --locked -p nuis -- dev-tensor --json
```

Check clean drift, coverage, hierarchy and lineage, plus unchanged goal selection
and scores. A task status is scoped to its milestone, not whole-engine completion.
New untracked documents must also be valid UTF-8 and use repository-relative links.

The [Build workflow](../../.github/workflows/build.yml) additionally installs
stable Rust, fetches locked dependencies, runs
[host-path portability](../../scripts/check-host-absolute-paths.sh) and builds
the workspace on Ubuntu. It does not run all compiler, native-session or GPU tests.
Inspect the remote result for the actual commit before reporting CI success.

## Cleanup And Cold-Rebuild Regressions

Stop builds/tests before inspecting or applying cleanup. The cleaner defaults to
dry-run, rejects tracked/non-ignored/symlinked candidates and must not touch sibling
projects, subprojects, global caches, Docker or Git history. Do not delete live
regressions just because their original bugs are old. Follow the
[cleanup policy](../repo-cleanup-candidates.md) before opting into a cold build.

```sh
bash scripts/disk-audit.sh
bash scripts/disk-clean-safe.sh --workspace --verbose
cargo metadata --locked --no-deps --format-version 1
cargo build --workspace --locked -j 1
cargo test --locked -p nuis --bin nuis -j 1 -- \
  tensor test_dirs workflow_surface project_health project_imports_and_scheduler \
  artifact_runtime_providers build_output_self_check_accepts_built_output_dir \
  build_report_json_exposes_lifecycle_and_domain_unit_summary --test-threads=1
cargo test --locked -p nuisc --test examples_mainline_compile -j 1 -- --test-threads=1
```

The preview command does not clear `target/`; only an explicit `--apply` after
review does that. A warm build must not be reported as a cold rebuild. Verify
temporary project and default build/release outputs disappear on normal exit and
panic unwinding, while another live fixture remains untouched. Generated example
bundles belong in ignored output directories, not source control.

The source checkpoint's cleanup run passed 72 focused CLI/tensor/workflow, six
example and 20 maintenance tests. These 98 tests are not the entire workspace
test suite. Compare future runs by revision and selected filters, not totals alone.

## Native Value And Artifact Checks

On a supported host with LLVM/native linker prerequisites, use the
[native session contract](../reference/nuis-native-scalar-session-bridge-v1.md)
and [value-return contract](../reference/nuis-native-scalar-value-returns-v1.md):

```sh
cargo test --locked -p nuisc --test native_application_bridge -j 1 -- --test-threads=1
cargo test --locked -p nuisc --test control_flow_syntax_native -j 1 -- --test-threads=1
cargo test --locked -p nuis --test native_session_workflow -j 1 -- --test-threads=1
```

These are broader follow-up suites, not part of the 98-test cleanup total.
Confirm actual native execution rather than a missing-prerequisite skip. Preserve
exact nested layouts, raw scalar bits, independent snapshots, overlapping buffers,
malformed-input sentinels, selected traps and zero aggregate allocation counters
for the admitted value path. Ordinary/resource ABI behavior remains separate.

Private capture tests must retain unchanged public/scoped/FFI signatures,
field/argument order, guarded effects, work budgets and conservative rejection.
Aliases and snapshots must not observe later writes or discard selected initializer
work. Build/cache/tamper and source-free restoration must keep exact artifact
identity without interpreter fallback for a selected native failure.

## Provider And Migration Boundaries

For devices, follow the current [image showcase](../../examples/projects/domains/ns_nova_image_showcase/README.md),
[scalar-script profile](../reference/nuis-yir-application-scalar-script-v1.md),
[cancellation](../reference/nuis-yir-application-cancellation-v1.md) and
[provider drain](../reference/nuis-yir-provider-session-drain-v1.md) contracts.
Record provider/hardware identity, actual pixels or hashes, replay and explicit
cleanup. Missing hardware remains unverified; host-scope retirement cannot stand
in for provider/device retirement, and cancellation cannot overwrite prior success.

For compiler migration, use each gate's `validation_command` in
[readiness data](../reference/nuis-self-hosting-readiness.toml). Closed preparation
gates are not completed compiler responsibility transfer. Nuis-rc policy tests and
safe cleanup do not establish shared CAS, compiler leases or a resident collector.

This document follows the existing beta-0.15.0 commit. Updating it does not create
a release/tag, advance the patch, alter runtime behavior or certify unexecuted
hardware, performance, packaging or self-hosting work.
