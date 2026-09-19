# Beta 0.14 Validation Checklist

This is the operational companion to the
[beta-0.14 snapshot](nuis-beta-0.14.0-snapshot.md), recorded against
`1fcfd655b2552f543baa38d4abf9d8a566ecbd3c` (`beta-0.14.1`).
The commands below are validation instructions, not claims that this
documentation-only synchronization reran compiler, native or GPU suites.
Keep historical evidence attached to its original revision.

## Documentation Alignment

- [ ] Check the root README, documentation index, reference index, current
  mainline map and versioning index route to the beta-0.14 snapshot/checklist.
- [ ] Preserve the beta-0.12 snapshot/checklist as the `505c820c` historical
  checkpoint. Do not globally replace version numbers in historical records or
  machine-readable protocol release fields.
- [ ] Keep documentation drift guards aligned with the new current-line wording
  without changing capability scores, production behavior or historical guards.
- [ ] Keep explicit native scalar sessions separate from the default embedded-YIR
  image host, and keep host retirement separate from provider-owned drain.
- [ ] Describe five closed self-hosting preparation gates as bounded preparation,
  not completed compiler replacement. Describe RC planning as read-only policy,
  not an installed shared store or collector.

## Existing CI Scope

The checked-in [Build workflow](../../.github/workflows/build.yml) runs these
source and build checks on Ubuntu, with stable Rust and locked dependencies:

```sh
python3 -m unittest discover -s scripts/tests -v
python3 scripts/check-text-encoding.py
bash scripts/check-doc-links.sh
export CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=2
export CARGO_PROFILE_DEV_DEBUG=0
export CARGO_PROFILE_TEST_DEBUG=0
export RUSTUP_TOOLCHAIN=stable
rustup toolchain install stable --profile minimal
cargo fetch --locked
bash scripts/check-host-absolute-paths.sh
cargo build --workspace --locked
```

The portability script runs one exact test in `examples_mainline_compile`;
this is not a full `cargo test --workspace` gate. The successful baseline
[Build #605](https://github.com/Team-silvortex/nuislang/actions/runs/34745939424)
certifies only its recorded steps on `1fcfd65`. Do not reuse that result as
acceptance evidence for a later commit.

## Focused Checks Outside The Current Build Workflow

Run from the repository root with the toolchain and prerequisites required by
the selected test. Record each command's exit status and any skipped tests;
unavailable prerequisites are not successful execution evidence.

For documentation guards and the dependency-free cache policy:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p nuis --bin nuis dev_tensor -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p nuis-rc -j 1 -- --test-threads=1
git diff --check
```

For selected LLVM/native scalar behavior, use a supported 64-bit Linux or macOS
host with the native compiler/linker prerequisites described by the
[native scalar session bridge](../reference/nuis-native-scalar-session-bridge-v1.md):

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p nuisc --test native_application_bridge -j 1 -- --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -p nuis --test native_session_workflow -j 1 -- --test-threads=1
```

Confirm that the native paths actually execute. Retain accepted and rejected
cases, zero-trip/induction guards, one-sided carry preservation, short-circuit
conditions, checked arithmetic, allocation/drop checks and typed state results.
The frontdoor suite must retain tamper rejection before open, registration/cache
isolation and standalone restoration after the original build directory is gone.
No test should silently substitute interpreter success for selected native failure.

For compiler migration, run the per-gate `validation_command` values in
[self-hosting readiness](../reference/nuis-self-hosting-readiness.toml).
Record candidate/stage0 identity, stage handoff, differential/reproducibility
results and replacement-authority boundaries separately. A gate marked
`stable/100` is scoped to its recorded protocol, not all source programs.

## Provider And Image Acceptance

Use the current [image showcase instructions](../../examples/projects/domains/ns_nova_image_showcase/README.md),
[scalar-script profile](../reference/nuis-yir-application-scalar-script-v1.md),
[cancellation contract](../reference/nuis-yir-application-cancellation-v1.md) and
[provider-owned drain contract](../reference/nuis-yir-provider-session-drain-v1.md)
for the selected backend and hardware. Do not transplant historical beta-0.12
commands without checking the current profile.

Record hardware/provider identity, dispatched pixels or frame hashes, replay,
explicit cleanup and failure-preserving results. Native scalar or portable
protocol tests do not certify Linux GPU execution, Windows transport or physical
device retirement. Missing hardware must be reported as unverified, not a green
fallback. A cancellation must not replace prior successful execution evidence.

## Evidence And Version Decision

For every reported result, record the exact Git SHA, OS/architecture, toolchain,
command, exit status, test/skip counts and artifact or run reference. Separate
checked-in test coverage from newly executed evidence.

This update introduces no release or tag, changes no Cargo package version,
and does not alter the five-gate migration phase, protocol versions, capability
scores or CI workload. The `.0` documentation filenames name the beta-0.14
series; their recorded source checkpoint is `beta-0.14.1`.
