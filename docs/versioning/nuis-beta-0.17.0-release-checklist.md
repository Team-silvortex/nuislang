# Beta 0.17 Validation Checklist

This is the operating checklist for the existing `beta-0.17.*` line.
The [snapshot](nuis-beta-0.17.0-snapshot.md) distinguishes committed identity from
uncommitted follow-ups. Checkboxes are recurring gates, not a claim that every
suite passed at once. Git operations require their own user request.

## Repository And Task Selection

- [ ] Read Git's current release and keep Cargo/protocol versions independent.
- [ ] Keep README, documentation/reference/versioning indexes and the mainline map consistent.
- [ ] Keep the three entry routers below 300 lines; use the proof index for detailed receipts.
- [ ] Inspect live tensor selection, drift, coverage, hierarchy and task-card lineage.
- [ ] Preserve complete predecessor values/hashes when accepting a tensor checkpoint.
- [ ] Keep current regressions, intentional fixtures, locks and minor-version history.
- [ ] Keep source/test/Markdown limits at 800/1000/2000 and ratchet legacy exceptions downward.

## Source-Only Maintenance

These checks do not require pre-existing build output or physical GPUs:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s scripts/tests -v
python3 scripts/check-text-encoding.py
bash scripts/check-doc-links.sh
cargo metadata --locked --no-deps --format-version 1
cargo fmt --all -- --check
git diff --check
```

CI's [Build workflow](../../.github/workflows/build.yml) discovers maintenance
tests, checks UTF-8 and links, fetches locked dependencies, runs the exact host-path
test and builds the workspace. It is not the complete semantic/native/GPU suite.

## Focused Compiler And Tensor Checks

Run Cargo serially and reuse the workspace target directory:

```sh
export CARGO_INCREMENTAL=0
export CARGO_BUILD_JOBS=1
cargo test -p nuisc --test file_line_limit --locked -j1 -- --test-threads=1
cargo test -p nuisc --test examples_mainline_compile --locked -j1 -- --test-threads=1
cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1
cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

For source/native return changes, select the affected families in the
[native session proof index](../reference/nuis-native-session-proof-index.md).
The bounded mixed-exit checkpoint used:

```sh
cargo test -p nuisc --lib --locked -j1 -- iteration_effects control_loops scalar_helpers scalar_control state_loop --test-threads=1
cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_loop_return_parent_mixed:: --test-threads=1
cargo test -p nuis --test native_session_workflow --locked -j1 -- effect_calls::state_loop_return_parent_mixed:: --test-threads=1
```

The later [record-storage certificate](../reference/nuis-native-return-effect-storage-v1.md)
adds its own reduction, source-authority and delivery checks:

```sh
cargo test -p nuisc --lib --locked -j1 -- effect_return_storage --test-threads=1
cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_loop_return_shared:: --test-threads=1 --nocapture
cargo test -p nuis --test native_session_workflow --locked -j1 -- effect_calls::state_loop_return_shared:: --test-threads=1 --nocapture
```

- [ ] Preserve actual source effects, selected traps, original return authority and current State.
- [ ] Check normal/reversed YIR storage, real callee arguments and exact float words where affected.
- [ ] Keep local break/continue and real false/zero returns distinct.
- [ ] Keep full reached-bound reservations without evaluating skipped child bounds.
- [ ] Check failure before result/pending publication; do not infer post-abort native memory from a signal.
- [ ] Require fresh/cache/two source-free restorations and unchanged artifact identity for delivery changes.
- [ ] Require the independent return effect-prefix certificate and its mutation regressions.
- [ ] Do not infer storage-sharing authority from that certificate; coalescing needs its own proof.
- [ ] Validate both original and shared read sets, preserve fallback, and measure actual YIR carry reduction.
- [ ] Keep the [initialization-prefix proof](../reference/nuis-native-return-effect-storage-prefix-v1.md) narrow: literal prints stay ordered, control-flow barriers remain, and initializer failure skips later prints.
- [ ] Require the [candidate-attempt proof](../reference/nuis-native-return-effect-storage-candidates-v1.md): declaration order, at most eight attempts, independent per-candidate certification and exact fallback after exhaustion.
- [ ] Require the [ordered reassignment proof](../reference/nuis-native-return-effect-storage-reassignment-v1.md): fresh definition, exact same-type writes, unchanged AST mutability checks and no later print/result after a failed write.
- [ ] Retain the [helper-call witnesses](../reference/nuis-native-return-effect-storage-helper-calls-v1.md): once-only ordered arguments, completed aggregate handoff, argument/callee failure boundaries and source-free replay without new call authority.
- [ ] Retain the [reached-loop witnesses](../reference/nuis-native-return-effect-storage-loop-calls-v1.md): real iteration/parent-exit updates, semantic rather than numeric oracle events, and later-iteration failure before callback publication.
- [ ] Retain the [branch-selected witnesses](../reference/nuis-native-return-effect-storage-branch-calls-v1.md): distinct multi-field helper paths, no unselected poisoned arguments, mirrored semantic roles and selected even/odd failure before publication.
- [ ] Retain the [record-dependent bound witnesses](../reference/nuis-native-return-effect-storage-bound-calls-v1.md): completed-State-dependent work, once-only reached evaluation, full computed reservations and argument/callee/division failure before child work or publication.
- [ ] Retain the [sibling-bound witnesses](../reference/nuis-native-return-effect-storage-sibling-bounds-v1.md): one completed snapshot per selected parent iteration, separate full reached reservations, child-local versus parent/function exits, and reached/unreachable sibling argument/callee failures.
- [ ] Retain the [parent-preheader witnesses](../reference/nuis-native-return-effect-storage-preheader-bounds-v1.md): selected once-only snapshot construction across parent backedges, distinct full reached reservations, actual State/fault retention and source-free delivery.
- [ ] Retain the [distinct-epoch witnesses](../reference/nuis-native-return-effect-storage-epoch-bounds-v1.md): simultaneous older/newer record identities, changing bounds, reached construction failures, separate full reservations and source-free delivery across backedges.
- [ ] Retain the [returned-snapshot witnesses](../reference/nuis-native-return-effect-storage-snapshot-returns-v1.md): complete older/newer payload identity and raw fields on first/post-backedge inner exits, full entered reservations, skipped work and source-free delivery.
- [ ] Retain the [payload-call witnesses](../reference/nuis-native-return-effect-storage-payload-calls-v1.md): ordered return arguments, selected finalizer failures, complete publication and source-free delivery; keep late-field construction as a separate gate.

## Application And Backend Gates

The [image showcase](../../examples/projects/domains/ns_nova_image_showcase/README.md)
owns its actual Metal pixel/window/replay route.
The [observable CLI regression](../../tools/nuis/tests/std_filesystem_smoke.rs)
owns bounded host I/O programs; image or ML report text is not device evidence.

- [ ] Distinguish embedded-YIR execution from native CPU callbacks and private Nsld images.
- [ ] Keep cancellation admission, host-scope retirement and provider/device retirement separate.
- [ ] Run Linux CUDA/Vulkan on appropriate hardware; absent hardware remains unverified.
- [ ] Keep PE/COFF, wider provider parity and self-contained packaging gaps explicit.
- [ ] Measure performance independently of reduced carry counts or passing smoke tests.

## Storage And Acceptance

Inspect `bash scripts/disk-clean-safe.sh --verbose` before `--apply`, with builds
stopped. Default cleanup preserves root CLI/library/test binaries; broader
`--build-binaries` or `--workspace` removal is an explicit choice.
No option touches other repositories, subprojects, shared Cargo downloads or Docker.
Prefer remote Linux infrastructure for Docker/CUDA-heavy work.

Record exact executed commands and scope. A focused result does not certify
full-workspace semantics, cross-platform/device parity, formal verification,
global memory safety, completed self-hosting or a stable public ABI.
