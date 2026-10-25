# Beta 0.17 Validation Checklist

This is the operating checklist for the existing `beta-0.17.*` line. The
[snapshot](nuis-beta-0.17.0-snapshot.md) separates dated committed identity from
worktree follow-ups. Checkboxes are recurring gates, not a single all-green run.
Git operations require their own user request.

## Select The Scope

- [ ] Read current Git history; keep Cargo and protocol versions independent.
- [ ] Consult the live tensor's selected dependency, evidence, next action and validation command.
- [ ] Identify whether the change affects source admission, native execution, artifact delivery, providers or documentation only.
- [ ] Keep the three entry routers below 300 lines and source/test/reference limits at 800/1000/2000.
- [ ] Preserve intentional regressions, package locks, historical snapshots and complete tensor predecessor lineage.

Detailed proof families belong in the [native session proof index](../reference/nuis-native-session-proof-index.md).
Do not copy its entire chronology into this checklist or treat an old receipt's
remaining boundary as a permanent veto after a later proof advances it.

## Documentation And Source Hygiene

These checks need no physical GPU or pre-existing build artifacts:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s scripts/tests -v
python3 scripts/check-text-encoding.py
bash scripts/check-doc-links.sh
cargo metadata --offline --locked --no-deps --format-version 1
cargo fmt --all -- --check
git diff --check
```

Offline metadata requires locally resolvable metadata; use CI's locked fetch
step when dependency downloads are actually needed. The
[Build workflow](../../.github/workflows/build.yml) separates source checks,
dependency fetch, an exact host-path check and workspace build. Its green result
is not the complete compiler semantic/native/GPU suite.

## Focused Compiler And Tensor Checks

Reuse the workspace target directory and run Rust jobs/tests serially:

```sh
export CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=1
cargo test -p nuisc --test file_line_limit --locked -j1 -- --test-threads=1
bash scripts/check-host-absolute-paths.sh
cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1
cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

Documentation-only work uses these structural gates. It does not advance the
functional tensor coordinate or turn old CPU/device results into a fresh run.
After doc-guard changes, rebuild the ordinary CLI before comparing its status
with source; a test binary is not the installed development frontdoor.

## Native Session Changes

Select the affected families from their receipts and the active task command.
The latest scalar/typed-field-entry print route has focused entrypoints:

```sh
cargo test -p nuisc --lib --locked -j1 -- print_epoch:: iteration_prints:: iteration_effects:: --test-threads=1
cargo test -p nuisc --test native_application_bridge --locked -j1 -- f64_field_epoch_print:: f32_field_epoch_print:: i32_field_epoch_print:: bool_field_epoch_print:: field_epoch_print:: child_epoch_print:: carry_epoch_print:: computed_iteration_print:: typed_i32_scoped_record_carries typed_f32_scoped_record_carries typed_f64_scoped_record_carries literal_print_policy:: literal_print_execution:: --test-threads=1
cargo test -p nuis --test native_session_workflow --locked -j1 -- f64_field_epoch_print --test-threads=1 --nocapture
```

These are focused entrypoints, not a replacement for adjacent checks selected by
the actual edit. The [typed-f64 receipt](../reference/nuis-native-f64-field-epoch-print-v1.md)
owns the exact latest witness and bounds; earlier compiler/storage families
retain their own commands.

- [ ] Preserve source scope, exact types, once-only selected calls and unselected poison suppression, including raw NIR where simplification could hide a defect.
- [ ] Keep loop induction, local break/continue, function return and real false/zero payloads distinct.
- [ ] Reserve complete reached work without evaluating skipped child bounds.
- [ ] Check failure before result/pending/State publication and retain last accepted State.
- [ ] Require independent effect-order certification before considering shared storage; retain fallback for unproved reductions.
- [ ] Keep literal-v1 and explicit i64-v1 print grants separate; a matching signature or cache token is not pure source authority.
- [ ] Keep one-print read lenses separate from real availability, assignments, conditions and reference/resource grants.
- [ ] Check normal/reversed YIR, actual callee entries, flushed native fault prefixes and exact raw float words where relevant.
- [ ] For delivery changes, require fresh/cache and two independent source-free images with unchanged binary/container and host-object identities.
- [ ] Do not rebuild or replace the CLI executable while a delivery test is using it.

Storage, origin, short-circuit and print proofs compose only within their stated
profiles. New field/value kinds require independent source permissions rather
than widening scalar grants by analogy.

## Application And Backend Changes

The [image showcase](../../examples/projects/domains/ns_nova_image_showcase/README.md)
owns real Metal pixel/window/replay evidence. The
[observable CLI regression](../../tools/nuis/tests/std_filesystem_smoke.rs)
owns bounded host I/O programs; image/ML report text alone is not device output.

- [ ] Distinguish embedded-YIR application execution, native CPU callbacks and private Nsld images.
- [ ] Keep cancellation admission, host-scope retirement and provider/device retirement separate.
- [ ] Run CUDA/Vulkan/Metal/CoreML gates on the appropriate hardware and provider profile; absent hardware is unverified.
- [ ] Keep PE/COFF, provider parity, native-object emission and self-contained packaging gaps explicit.
- [ ] Measure performance separately from reduced carry counts, fixture size or passing smoke tests.

## Storage And Acceptance

Inspect `bash scripts/disk-clean-safe.sh --verbose` with writers stopped before
choosing `--apply`. Broader `--build-binaries` and `--workspace` removal are
explicit choices; no option cleans other repositories or shared downloads.
Prefer remote Linux infrastructure for Docker/CUDA-heavy work.

Record exactly which commands ran and what they established. A focused result
does not certify whole-workspace semantics, device parity, formal verification,
global memory safety, completed self-hosting or stable public ABI.
