# Beta 0.15 Validation Checklist

This is the operational companion to the
[beta-0.15 snapshot](nuis-beta-0.15.0-snapshot.md), anchored to
`05951befc70d6a145e4978a3ff8909737e5c4fbb` (`beta-0.15.0`, 2026-09-24).
The [beta-0.15.6 patch](nuis-beta-0.15.6-patch.md) records subsequent read-only
scoped inputs, child exits, parent-entry snapshots and selected validation results.
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

Private capture tests must retain unchanged public/FFI signatures, field/argument
order, guarded effects, work budgets and conservative rejection. Generated scoped
helpers may project invariant inputs and proven partial or entirely unread flat-record carry inputs;
induction and break-control identities must survive unchanged. Aliases and snapshots must not observe later writes or
discard selected initializer work. Returning-child versions must not leak into
parent continuations. Fallthrough and loop-written input reconstruction require an exact-type,
field-only copy-family proof and must retain original record assignments and
complete initializers, per-trip snapshots and backedges. Whole escapes remain
conservative; zero trips must not execute body work.
Explicit scoped field seed maps require complete initial state and unique, same-typed
argument mappings, including partial maps with independent seeds. Prove each initial field-seed record's
exact nominal identity before flattening; reject mismatched or unknown origins.
Preserve bool conversion and named break zero seeds; never treat mutable-field
operands as invariant input snapshots.
Check legacy and explicit-seed parsing, metadata-free dependency/GLM reads, unused
seed validation, zero-trip initializer failures and independent state/parameter bounds.
Generated partial-record projection must prove the original seed/reconstruction map
at every scoped caller, version only eligible record parameters, keep full output
state and prove the exact nominal type/width/result-slot range when no record field
is passed. All scoped callers must agree; vetoed rewrites must publish no elision proof.

Generated bool/i32/i64/f32/f64 record carries must retain explicit word encodes/decodes, exact
nominal seed origins and per-field mapping proofs. Cover full-width records, nested
loops, multiple flags, snapshots, breaks, zero trips and ordinary native execution.
Keep arithmetic/work-limit failures atomic and source-free CLI restoration exact;
Cover signed i32 limits, wrapping arithmetic, independent scalar slots and wrong-kind
cast rejection. F32/F64 require raw negative-zero/NaN/subnormal bit parity, exact kinds,
numeric-cast rejection and source-free restoration; nested records and resources need separate scoped-carry admission.
F64 must preserve every bit, including the sign bit, and coexist with the other four scalar kinds.

The shared pure-value codec also has an opt-in host-clang argument/return probe:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p yir-lower-llvm --lib -- typed_record_arguments_round_trip_through_host_llvm --ignored --test-threads=1
```

Verify flat/nested 1..64-leaf records and scalar bit patterns at both `-O0` and
`-O2`. This isolated probe exercises the codec; generated non-scoped source capture
admission additionally requires the following source/native and restoration gates:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- dead_record_snapshots return_invariant return_signal return_storage continuation_reads index_recovery scalar_branch_inputs materialized_word_loop_returns capture_params --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --test native_application_bridge -- typed_record_inputs typed_record_guards typed_scoped_record_inputs typed_mixed_scoped_record_carries typed_i32_scoped_record_carries typed_f32_scoped_record_carries typed_f64_scoped_record_carries typed_nested_ typed_sparse_record_carries typed_sparse_branch_snapshots typed_materialized_join_snapshots typed_materialized_loop_snapshots typed_sparse_nested_ wide_scoped_record typed_local_declared --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuis --test native_session_workflow -- native_record_inputs_build_cache_and_restore_without_sources native_scoped_record_inputs_cache_and_restore_the_full_mapping_without_sources native_branch_record_inputs_cache_and_restore_guarded_branches_without_sources native_mixed_record_carries_cache_and_restore_typed_maps_without_sources native_i32_record_carries_cache_and_restore_signed_maps_without_sources native_f32_record_carries_cache_and_restore_bit_maps_without_sources native_f64_record_carries_cache_and_restore_full_width_maps_without_sources native_nested_record_carries_cache_and_restore_exact_paths_without_sources native_sparse_typed_record_inputs_restore_complete_seeds_without_sources native_sparse_branch_snapshots_restore_current_versions_without_sources native_materialized_join_snapshots_restore_current_versions_without_sources native_materialized_loop_snapshots_restore_nested_versions_without_sources native_materialized_nested_returns_restore_complete_state_without_sources native_full_width_nested_returns_restore_complete_state_without_sources --test-threads=1
```

The generated 64-field-plus-predicate case must run, while a source-declared 65-leaf
namesake still rejects. Keep bounded nominal layout, exact kind and unique parameter
binding checks, selected traps and failure sentinels. Scoped flat-i64 record inputs
require complete seed proofs and exact per-trip maps. Wide generated branch inputs
must retain one-time predicates and skip unselected calls/constructor arithmetic.
Nested pure-scalar carries require complete field-segment seed/backedge maps and exact child nominal types.
Sparse mixed/nested inputs retain complete seeds, exact typed leaf maps and child-local/identity-version proofs.
Materialized outer joins retain real assignments, checked operands and fallthrough demand. Changing nested-loop snapshots
require converged header demand, zero-trip state and nearest-loop break/continue targets. Nested returns must keep 9/30/31/57/58/59/60/61/62/63/64-word
state, exact shared budgets, canonical rejection and failure-atomic overlapping output. Width 64 fits outer 63/inner 60; a changed outer tag now fits outer 64/inner 60 through the preserved count snapshot.
Changing both tag and count must still reject 65 outer private carries. Exact initialized record reuse must retain original source admission and transactional revalidation.
Preheader snapshot identities require ordered-pair branch joins, per-field parent-entry summaries, separately versioned opaque evaluations and fixed-point checks for delayed aliases on later trips. Join only relations valid on every arm; keep unknown leaves distinct, branch locals scoped, type/width/work rejection atomic and old versions unchanged. Parent summaries must retain only leaves stable at every intermediate publication across zero and arbitrarily many trips; independently freshen varying leaves without equating unknowns. Keep the original 65536-work/64-depth limits and inner-only/original-body fallback. The 64-word changed-tag fixture must run through a preheader `if` with 60/64 carries and 5/11 private arguments; one opaque count arm must reject at 65 outer words. Retain selected preheader checks and the existing restriction on scalar rebinding outside loops.
Run `cargo test --locked -j1 -p nuis --test native_session_workflow -- native_parent_entries native_materialized_nested_returns native_checked_child_snapshots --test-threads=1` with the same bounded build environment. Parent-entry restoration must keep 7/12 private arities, zero and later parent trips, repeated events, cache hits, pre-open tamper rejection and byte-identical LLVM/state after source/manifest/build deletion. Changed parent tags must not retain first-trip equality. Private argument counts are separate from carried-word widths, not a performance result.
Nested return-owned loops may promote only leaves preserved at every write, using bounded fixed points that include zero trips and intermediate child exits. Keep original RHS evaluation, private compact record seed maps, nominal reconstruction and proof fallback; scoped calls must not consume ordinary-call boolean packing. Ordinary child-exit rewrites preserve assignment own-read order, registered break identities and checked compact parameter seed types; failed admission still retains an admitted inner-only plan or the original body.
The ordinary-child fixture must execute 64-word state with zero/one/two trips and break/continue.
Run `cargo test --locked -j1 -p nuis --test native_session_workflow -- child_exit partial_child_snapshot opaque_child_snapshot opaque_continue_snapshot checked_child_snapshot --test-threads=1` with the same bounded build environment for the source-free child-exit workflows.
Observed child indices must remain correct at 9/63/64 words; break keeps two child control words and continue no child record backedge, without dropping the visible index.
Ordinary child invariants do not grant return-signal authority. Partial reads of exact total reconstructions from unwritten inputs may project typed fields before unused copies disappear; calls, arithmetic, codecs and resources stay evaluated. Keep bounded lexical/caller proof, the 108-case child-mutation matrix and conservative proof fallback. Full-width partial-snapshot workflows must retain observed indices, checked field uses and exact 4/2 child arguments without widening ABI.
Selected child traps must leave aliased/disjoint output and border sentinels untouched after five loop reservations and eleven helper entries.
Opaque snapshot calls in multi-carry breaks and no-carry/single-carry continues retain their additional helper entry, selected traps and zero-trip skipping. Read-only mixed-record transport requires every caller's invariant-root proof; it grants no seed/elision authority. Keep exact kinds, 5/5/11 observed break iteration arities and source-free restoration. Checked constructors now fit through cached nominal layouts/ranges, not higher budgets or deleted arithmetic; retain fresh value origins, qualifier rejection, field-order traps and 5/5/11 break / 3/5/11 continue restoration. Changing outer fields and opaque carry mutation must still reject over-limit state. Continue probes must preserve observed exit indices, first/second-trip timing, exact budgets and failure-atomic aliasing without adding record carry mappings.
Dead leading-index recovery requires bounded suffix/generated-output proof; unknown syntax or exhausted work keeps recovery.
Direct/outer suffixes, branch-helper outputs and parent backedges must retain observed exit indices. Finer carry liveness, task thunks and resources remain separate.
Only compiler-minted return signals with bounded return-owned exit proof may share a canonical break slot. Keep same-loop ordinary breaks independent, child exits local, mixed continues separate and payload evaluation before publication. Retain 300 differential source cases, 36 native cases and both flattened/whole-record boundary transport.
Callback/public/FFI ABI is unchanged.
Source names cannot grant authority and scalar/bool/break seeds stay required. Reject ambiguous or computed maps,
unresolved fallthrough joins and exhausted nested-loop demand proofs without partially mutating the
candidate. Check 3/7/64-slot state with two/three arguments, multiple records, break/continue/bool identities,
ordinary native results and initializer/second-trip traps, plus native lifecycle
arity inspection and byte-identical source-free restoration.
Build/cache/tamper and source-free restoration must keep exact
artifact identity without interpreter fallback for a selected native failure.

Sparse mixed/nested compiler proof gates:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- record_words::tests sparse_nested_word_seeds materialized_record_demand --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuisc --lib -- registered_control_identities compact_record_seed_origins return_invariant --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test --locked -j1 -p nuis --test native_session_workflow -- native_nested_child_exits_restore_full_width_state_without_sources --test-threads=1
```

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

This checklist follows the beta-0.15 minor line; the linked patch notes distinguish
later evidence from the original baseline. Updating documentation alone does not
create a release/tag, alter runtime behavior or certify unexecuted
hardware, performance, packaging or self-hosting work.
