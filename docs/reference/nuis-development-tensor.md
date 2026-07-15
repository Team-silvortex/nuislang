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
* `coverage_status`
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

`nuis status` also prints the short tensor summary. That makes the model part
of the toolchain self-orientation surface, not just a separate report command.

## Coverage Manifest

The tensor now has a small built-in coverage manifest. The manifest lists the
coordinates that the alpha line expects to see in the tensor:

`expected architecture/module/function coordinates`

The coverage layer compares that expected coordinate set with the actual
`DEV_TENSOR_CELLS` entries and reports:

* `coverage_status`
  `clean` when required expected coordinates are covered and no stale/orphaned
  cells are present; otherwise `gap`
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
versions can derive expected coordinates from galaxy manifests, Nustar
registries, std module manifests, and milestone files.

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
checked snapshot and fallback. The important change is that the tensor now
derives a second coordinate view from the milestone manifest and compares all
three sides:

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

Short rule:

`milestone coverage makes the tensor less hand-written: milestones own the map,
Rust constants must prove they still mirror it`

The next step is to make `DEV_TENSOR_EXPECTED_COORDINATES` generated or cached
from this manifest instead of treating the manifest as a parity peer.

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
* `examples/projects/tooling/README.md`
* `stdlib/std/README.md`

That keeps the standard-library progress cell tied to the project-form
filesystem, IO, text, terminal, and tooling smoke chain instead of only a broad
roadmap phrase. The current std evidence also includes the observable CLI smoke
`std_tooling_observable_cli_smoke_checks_reports_and_stdin`, which checks
`run-artifact --json` prelaunch readiness, stdout/stderr report output from the
host IO report lane, direct stdin consumption by the built binary, and
`host_stdin_read` / `host_stdout_write` / `host_stderr_write` lowering anchors.

The language-core checks anchor the bootstrap-critical
`language-core/nuisc/type-control-flow-generics` cell to:

* `tools/nuis/tests/language_bootstrap_smoke.rs`
* `examples/projects/task/task_result_enum_demo`
* `examples/projects/state/generic_method_bound_guarded_nested_match_demo`
* `examples/projects/state/glm_buffer_roundtrip_state_demo`

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
YIR lifetime/effect edges around `cpu.store_at` / `cpu.free`. The next gap is
to combine these once-separate language proofs into a std-style helper workload
that mixes `Result`, `Buffer`, lambdas, trait bounds, and pointer-heavy control
flow in one project.

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

`architecture x module x function -> status/progress/evidence/next_step`

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
