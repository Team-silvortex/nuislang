# Beta 0.12 Validation Checklist

This is the operational companion to the
[beta-0.12 snapshot](nuis-beta-0.12.0-snapshot.md), recorded against `505c820c`
(`beta-0.12.2`). It is not a new release request or a whole-workspace certification.
Keep Git as the release authority; do not bump Cargo versions or create a tag
merely to synchronize documentation.

## Documentation Gate

- README, documentation index, mainline map and local ns-nova/example guides
  describe the same current line and route to the current contracts.
- Historical beta/alpha/pre-alpha anchors retain their original checkpoint
  meaning. Preparation-gate closure is not general self-hosting completion.
- Cancellation admission, host retirement and provider/device retirement remain
  distinct. Window API availability must not imply an AppKit trigger or a new
  Nuis intrinsic/CFFI allowlist grant.
- Plain CLI examples distinguish bounded input/report proofs from large-file,
  full-Unicode, device-execution and distribution claims.
- Links remain repository-relative. No private host address, personal directory,
  fixed macOS release or temporary artifact path becomes a prerequisite.
- Tensor evidence and its documentation drift guards agree; documentation-only
  edits do not raise implementation scores or close hardware-dependent gates.

## Focused Checks

Run from the repository root, serially. These are reproducible commands, not a
claim that every documentation edit reruns all device and compiler tests.

```sh
export CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=1
cargo fmt --all -- --check
git diff --check
python3 scripts/check-text-encoding.py
cargo test -p nuisc --test file_line_limit -j 1 -- --test-threads=1
cargo test -p nuis --bin nuis dev_tensor -j 1 -- --test-threads=1
cargo run -p nuis -- dev-tensor --json
```

Confirm clean coverage, recursive hierarchy, mainline selection and drift.
Use the current CLI output for check counts rather than pinning a rolling total
in a README. The selected prerequisite remains the persistent application session
until the dependency plan and acceptance evidence explicitly move it.

For session/cancellation implementation changes:

```sh
cargo test -p yir-runtime-host --lib --test application_session --test provider_application_session -j 1 -- --test-threads=1
cargo test -p nuisc --test ns_nova_application_session --test ns_nova_application_outcome --test ns_nova_application_cancellation -j 1 -- --test-threads=1
cargo clippy -p yir-runtime-host -p nuisc -p nuis -p yir-pack-aot --all-targets -j 1 -- -D warnings
```

Verify cancellation while busy, Finish winning admission, first-fault retention,
invalid output nonconsumption, once-only receipts, ticket/window lifetime
independence and no implicit close or manufactured parent outcome. Compiled Nuis
cancellation tests use protocol peers; they do not prove physical device drain.

For the real Metal image/window route, on a supported Apple host:

```sh
NUIS_TEST_QUIET_SUCCESS_LOGS=1 cargo test -p nuis --bin nuis artifact_device_sample_shader_render -j 1 -- --test-threads=1
```

This suite includes actual image pixels, persistent state, AppKit input,
compiled export, parent delivery, provider fault injection and replay exhaustion.
Do not count an unavailable or skipped backend as a pass. Linux CUDA/Vulkan and
other registered providers require separate suitable-host validation; prefer
remote Linux for Docker/CUDA work rather than starting local Docker Desktop.

For the practical CLI baseline:

```sh
cargo test -p nuis --test std_filesystem_smoke std_tooling_observable_cli_smoke_checks_reports_and_stdin -j 1 -- --exact --test-threads=1
```

## Acceptance Boundary

Before widening the current slice, require an explicit host cancellation path,
provider-owned resource retirement evidence before reuse, and preservation of
all existing close/failure semantics. Do not weaken GLM, clocks, budgets, identity
checks or ownership to make an example pass. Native CPU lowering, self-contained
Nsld images, multi-child orchestration and engine ML/audio integration are
separate gates, not automatic consequences of these checks.
