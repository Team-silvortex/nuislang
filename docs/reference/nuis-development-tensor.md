# Nuis Development Tensor

This file defines the first lightweight development-progress model for the
alpha line.

It answers one narrow question:

`how do we describe current system progress without flattening everything into
one vague roadmap list?`

## Model

The development tensor is a 3-axis progress model:

* `architecture`
  the broad system layer or design lane
* `module`
  the concrete repository/tool/package area carrying the work
* `function`
  the user-visible or toolchain-visible capability being matured

Each tensor cell carries:

* `status`
  protocol-owned maturity label. In `dev-tensor-status-v1`, valid values are
  `stable`, `usable`, `active`, and `early`
* `progress`
  current alpha-era progress score from `0` to `100`
* `bootstrap_critical`
  whether Nuis should treat this cell as important before self-hosting
* `closure_role`
  the role this cell plays in the compiler/toolchain/runtime closure
* `evidence`
  the current proof anchor, usually tests, frontdoor fields, docs, or examples
* `next_step`
  the most useful next action for that cell
* `blocker`
  the current concrete blocker that makes this cell weaker than done
* `next_action`
  the action-oriented task-card step; this can mirror `next_step` while tools
  migrate from narrative guidance to machine-consumable planning
* `validation_command`
  the narrow command that should prove the next action worked
* `expected_artifact`
  the concrete artifact or surfaced contract expected after the next action

Short rule:

`architecture tells where the work lives; module tells who owns it; function
tells what capability is being matured`

## CLI

Use:

```bash
cargo run -p nuis -- dev-tensor
cargo run -p nuis -- dev-tensor --json
```

The JSON surface is intentionally simple:

* `kind = "nuis_dev_tensor"`
* `model = "architecture-module-function-progress-tensor"`
* `axis_0 = "architecture"`
* `axis_1 = "module"`
* `axis_2 = "function"`
* `status_protocol_version`
* `status_protocol = [...]`
* `hierarchy_protocol_version`
* `hierarchy_validation_status`
* `hierarchy_validation_node_count`
* `hierarchy_validation_max_depth`
* `hierarchy_validation_error_count`
* `hierarchy_validation_first_error`
* `hierarchy_validation_errors`
* `hierarchy_root_status`
* `hierarchy_root_progress`
* `hierarchy_root_weakest_child_path`
* `bootstrap_critical_count`
* `bootstrap_critical_average_progress`
* `weakest_bootstrap_architecture`
* `weakest_bootstrap_module`
* `weakest_bootstrap_function`
* `weakest_bootstrap_status`
* `weakest_bootstrap_progress`
* `weakest_bootstrap_closure_role`
* `weakest_bootstrap_evidence`
* `weakest_bootstrap_next_step`
* `weakest_bootstrap_blocker`
* `weakest_bootstrap_next_action`
* `weakest_bootstrap_validation_command`
* `weakest_bootstrap_expected_artifact`
* `weakest_bootstrap_task_card_protocol`
* `weakest_bootstrap_task_card_source`
* `weakest_bootstrap_task_card_status`
* `weakest_bootstrap_task_card_ready`
* `weakest_bootstrap_task_card_coordinate`
* `weakest_bootstrap_task_card_priority_reason`
* `weakest_bootstrap_task_card_action`
* `weakest_bootstrap_task_card_command`
* `weakest_bootstrap_task_card_expected_artifact`
* `weakest_bootstrap_task_card_handoff_mode`
* `weakest_bootstrap_task_card_handoff_coordinate`
* `weakest_bootstrap_task_card_handoff_reason`
* `weakest_bootstrap_task_card_handoff_action`
* `weakest_bootstrap_task_card_handoff_command`
* `weakest_bootstrap_task_card_handoff_expected_artifact`
* `coverage_status`
* `coverage_expected_source`
* `coverage_expected_fallback_used`
* `coverage_expected_source_error`
* `coverage_expected_count`
* `coverage_covered_count`
* `coverage_missing_count`
* `coverage_orphaned_count`
* `coverage_stale_count`
* `coverage_first_gap`
* `coverage_missing_coordinates`
* `coverage_orphaned_coordinates`
* `coverage_stale_coordinates`
* `manifest_coverage_status`
* `manifest_coverage_source`
* `manifest_backed_coordinates`
* `manifest_missing_modules`
* `manifest_untracked_modules`
* `milestone_coverage_status`
* `milestone_coverage_source`
* `milestone_schema`
* `milestone_coordinates`
* `milestone_missing_coordinates`
* `milestone_untracked_coordinates`
* `milestone_constant_drift_count`
* `milestone_constant_drift_coordinates`
* `drift_status`
* `drift_check_count`
* `drift_check_passed_count`
* `drift_check_failed_count`
* `drift_first_failed_check`
* `drift_checks = [...]`
* `hierarchy = {...}`
* `cells = [...]`

Each cell includes both named coordinates and a `coordinates` array so scripts
can read it either as records or as tensor coordinates.

## Status Protocol

The tensor status field is now protocolized rather than free-form text. The
current protocol is `dev-tensor-status-v1`:

* `stable`
  rank `4`, phase `validated`, terminal for the current milestone slice
* `usable`
  rank `3`, phase `usable`, strong enough to consume but still evolving
* `active`
  rank `2`, phase `in-progress`, actively maturing and allowed to move fast
* `early`
  rank `1`, phase `exploratory`, not mature enough to anchor bootstrap-critical
  closure by itself

Coverage treats an unknown status as stale metadata. This keeps the tensor from
quietly drifting into ad-hoc labels.

## Recursive Hierarchy

The flat `architecture/module/function` cells are also projected into a
recursive hierarchy:

`root -> architecture -> module -> function`

The recursive representation is governed by
`nuis-dev-tensor-hierarchy-v1`. Its validator walks the full tree and checks
legal level transitions, parent-derived paths, unique nodes, progress bounds,
branch status/progress/count aggregates, weakest-child selection, and the
two-way mapping between function leaves and registered tensor cells. A clean
tree reports `hierarchy_validation_status = "clean"`; malformed trees report
the first error and the complete deterministic error list.

Each hierarchy node carries:

* `level`
* `path`
* `status`
* `status_rank`
* `progress`
* `cell_count`
* `bootstrap_critical_count`
* `weakest_child_path`
* `children`

Branch status is derived from the weakest child status, and branch progress is
the weighted average of descendant function cells. This means the tensor can be
read both as a table and as a recursively inspectable project tree. The
recursive form is intended to support future bootstrap planning where a weak
architecture lane can be expanded into its weakest module and then into the
exact function cell that needs work.

The summary also mirrors the weakest bootstrap-critical function cell as a
small navigation bundle: status, progress, closure role, evidence, and next
step. This is the preferred first read when choosing the next mainline task.
The same weakest cell is also projected into a small task-card surface:
protocol, source, status, ready flag, coordinate, priority reason, action,
validation command, and expected artifact. That gives scripts and future
self-hosted tooling one stable bundle to consume without reassembling many
`weakest_bootstrap_*` fields by hand.

The task-card protocol is `nuis-dev-tensor-task-card-v1`. A ready task card
means the tensor found a weakest bootstrap-critical coordinate and coordinate
coverage is currently clean.

Task-card selection uses the deterministic ordering
`status_rank -> progress -> coordinate`, reported as source
`weakest-bootstrap-status-progress-path`. Lower status maturity is weaker;
progress breaks status ties, and the full coordinate makes selection stable
when input registration order changes.

The task-card also exposes a handoff bundle. When the weakest coordinate is the
tensor itself, `weakest_bootstrap_task_card_handoff_mode` becomes
`self-maintenance-handoff` and the handoff coordinate names the next weakest
bootstrap-critical non-tensor cell to continue after refreshing the model.
Otherwise the handoff mode is `direct` and mirrors the current task-card
coordinate. The same status/progress/path ordering chooses the non-tensor
handoff, so a completed stable frontdoor does not hide a merely usable runtime
lane just because it appears earlier in the source snapshot.

`nuis status` also prints the short tensor summary plus hierarchy protocol and
validation state. That makes the model part of the toolchain self-orientation
surface, not just a separate report command, and prevents task handoff from
trusting an invalid recursive projection.

## Coverage Manifest

The tensor now has a milestone-owned expected-coordinate source. The primary
source is:

`docs/reference/nuis-development-tensor.milestones.toml`

That manifest lists the coordinates that the alpha line expects to see in the
tensor:

`expected architecture/module/function coordinates`

The coverage layer derives expected coordinates from that manifest, falls back
to the Rust `DEV_TENSOR_EXPECTED_COORDINATES` emergency snapshot only if the
manifest cannot be read, compares the expected coordinate set with the actual
`DEV_TENSOR_CELLS` entries, and reports:

* `coverage_status`
  `clean` when required expected coordinates are covered and no stale/orphaned
  cells are present; otherwise `gap`
* `coverage_expected_source`
  the active source for expected coordinates, normally
  `docs/reference/nuis-development-tensor.milestones.toml`
* `coverage_expected_fallback_used`
  true only when the Rust fallback snapshot was used because the manifest could
  not be loaded
* `coverage_expected_source_error`
  the manifest load error when fallback was needed, otherwise `<none>`
* `coverage_missing_coordinates`
  expected coordinates that do not currently have a tensor cell
* `coverage_orphaned_coordinates`
  tensor cells that exist but are not declared by the coverage manifest
* `coverage_stale_coordinates`
  cells with invalid metadata, such as empty evidence or out-of-range progress
* `coverage_first_gap`
  the first missing, orphaned, or stale coordinate for quick CLI triage

Short rule:

`drift checks ask whether evidence anchors still exist; coverage asks whether
the tensor itself still spans the expected project map`

This is not yet automatic repository discovery. It is the first guardrail that
prevents the tensor from becoming only a hand-written status list. Future
versions can derive additional coordinates from galaxy manifests, Nustar
registries, and std module manifests, while the milestone file remains the
human-owned alpha planning map.

## Manifest-Backed Coordinate Coverage

The tensor now has a first manifest-backed coordinate view. It reads the stdlib
galaxy layout from `stdlib/index.toml`, compares those module names with the
current `standard-library/*/*` tensor cells, and reports:

* `manifest_coverage_status`
* `manifest_coverage_source`
* `manifest_backed_coordinates`
* `manifest_missing_modules`
* `manifest_untracked_modules`

This is intentionally advisory for alpha. A manifest module such as `core` or
`ns-nova` can be reported as untracked without failing coverage, because not
every official galaxy is ready to become a tensor cell at the same time.

The useful invariant is narrower:

`if a standard-library tensor cell claims progress for std, PixelMagic, or
WitSage, the dev tensor can now verify that the matching official stdlib module
manifest still exists`

## Milestone-Owned Coordinate Coverage

The tensor now also has a milestone-owned expected-coordinate manifest:

`docs/reference/nuis-development-tensor.milestones.toml`

This file groups expected tensor coordinates by alpha milestone, marks whether
the milestone is bootstrap-required or optional, and gives the tensor a
project-owned source of truth outside the Rust constant table.

The current Rust `DEV_TENSOR_EXPECTED_COORDINATES` table still exists as a
checked snapshot and emergency fallback. The important change is that the
tensor now derives the primary expected-coordinate set from the milestone
manifest and compares all three sides:

* milestone manifest coordinates
* current `DEV_TENSOR_CELLS`
* Rust expected-coordinate snapshot

The milestone coverage reports:

* `milestone_coverage_status`
  `clean` when the milestone manifest covers all cells, all manifest
  coordinates have cells, and the Rust snapshot has no drift
* `milestone_coordinates`
  derived records in `milestone:requiredness:architecture/module/function`
  form
* `milestone_missing_coordinates`
  milestone coordinates that do not have tensor cells
* `milestone_untracked_coordinates`
  tensor cells that are not owned by any milestone manifest entry
* `milestone_constant_drift_count`
  parity failures between the manifest-derived coordinates and the Rust
  expected-coordinate snapshot
* `milestone_derived_cache_protocol`
  the protocol name for the generated coordinate snapshot metadata
* `milestone_derived_cache_status`
  `cacheable` when the manifest-derived coordinate set has a reproducible
  cache key; this does not imply that a cache file was written
* `milestone_derived_cache_key`
  a stable hash over normalized `milestone:requiredness:coordinate` records
* `milestone_derived_cache_coordinate_count`
  the number of coordinates covered by that generated snapshot key

Short rule:

`milestone coverage makes the tensor less hand-written: milestones own the map,
Rust constants must prove they still mirror it`

The milestone-derived cache metadata is intentionally zero-write for now. It
gives future tooling a deterministic key for generated coordinate snapshots
without creating hidden disk usage. The Rust `DEV_TENSOR_EXPECTED_COORDINATES`
table remains an emergency fallback mirror, not the preferred editing surface.

## Drift Checks

The tensor now includes a first lightweight drift-check layer.

These checks do not replace the real test suite. They only verify that selected
progress evidence anchors still exist in the repository, such as:

* frontdoor JSON fields
* workflow/artifact runtime regression assertions
* reference-document field anchors
* standard-library smoke-test and example-lane anchors
* registered Nustar domain contract anchors, including dispatch readiness and
  bridge materialization fields

The current status values are:

* `clean`
  every configured evidence anchor is still visible
* `drift`
  at least one configured evidence anchor is missing

Short rule:

`drift checks make the tensor less imaginary: if a progress cell claims a
frontdoor or document exists, the tensor can at least notice when that anchor
disappears`

The first std-oriented checks deliberately anchor the bootstrap-critical
`host-io-filesystem-text` cell to:

* `tools/nuis/tests/std_filesystem_smoke.rs`
* `tools/nuis/tests/official_galaxy_hetero_smoke.rs`
* `examples/projects/tooling/README.md`
* `stdlib/std/README.md`

That keeps the standard-library progress cell tied to the project-form
filesystem, IO, text, terminal, and tooling smoke chain instead of only a broad
roadmap phrase. The current std evidence also includes the observable CLI smoke
`std_tooling_observable_cli_smoke_checks_reports_and_stdin`, which checks
`run-artifact --json` prelaunch readiness, stdout/stderr report output from the
host IO report lane, direct stdin consumption by the built binary, and
`host_stdin_read` / `host_stdout_write` / `host_stderr_write` lowering anchors.
The PixelMagic side of that lane now also keeps `std-preprocessed-pgm` input
evidence visible through provider-sample materialization and
`execute-provider-samples` comparison metadata, including the input evidence
hash used by later shader output comparison. That evidence now binds a raw
`gray8` payload path, dimensions, stride, maximum value, operation, byte count,
and content hash. These values now lower into package-independent
`nuis-provider-buffer-descriptor-v1` and
`nuis-provider-kernel-descriptor-v1` requests; Nsdb converts legacy evidence
into the same model but native adapters consume only the registered request.
The official heterogeneous smoke verifies persistence of the four source
pixels and, on supported macOS hosts, submits them through the registered Metal
runner for real buffer upload, invert dispatch, readback, and output hashing.
Unsupported hosts keep the deterministic provider fallback.

The WitSage side now uses the same registered provider request model for a
contiguous four-element `f32` tensor and `witsage.vector.affine` kernel. On
macOS, Nuis persists a deterministic `.mlmodel` asset and binds its path,
length, hash, input feature, and output feature through
`nuis-provider-model-asset-descriptor-v1`. Nsdb validates that descriptor,
compiles and loads the model through CoreML, requests `CPUAndNeuralEngine`
compute units, executes `predictionFromFeatures`, and verifies the affine
result `[3, 5, 7, 9]` through stable output bytes and hash evidence. This is a
real `MLModel` prediction closure. It does not prove that CoreML scheduled the
operation on ANE. The adapter now loads `MLComputePlan` and emits
`nuis-coreml-compute-plan-evidence-v1`, including layer count plus preferred
and supported compute-device sets. On the M2 smoke host the affine model has
four CoreML plan layers, supports CPU, GPU, and Neural Engine, but prefers CPU.
That result is preserved rather than upgraded into a false ANE-execution
claim: CoreML's public plan API describes anticipated device usage, and this
small graph is not an effective Neural Engine workload.

The second registered model is a deterministic `16x64x64` feature-grid
projection. Nuis persists its 256 KiB all-ones input and hash-bound model asset;
Nsdb consumes the same generic buffer/kernel/model descriptors, without
matching a WitSage operation name. The prediction returns 65,536 `f32` ones,
while its one-layer compute plan supports CPU, GPU, and Neural Engine and
prefers Neural Engine on the M2 smoke host. The affine and feature-grid tests
therefore provide honest CPU-preferred and ANE-preferred baselines.

Both operations now coexist in one `nuis-provider-request-collection-v1`
record. Collection order is explicit (`feature-grid` then `affine`), every
request retains independent buffer/kernel/model validation, and Nsdb executes
all entries fail-closed. `nuis-provider-output-collection-v1` mirrors indexed
request identities, byte counts, hashes, execution/compute-plan evidence, and
an order-sensitive collection hash. Each model request also binds a
`nuis-provider-output-comparison-descriptor-v1` to its output buffer, `f32`
shape, hash-bound expected asset, absolute/relative tolerance, and non-finite
policy. Nsdb reads the expected asset independently and compares every returned
element before emitting `comparison-passed`; shape/byte-count mismatches,
tampered expected assets, invalid policies, NaN/Inf under `reject`, and values
outside tolerance all fail closed. The official M2 lane compares 65,536 dense
elements and four affine elements with zero mismatches.

The next collection boundary is dependency structure. Requests are ordered but
do not yet declare producer/consumer data-flow edges, so independent entries
are closed while graph-shaped provider work remains explicit future work.

The language-core checks anchor the bootstrap-critical
`language-core/nuisc/type-control-flow-generics` cell to:

* `tools/nuis/tests/language_bootstrap_smoke.rs`
* `examples/projects/task/task_result_enum_demo`
* `examples/projects/state/generic_method_bound_guarded_nested_match_demo`
* `examples/projects/state/glm_buffer_roundtrip_state_demo`
* `examples/projects/state/std_style_language_bootstrap_demo`
* `examples/projects/state/std_style_language_import_bootstrap_demo`

That smoke is intentionally higher-level than an isolated parser or frontend
unit test. It builds the project through the `nuis` CLI, checks the
`run-artifact --json` prelaunch contract, verifies NIR/YIR/LLVM anchors for
generic `Result<T, E>`, higher-order specialization, enum variant lowering,
task-result control flow, and host-FFI signature whitelist evidence, then runs
the produced binary and asserts its deterministic Result/task/error exit code.
It also builds and directly executes the generic trait-bound guarded nested
match project and the GLM buffer roundtrip project. Those checks anchor
monomorphized trait method calls (`impl.Addable.for.i64.add`), alias-expanded
generic functions (`bump__i64`), buffer length/load/store/free lowering, and
YIR lifetime/effect edges around `cpu.store_at` / `cpu.free`. The same smoke
now also builds and executes the chained try/await Result HOF project, which
keeps `?` continuations alive across `normalize`, `decorate`, and `pipeline`
helper boundaries, feeds dynamic `host_argv_count()` input into helper-side
Result `?` continuations, asserts the produced native binary exit code, and
checks the LLVM output contains no deferred lowering. Sequence-level early
return folding now lifts `?` continuations to whole-Result selects instead of
selecting between an Err struct and an Ok payload. The std-style language
bootstrap workload now combines that dynamic Result path with Buffer
load/store/free lowering, higher-order lambda specialization, trait-bound
method calls, pointer borrow/free control flow, and async helper boundaries in
one native executable, with a deterministic exit code and no LLVM deferred
lowering. The import-boundary version now splits public helper enum/struct/type,
generic `Result`/HOF helpers, and Buffer/pointer helpers into
`StdStyleLanguageSupport`, consumes them through `use cpu
StdStyleLanguageSupport`, verifies the project module/import reports, keeps
entry-local trait-bound generic execution alive, and still produces the
deterministic native exit code. Helper modules now participate in lambda/HOF
expansion, helper public generic functions are visible as imported templates,
and helper-private `__hof_` / `__lambda_` synthetic functions are retained for
internal lowering. Helper-module impl method emission now also keeps
support-side trait-bound calls such as `bump<T: Addable>` executable through
the imported helper workload. The same workload now leaves imported
`result_map(...)` calls and Result helper constructors unannotated, proving that
cross-helper expected-type inference can carry generic arguments through the
std-shaped HOF boundary. A second package-shaped workload now splits the same
surface across `StdPkgCore`, `StdPkgOps`, and `Main`, including helper-to-helper
imports, imported aliases, Result/HOF inference, trait-bound methods,
Buffer/pointer control flow, and a deterministic native exit code. That path is
backed by partial expected-type propagation, so a helper HOF argument can retain
known generic slots such as `Result<T, Error>` while payload constructors infer
the remaining `T`. That surface has now started moving into the real std
galaxy as auto-injected `lib/language_core.ns` and `lib/language_ops.ns`; the
`std_language_galaxy_bootstrap_demo` consumes them via `std=workspace`, verifies
std galaxy module/import reports, and runs the same helper-to-helper
Result/HOF/trait/memory path as a native binary. The
`std_language_cli_report_demo` now extends that surface into a CLI-shaped std
consumer by combining language contracts with `StdTextContracts` and
`StdIoContracts`, writing a real stdout report, and validating the text/IO
gates through native execution. `std_language_report_file_demo` then pushes the
same language surface through `StdReportContracts`, writes an argv-selected
report file plus stdout, and validates the reusable report-file gates. The next
step, `std_language_workflow_demo`, feeds `StdLanguageOps.build_report` into a
two-step host command workflow through `StdCliContracts`, proving the same
Result/HOF/trait/memory surface can participate in command gates rather than
only report output. `std_language_build_pipeline_demo` extends that route into
a four-stage prepare/check/compile/package gate through
`StdCliContracts.build_pipeline_total` with no LLVM deferred-lowering notes.
`std_language_task_cli_demo` then carries the same surface into a task-backed
CLI path through `StdTaskContracts` and real stdout output. Integer scalar task
payloads now cross the native scheduler ABI as pending handles. Arbitrary-arity
`bool`/`i32`/`i64` async bodies are emitted as deferred helper thunks, then
normalized through LLVM-generated `i64(ptr context)` wrappers and one runtime
spawn ABI. Task polling invokes the wrapper on the next lifecycle tick, commits
completion, and reads through the runtime handle without LLVM deferred-lowering
notes. Timeout limits
now bind to the same scheduler slot: a zero limit produces a native `TimedOut`
terminal state and a positive limit preserves completed thunk execution.
Cancellation now transitions a pending slot to the native `Cancelled` terminal
state before join. Runtime slot storage is now one normalized thunk packet with
a common invoker and opaque context. All terminal paths and shutdown release
owned contexts. The larger `cli_build_pipeline_demo` also retains its
auto-injected language gate through native LLVM execution. The remaining
task/native closure gap is aggregate payload ownership and a mature worker
executor.

The source frontend now recognizes `ready_after(task, ticks)`, carries it
through every NIR visitor to `cpu.ready_after`, stores overflow-safe ready ticks
in native task slots, and applies completion-at-equal-positive-tick ordering
consistently with the built-in CPU interpreter. Native smoke coverage locks
both completion-before-deadline and timeout-before-readiness behavior. The
same smoke matrix also covers mixed `bool`/`i32` arguments, signed `i32`
returns, and `bool` returns through the normalized eight-byte slot ABI. The
same packed ABI now carries `f32` and `f64` by bit pattern rather than numeric
conversion, with native exact-value smoke coverage. Non-empty recursive source
structs with scalar leaves now encode their complete type tree while
materializing declaration-ordered leaves as tagged scalar/blob slots in one
native `NuisSchedulerOwnedPayloadV1` allocation. Type identity covers the
recursive shape, and one-shot take reconstructs nested field SSA before
drop-hook cleanup. Native mixed `bool`/integer/float/`String` nested field
coverage returns through await, direct join, and TaskResult paths. Text leaves
copy UTF-8 bytes into GLM-tokened task-owned blobs, re-intern on take, and are
released by the common self-describing aggregate drop hook. The shared native
text registration boundary now validates UTF-8 with Rust-compatible strictness;
compiled coverage accepts multibyte Chinese text and rejects overlong,
surrogate, truncated, and out-of-range encodings without leaking blobs.
The aggregate helper now remains a YIR `call_owned_struct` lane and executes
from the lifecycle poll through an owned invoker, rather than being evaluated
at submission. A null owned-invoker result now enters the explicit `Failed`
terminal state, is observable through `task_failed(...)`, and is covered by a
compiled C runtime harness. Native timeout and cancellation probes also prove
that deferred aggregate helpers do not execute before context cleanup. The
explicit Buffer conversion now materializes through LLVM as a GLM-tokened blob,
transfers through recursive task aggregates, and is detached with `take_blob`
before aggregate cleanup. Source Nuis now exposes typed `bytes_len` and
`drop_bytes` operations; GLM rejects reuse after drop, and a native recursive
task smoke returns the expected 24-byte length. The compiler now synthesizes
reverse-declaration-order cleanup for straight-line fallthrough and explicit
returns while preserving return-value evaluation and recognizing explicit drops
plus aggregate ownership transfer. Path-sensitive `if` cleanup now handles
branch-local scope exits, equal ownership-state merges, one-sided early returns,
and two-way terminal returns. Conditional YIR drop-return operations lower to
real LLVM basic blocks, so only the selected path releases the blob.
Ownership-neutral `while` loops may now carry outer bytes unchanged across
backedges and reach normal post-loop cleanup; conditions and nested loop-body
expressions are checked for hidden owned-byte creation or transfer. The
NIR cleanup pass now also releases per-iteration locals before linear-body
fallthrough, direct `break`, and direct `continue`, and GLM verifies the
generated edge cleanup with the outer Buffer lifetime. The backend now covers
both the first resource-aware direct-break loop and iterative counted loops.
`cpu.loop_owned_bytes_copy_drop_break` handles the selected break path, while
extensible `cpu.loop_while_i64_effect` metadata registers
`cpu.owned_bytes_copy_drop` without coupling the generic induction/backedge
skeleton to `Bytes`. Native coverage re-evaluates a changing condition across
two copy/drop iterations; tail `continue` lowers to the same deterministic
copy, update, cleanup, and backedge sequence. Direct guarded `break` now lowers
through `cpu.loop_while_i64_effect_flow`; the selected exit and natural backedge
both cross cleanup, and native aggregate return observes final induction value 2
through exit 26. Effect-flow metadata now also carries linear scalar state:
guarded `continue` skips `add_current`, while the normal update edge applies the
carry before both edges perform registered cleanup. GLM now treats a same-name
`let` after move/drop as a fresh identity. Native payload observation combines
break iteration 2 and carry score 7. Ordered multiple carries now accept
`add_carryN` dependencies on earlier same-edge results and reject forward
references; native `weighted += score` observes 10, producing exit 43 with the
24-byte blob. Uniform-action compound `and`/`or` guards now reuse the recursive
flow condition vocabulary through a length-delimited effect-flow payload; LLVM
evaluates the full tree after the induction update and still releases the blob
exactly once on either selected action or normal update. Carry records are now
arity-driven rather than fixed pairs. The affine multiplicative recurrence
`scaled *= current + 1` composes updated induction state with its invariant
payload only on the two normal update edges, producing factors 4 and 5. Native
multi-state resolution now also shares the common term vocabulary: grouped
`weighted += current + carry0` consumes the earlier same-edge score and reaches
17. Scaled recurrence records now reuse the canonical scaled-source resolver:
`scaled *= (current + 1) * 2` emits `mul_scaled_current_plus_invariant` and
reaches 80. Its invariant-factor ABI stores the additive offset after scaling,
so LLVM resolves it as `terms * factor + scaled_offset`; a native regression
locks this ordering against double scaling. The exit-130 baseline therefore covers compound
continue, multi-state addition, affine multiplication, and scaled multiplication
together. State-driven scaling is also encoded through
`mul_scaled_by_carry0_current_plus_invariant`; its LLVM regression proves that a
later carry reads the earlier carry's new value on the same edge. Remaining gaps
no longer include factor groups: linear effect-flow carries now reuse the
async-post-flow factor-group payload grammar, and
`grouped += (current + carry0) * ((current + -3) * (carry0 + -2))`
reaches 55 in the native aggregate. Exit 185 covers that path together with the
24-byte owned payload. Mixed-action resource controls now use terminal-local
`flow_break`/`flow_continue` tokens and ordered LLVM leaf blocks; recursive
cleanup rewriting releases the iteration blob once on either action or the
normal update path. Nested ownership scopes now recurse safely in NIR cleanup:
inner continue/break edges drop only inner iteration owners and preserve the
outer owner until its own edge. Registered `cpu.scoped_call` actions now
materialize scalar helpers as static function lanes, pass the current iteration
through `$current`, and lower an outer loop whose helper owns an inner Bytes loop
through LLVM without a fixed nested-loop opcode. Scoped helpers now also borrow
an outer `ref Buffer` through one logical YIR parameter expanded to LLVM
`(ptr, len)`; a Lifetime edge spans the loop, and task invokers reject the
borrowed ABI kind. An explicit `copy_bytes(buffer)` scoped argument now becomes
the `copy_owned:<buffer>` descriptor, carries Dep and Lifetime edges, performs a
scheduler-owned deep copy on each iteration, and enters the helper through
`cpu.param_owned_bytes`. Compiler cleanup drops that helper-owned payload
exactly once. Passing an existing `Bytes` value directly is rejected rather
than becoming an implicit clone. Outside scoped loops, `move(Bytes)` lowers
through the general `cpu.move_owned_bytes` operation; interpreted and LLVM
paths preserve the existing blob identity without copying. A scoped
`move(bytes)` becomes `move_owned:<bytes>` only when constant loop facts prove
exactly one execution. Zero-trip, repeating, non-constant, and unnamed-owner
moves are rejected. Direct and recursive helpers now transfer return ownership
through `cpu.return_owned_bytes` / `cpu.call_owned_bytes` and the LLVM `ptr`
ABI; the caller becomes the unique owner without another copy. The remaining
scoped-loop gap no longer includes outer rebinding: `scoped_call_owned_return`
keeps the blob in an LLVM `ptr` backedge slot and `cpu.loop_owned_result`
projects the final owner into the outer binding. GLM treats that projection name
as an output and the moved descriptor as a resource-own access. Dynamic `if`
branches can now converge the same explicit `move(Bytes)` owner through
`cpu.select_owned_bytes`; GLM records resource ownership on the branch inputs
and LLVM emits a native pointer select without copying. Conditional unary
`Bytes -> Bytes` helper returns now use `cpu.branch_call_owned_bytes`: the
helpers are statically outlined, LLVM emits mutually exclusive call blocks,
and a `phi ptr` carries the selected owner forward. A counted segmented YIR ABI
also carries branch-specific pure `bool/i32/i64/f32/f64` arguments without
duplicating the owner or eagerly executing opaque effects. Distinct owners lower
through `cpu.select_owned_bytes_drop_unselected`: GLM owns both candidates,
LLVM drops only the unselected branch value, and a `phi ptr` carries the
survivor. Exact-one scoped-loop moves can now be proved from cycle-safe local
constant chains, integer arithmetic, comparisons, and casts instead of only
literal YIR nodes; unresolved, zero-trip, repeated, and overflowing cases still
fail closed. Nested move-return `if` trees now carry survivor proofs through
`cpu.select_owned_bytes_tree`: a deduplicated owner table
and prefix decision tree let GLM consume aliases once, while LLVM performs
leaf-local cleanup and a multi-entry pointer merge. Leaves now also encode
registered static `(Bytes, scalar...) -> Bytes` helpers with pure scalar
arguments; their scalar dependencies remain explicit and LLVM invokes only the
selected leaf after dropping other owners. Three-arm scalar matches now reuse
the same prefix tree directly, and enum payload matches may
discard pure arm-leading bindings only when the remainder never references
them. Tagged `value`, `variant_field`, and recursively nested `struct_field`
scalar descriptors now provide the selected-leaf projection action required by
payload-using helpers. GLM depends only on the root projection base, while CPU
interpretation and LLVM resolve the complete field path only in the selected
leaf. Wrong variants or missing nested fields in unselected leaves therefore
remain unevaluated. A closed `cast` descriptor now composes all eight existing
NIR scalar conversions with those paths; unknown casts are protocol errors and
LLVM emits conversion instructions only inside the selected leaf. Pointer leaf
policies now admit non-optional `ref Buffer` arguments through direct values and
recursive structure/enum field projections. LLVM represents these borrows as
provenance-carrying `ptr + len` values, so aggregate assembly, variant selection,
and leaf-local projection retain the complete Buffer ABI without relying on a
projected SSA name. They remain read-only GLM dependencies owned and cleaned up
by the caller. Nullable Buffer fields may cross the same leaf ABI only through
`require_non_null(...)` under a matching branch-local null proof. The frontend
encodes a recursive `non_null` descriptor only when the exact source expression
is dominated by the non-null branch; the CPU interpreter rechecks it and LLVM
emits a leaf-local `llvm.assume`. Unproven uses fail closed. Read-only traversal
pointers are now a separate selected-leaf capability: a non-optional `ref Node`
must cross every call boundary through explicit `borrow(...)`, the tree records
`traversal_borrow <descriptor>`, GLM retains a `Read` on the root, and LLVM uses
a single-pointer ABI. The selected leaf rejects a null traversal pointer, while
unselected leaves do not inspect it; ownership and final cleanup remain with the
caller. Traversal pointers cannot be returned or placed in task payloads, and
owned pointer transfer now has a deliberately narrow exact-one contract for
selected helper trees. Every reachable leaf must contain exactly one
`move(<named Node>)` for the same transfer set, encoded as `owned_transfer`;
GLM marks each root as `Own` and requires its lifetime edge. The receiving
helper must contain exactly one `free(...)` on every exit path; verification is
path-sensitive across `if` and early return, while loops remain fail-closed.
Matching conditional effects can be merged before LLVM emission. Differing
effect-only branches now lower through the composition-independent
`cpu.branch_effect` protocol: each leaf carries an ordered list of
`module/instruction/result/arity/(access, operand)` actions. Nustars expose
their supported leaf signatures through the declarative
`BranchEffectActionCapability` registry contract. CPU currently registers
`load_value` as `i64(resource_read)` and `free` as `unit(resource_own)`; GLM
derives `Read` versus `Own` from operand metadata without an instruction-name
white list. NIR semantics exposes registration keys and operands without
lowering metadata, while nuisc obtains result/access plans from the active
static all-Nustar `ModRegistry`; an injected empty registry test proves the
source path fails closed before encoding. The interpreter rejects forged
contracts and evaluates only the selected list, and LLVM emits explicit
then/else/merge blocks plus a
continuation effect edge. Matching terminal `i64` actions can now declare an
`i64` branch-level merge result: CPU returns the selected heap value, LLVM emits
`phi i64`, and an `if` expression retains the merged binding. A native result
smoke executes both leaves in one binary and returns their `41 + 73` sum. The
native selected-transfer smoke runs
both leaves of one binary, observes distinct helper output, exercises a
branch-local load, and confirms a single Node allocation with no deferred tree
lowering.
Asymmetric paths, duplicate moves, null selected transfers, non-consuming
helpers, projected transfers, non-`i64` merge-visible branch-action results,
and task or return transport remain closed. Branch composition execution is
now hosted by YIR core rather than `CpuMod`: registry validation covers every
leaf, selected execution delegates to the owning `RegisteredMod`, and
`execute_module_with_registry` proves an injected `probe` Nustar can return the
selected `i64` value under a CPU composition parent. LLVM action emission uses
`BranchEffectLlvmEmitterRegistry`; `emit_module_with_registries` proves an
injected probe emitter can generate both values and the common `phi` without
changing the composition loop. Registered YIR actions with no matching LLVM
emitter fail closed. The ordinary source and project AOT paths now load the
manifests named by `loaded_nustar`, resolve static providers by
`yir_lowering_entry`, and pass the assembled YIR/emitter registries into LLVM.
CPU and AArch64 CPU install their emitters through this path. Provider
descriptors now live in the LLVM backend's static Nustar catalog, so `nuisc`
contains no CPU entry names or emitter functions. The paired YIR semantic
registry is now assembled from the same manifests through a verifier-owned
provider catalog. Unloaded domains remain absent, unknown providers fail during
assembly, and a catalog-coverage test locks every indexed official manifest.
Branch composition also has its first ownership-carrying result:
`owned_ptr` requires both paths to consume the same two live, distinct,
unborrowed owners through `cpu.take_ptr_drop_other`. Interpretation frees only
the discarded object, GLM returns `Res`, heap verification moves both source
names, and LLVM emits path-local frees plus `phi ptr`. Typed source lowering is
now exposed as `select_owned_ptr(condition, move(left), move(right))`. Both
candidates must be same-typed, named, distinct owners; NIR verification rejects
aliasing and any later reuse, while cleanup synthesis removes both consumed
inputs. The YIR merge now carries explicit `address_kind=node|buffer` and
`nullable=true|false` metadata. Heap verification rejects kind/object mismatch;
source may widen two live owners into an optional result but still rejects
nullable candidates. `owned_pointer_select_demo` executes Node,
nullable-result, and Buffer selections, reaches both `load_value` and
`load_at`, and proves final survivor cleanup in a native binary with exit `78`.
Projected and task-carried address results remain closed.
The runtime now defines `NuisSchedulerOwnedBlobV1` as the first GLM-tokened
dynamic leaf primitive. It deep-copies borrowed bytes and has scheduler-native
move/drop hooks; a compiled harness covers take and cancellation. Recursive
String lowering now consumes it through self-describing aggregate slots, while
borrowed Buffer remains deliberately unavailable as a task input. The new
source-level `copy_bytes(ref Buffer) -> Bytes` conversion now reaches
`cpu.copy_buffer_owned`; interpreted YIR deep-copies the elements and remains
independent after source mutation. LLVM now emits the byte copy, recursive task
packing, and ownership-taking unpack path. Source-level observation and explicit
destruction now reach the same runtime. Straight-line exits and path-sensitive
`if` exits synthesize cleanup through that runtime, including real conditional
LLVM drop-return blocks. Ownership-neutral loops preserve outer owners across
backedges and reach post-loop cleanup. Linear per-iteration ownership cleanup is
synthesized and GLM-verified; direct-break and iterative counted copy/drop forms
now reach native LLVM, including changing-condition fallthrough and tail
`continue`. Conditional resource flow is covered, and nested resource loops
compose through static scoped helpers. Borrowed Buffer capture preserves
pointer, length, and lifetime metadata; owned resource transfer across that
boundary remains open.
Aggregate construction now has a transactional `finish` boundary: unset,
duplicate, or invalid slots poison the build and release already attached
blobs. Deferred helpers surface null as `Failed`, while immediate awaits reject
partial aggregates deterministically.
Direct floating literals inside
spawned calls still need stronger callee-parameter expected-type propagation;
explicitly typed bindings currently preserve the intended `f32` boundary.

The Nustar checks anchor the bootstrap-critical
`heterogeneous-runtime/nustar/registered-domain-contracts` cell to:

* `tools/nuisc/src/registry_contract.rs`
* `tools/nuisc/src/registry_domain_json.rs`
* `tools/nuis/src/surface_render/link_plan.rs`
* `tools/nuis/src/workflow/link_plan_domain.rs`

That keeps shader/kernel/network execution readiness in the registry contract
surface itself. Nuis workflow and link-plan readiness now consume the registry
dispatch readiness status, missing signals, bridge materialization, and
execution-readiness materialization for each heterogeneous domain. Nsld final
output blocker ordering is still the next integration point; the current
frontdoor deliberately exposes enough normalized facts for that step without
hardcoding shader/kernel/network-specific logic.

The native-binary checks anchor the bootstrap-critical
`native-binary-system/nsb-nsld/self-owned-binary-assembly` cell to the shared
Nsld final-output replay vocabulary. Nsld still owns the concrete object and
package summaries, while Nsdb owns the YIR replay transcript contract, but Nuis
frontdoors now mirror both as abstract readiness fields:
`nsld_final_executable_output_object_package_*`,
`nsld_final_executable_output_debugger_transcript_*`, and
`closure_summary_*_debugger_transcript_*`. This keeps run-artifact, workflow,
project-status, and release/build-report surfaces aligned without coupling the
frontdoor to Mach-O, ELF, PE, or any one future object format. Nsdb now layers
`nsdb-yir-replay-control-v1` over that transcript: `--frame` consumes one exact
index or frame id, while `--break-at` consumes the ordered prefix through an
exact frame and reports `breakpoint-hit`. Missing or ambiguous targets fail
closed. Typed `execution_phase` and `entry_symbol` predicates now stop at the
first ordered AND-match through `nsdb-yir-breakpoint-predicate-v1`. Every
successful stop emits `nsdb-yir-replay-resume-cursor-v1` with the stopped frame
and deterministic next frame, or an explicit terminal status.
`nsdb-yir-replay-resume-input-v1` now consumes a stopped/next frame pair only
when both resolve as immediate neighbors, then replays the suffix from the next
frame; stale, mismatched, incomplete, and terminal cursors consume nothing. The
PixelMagic smoke now proves a real multi-checkpoint stop-resume-stop command
chain against heterogeneous trace records: Nsdb falls back from absent
payload-handoff events to ordered metadata/device-dispatch trace frames, persists
`nsdb-yir-replay-cursor-record-v1`, resumes exactly at the advertised successor,
and stops again through `--resume-cursor`. Cursor loading validates the record,
transcript/source contracts, and manifest before applying the exact successor.
Nuis adapts that file through `nuis-debugger-cursor-handoff-v1`, mirroring its
expected path, readiness, and status through final-output and closure summaries
without importing Nsdb types. Missing cursors remain optional/unavailable while
malformed, stale-contract, and wrong-manifest records are invalid. The next gap
was a cursor-specific resume command handoff; ready mirrors now publish that
command through final-output and closure summaries, while unavailable/invalid
mirrors publish none. Nuis now owns a first-class `debug-resume` route that
validates the abstract handoff before dispatching Nsdb with structured argv;
unavailable/invalid cursors fail before dispatch. Exact and typed breakpoint
controls plus optional cursor persistence now flow through that route. The
PixelMagic proof now uses real data, kernel, and shader records to save at the
first checkpoint, replace the cursor at the second, and resume to the third.
That work also removed the compiler's global first-two/next-two data-pipe
assumption: registered data units are stitched and validated through their own
handle-table and window ancestry. Cursor replacement now uses a synced,
same-directory temporary file, validates it through the normal loader, then
atomically renames it; invalid replacements preserve the previous cursor.
An optional sibling lineage sidecar retains the latest eight replacements as a
monotonic FNV-1a hash chain over public cursor identities. A damaged sidecar is
preserved without invalidating the authoritative cursor. Nuis does not import
Nsdb types: its artifact adapter mirrors lineage protocol, path,
readiness, status, bounded depth, and latest hash through final-output and
closure summaries. The latest hash must match the authoritative cursor bytes.
Invalid lineage now carries a stable blocker, repair action, and executable
Nsdb command. Repair validates the authoritative cursor, archives the damaged
sidecar under a content hash, atomically rebuilds one current entry, and is
idempotent once healthy. Nuis does not yet own a first-class repair command;
native execution remains outside this metadata-level debugger control.

## Current Role

The first implementation is static and intentionally conservative. It is not a
replacement for tests, release checklists, or Nsld/Nuis frontdoor reports.

It is a development-system index over those surfaces, with a small drift-check
layer over the most bootstrap-critical anchors.

The first useful jobs are:

* keep CLI closure, Nsld, std, language-core, Nustar, and native-binary work in
  one comparable view
* make weak cells explicit instead of hiding them in broad status prose
* separate `host runnable`, `Nsld-owned ready`, and `self-owned binary assembly`
  as different functions instead of one overloaded "binary works" claim
* let `nuis` name the weakest bootstrap-critical coordinate without requiring
  a human to reread the whole roadmap
* give alpha milestones a structured progress vocabulary before beta
  self-hosting pressure grows

## Current Honesty Boundary

The tensor is a progress model, not a contract freeze.

In alpha it may change cell names aggressively when the architecture changes.
The stable part is the coordinate idea:

`architecture x module x function -> status/progress/evidence/next_step/task-card`

The task-card layer is intentionally small: protocol/source/status/ready,
handoff metadata, `blocker`, `next_action`, `validation_command`, and
`expected_artifact`. It lets the weakest bootstrap coordinate become a concrete
work item without turning the tensor into a full issue tracker.

Future work should move cells from static entries toward generated readings
from:

* checked tests
* frontdoor JSON fields
* Nsld reports
* docs/reference anchors
* package manifests
* roadmap milestones

The first drift checks are intentionally narrow. Future checks should become
milestone-owned instead of merely field-owned, so they can verify examples,
packages, and command workflows as well as names in source files.
